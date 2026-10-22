//! Biblioteca de sincronização bidirecional entre rekordbox.xml e mixxxdb.sqlite.
//!
//! Reproduz rigorosamente as regras de segurança, conversões e formatos de `tools/rekordbox/sync.py`.

use chrono::Local;
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use rusqlite::{params, Connection, OpenFlags, OptionalExtension, Transaction};
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

// Mesmo conjunto de urllib.parse.quote(safe="/"): só letras, dígitos, "_.-~" e "/"
// ficam sem codificação.
const PATH_SAFE: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'/')
    .remove(b'_')
    .remove(b'.')
    .remove(b'-')
    .remove(b'~');

#[derive(Debug)]
pub enum SyncError {
    Io(std::io::Error),
    Sqlite(rusqlite::Error),
    Xml(String),
    Validation(String),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "Erro de E/S: {}", e),
            Self::Sqlite(e) => write!(f, "Erro SQLite: {}", e),
            Self::Xml(e) => write!(f, "Erro XML: {}", e),
            Self::Validation(e) => write!(f, "Erro de validação: {}", e),
        }
    }
}

impl std::error::Error for SyncError {}

impl From<std::io::Error> for SyncError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<rusqlite::Error> for SyncError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e)
    }
}

impl From<quick_xml::Error> for SyncError {
    fn from(e: quick_xml::Error) -> Self {
        Self::Xml(e.to_string())
    }
}

/// Valor do atributo com entidades resolvidas (`&amp;` vira `&`), como no ElementTree.
fn attr_text(attr: &Attribute) -> Result<String, SyncError> {
    Ok(attr.unescape_value()?.into_owned())
}

/// `true` para `C:`, `C:/...` ou `C:\...`.
fn has_drive(path: &str) -> bool {
    let b = path.as_bytes();
    b.len() >= 2
        && b[0].is_ascii_alphabetic()
        && b[1] == b':'
        && (b.len() == 2 || b[2] == b'/' || b[2] == b'\\')
}

/// `os.path.abspath` do Python: junta ao diretório atual e normaliza `.` e `..`
/// sem resolver links simbólicos. Caminhos POSIX ficam iguais ao `sync.py`;
/// caminhos com unidade (`C:\x`, como o Mixxx no Windows grava `C:/x`) viram
/// `/C:/x`, a forma usada pelo rekordbox em `file://localhost/C:/x`.
fn abspath(path_str: &str) -> String {
    // Barra invertida só é separador no Windows; no POSIX é um caractere válido.
    let path = if cfg!(windows) || has_drive(path_str) {
        path_str.replace('\\', "/")
    } else {
        path_str.to_string()
    };
    let joined = if path.starts_with('/') || has_drive(&path) {
        path
    } else {
        let cwd = std::env::current_dir()
            .map(|d| d.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        format!("{}/{}", cwd, path)
    };
    let mut parts: Vec<&str> = Vec::new();
    for part in joined.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                // A unidade é a raiz: `..` não sobe acima dela.
                if parts.len() > 1 || !parts.first().is_some_and(|p| has_drive(p)) {
                    parts.pop();
                }
            }
            p => parts.push(p),
        }
    }
    format!("/{}", parts.join("/"))
}

/// `round()` do Python: empate vai para o par.
fn round_half_even(value: f64) -> i64 {
    if (value - value.trunc()).abs() == 0.5 {
        (2.0 * (value / 2.0).round()) as i64
    } else {
        value.round() as i64
    }
}

/// Converte cor inteira do Mixxx (0xRRGGBB) para tupla (r, g, b).
pub fn int_to_rgb(color_int: Option<i64>) -> (u8, u8, u8) {
    match color_int {
        Some(c) if c >= 0 => {
            let r = ((c >> 16) & 0xFF) as u8;
            let g = ((c >> 8) & 0xFF) as u8;
            let b = (c & 0xFF) as u8;
            (r, g, b)
        }
        _ => (0, 122, 255), // Pioneer blue default
    }
}

/// Converte tupla (r, g, b) para inteiro 0xRRGGBB do Mixxx.
pub fn rgb_to_int(r: u8, g: u8, b: u8) -> i64 {
    ((r as i64) << 16) | ((g as i64) << 8) | (b as i64)
}

/// Converte caminho de arquivo para URL `file://localhost/...`.
pub fn path_to_file_url(path_str: &str) -> String {
    let abs = abspath(path_str);
    // O `:` da unidade fica literal (`/C:/`); o resto segue `quote(safe="/")`.
    let (drive, rest) = if has_drive(&abs[1..]) {
        abs.split_at(3)
    } else {
        ("", abs.as_str())
    };
    format!(
        "file://localhost{}{}",
        drive,
        utf8_percent_encode(rest, PATH_SAFE)
    )
}

/// Converte URL `file://localhost/...` para caminho local do sistema.
pub fn file_url_to_path(url: &str) -> String {
    let stripped = if let Some(rest) = url.strip_prefix("file://localhost") {
        rest
    } else if let Some(rest) = url.strip_prefix("file://") {
        rest
    } else {
        url
    };
    let path = percent_decode_str(stripped).decode_utf8_lossy().to_string();
    // `/C:/x` volta a `C:/x`, como o Mixxx grava no Windows.
    match path.strip_prefix('/') {
        Some(rest) if has_drive(rest) => rest.to_string(),
        _ => path,
    }
}

/// Verifica se o processo Mixxx / VILELABOX está em execução.
pub fn assert_mixxx_stopped() -> Result<(), SyncError> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if let Some(s) = name.to_str() {
                    if s.chars().all(|c| c.is_ascii_digit()) {
                        let names = ["mixxx", "blackmesa", "vilelabox"];
                        let comm = fs::read_to_string(entry.path().join("comm"))
                            .map(|c| c.trim().to_lowercase())
                            .unwrap_or_default();
                        let exe = fs::read_link(entry.path().join("exe"))
                            .ok()
                            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()))
                            .unwrap_or_default();
                        if names.contains(&comm.as_str()) || names.contains(&exe.as_str()) {
                            return Err(SyncError::Validation(
                                "Feche o Mixxx/VILELABOX antes de importar no banco.".into(),
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Cria backup consistente via API do SQLite, por outra conexão (inclui o WAL).
/// Na importação é chamada com a transação IMMEDIATE já aberta: ver `protected_import`.
pub fn create_consistent_backup(db_path: &Path) -> Result<PathBuf, SyncError> {
    if !db_path.is_file() {
        return Err(SyncError::Validation(format!(
            "Banco do Mixxx não encontrado em: {}",
            db_path.display()
        )));
    }

    let backup_dir = db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("backups-rekordbox");
    fs::create_dir_all(&backup_dir)?;

    let stamp = Local::now().format("%Y%m%d-%H%M%S-%6f").to_string();
    let db_name = db_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("mixxxdb.sqlite");
    let backup_path = backup_dir.join(format!("{}-{}.bak", db_name, stamp));

    // Conexão fonte somente leitura: o backup inclui o WAL.
    let src_conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    src_conn.busy_timeout(std::time::Duration::ZERO)?;
    let mut dst_conn = Connection::open(&backup_path)?;

    let backup = rusqlite::backup::Backup::new(&src_conn, &mut dst_conn)?;
    backup.run_to_completion(5, std::time::Duration::from_millis(50), None)?;

    // Manter apenas os 10 últimos backups do mesmo banco
    let prefix = format!("{}-", db_name);
    if let Ok(entries) = fs::read_dir(&backup_dir) {
        let mut backups: Vec<(std::path::PathBuf, std::time::SystemTime)> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|path| {
                path.is_file()
                    && path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.starts_with(&prefix) && n.ends_with(".bak"))
                        .unwrap_or(false)
            })
            .filter_map(|path| {
                let mtime = path.metadata().and_then(|m| m.modified()).ok()?;
                Some((path, mtime))
            })
            .collect();
        backups.sort_by_key(|a| std::cmp::Reverse(a.1));
        for (old_path, _) in backups.into_iter().skip(10) {
            let _ = fs::remove_file(old_path);
        }
    }

    Ok(backup_path)
}

/// Mesmas garantias de `protected_import_connection` em `sync.py`: recusa com o
/// Mixxx aberto, trava escritores (BEGIN IMMEDIATE, sem espera) *antes* do
/// backup, confere o processo de novo, faz o backup e só então executa `f`.
/// Erro em qualquer passo desfaz a transação (o `Transaction` é descartado).
pub fn protected_import<T>(
    db_path: &Path,
    f: impl FnOnce(&Transaction) -> Result<T, SyncError>,
) -> Result<T, SyncError> {
    protected_import_checked(db_path, &assert_mixxx_stopped, f)
}

fn protected_import_checked<T>(
    db_path: &Path,
    check_stopped: &dyn Fn() -> Result<(), SyncError>,
    f: impl FnOnce(&Transaction) -> Result<T, SyncError>,
) -> Result<T, SyncError> {
    check_stopped()?;
    if !db_path.is_file() {
        return Err(SyncError::Validation(format!(
            "Banco do Mixxx não encontrado em: {}",
            db_path.display()
        )));
    }
    let mut conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    conn.busy_timeout(std::time::Duration::ZERO)?;
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    check_stopped()?;
    let backup_path = create_consistent_backup(db_path)?;
    println!("[*] Backup criado: {}", backup_path.display());
    let result = f(&tx)?;
    tx.commit()?;
    Ok(result)
}

#[derive(Debug, Clone, Default)]
pub struct BeatsInfo {
    pub bpm: Option<f64>,
    pub first_beat_frame: Option<i64>,
    pub first_downbeat_index: i32,
    pub beats: Vec<i64>,
}

fn decode_varint_slice(data: &[u8], mut offset: usize) -> Option<(u64, usize)> {
    let mut res = 0u64;
    let mut shift = 0;
    while offset < data.len() {
        let b = data[offset];
        offset += 1;
        res |= ((b & 0x7F) as u64) << shift;
        if b & 0x80 == 0 {
            return Some((res, offset));
        }
        shift += 7;
        if shift > 64 {
            return None;
        }
    }
    None
}

fn zigzag_decode_i32(n: u64) -> i32 {
    ((n >> 1) as i32) ^ (-((n & 1) as i32))
}

pub fn parse_beats_blob(blob: &[u8]) -> Option<BeatsInfo> {
    if blob.is_empty() {
        return None;
    }
    let mut offset = 0;
    let mut info = BeatsInfo::default();

    while offset < blob.len() {
        let (tag, new_offset) = decode_varint_slice(blob, offset)?;
        offset = new_offset;
        let field_num = tag >> 3;
        let wire_type = tag & 0x07;

        match wire_type {
            0 => {
                let (val, new_offset) = decode_varint_slice(blob, offset)?;
                offset = new_offset;
                if field_num == 2 || field_num == 3 {
                    info.first_downbeat_index = zigzag_decode_i32(val);
                }
            }
            1 => {
                offset += 8;
            }
            2 => {
                let (len, new_offset) = decode_varint_slice(blob, offset)?;
                offset = new_offset;
                let end = offset + len as usize;
                if end > blob.len() {
                    break;
                }
                let field_data = &blob[offset..end];
                offset = end;

                if field_num == 1 {
                    if field_data.len() >= 9 && field_data[0] == 9 {
                        let bytes: [u8; 8] = field_data[1..9].try_into().ok()?;
                        info.bpm = Some(f64::from_le_bytes(bytes));
                    } else if field_data.len() >= 2 && field_data[0] == 8 {
                        if let Some((frame, _)) = decode_varint_slice(field_data, 1) {
                            info.beats.push(frame as i64);
                        }
                    }
                } else if field_num == 2 && field_data.len() >= 2 && field_data[0] == 8 {
                    if let Some((frame, _)) = decode_varint_slice(field_data, 1) {
                        info.first_beat_frame = Some(frame as i64);
                    }
                }
            }
            5 => {
                offset += 4;
            }
            _ => break,
        }
    }
    Some(info)
}

#[derive(Debug, Clone, PartialEq)]
pub struct TempoSection {
    pub inizio_sec: f64,
    pub bpm: f64,
    pub battito: i32,
}

pub fn extract_tempos(
    beats_info: Option<&BeatsInfo>,
    fallback_bpm: f64,
    samplerate: f64,
) -> Vec<TempoSection> {
    if let Some(info) = beats_info {
        if info.beats.len() >= 2 {
            let mut tempos = Vec::new();
            let mut cur_bpm: Option<f64> = None;
            let downbeat_idx = info.first_downbeat_index;

            for i in 0..info.beats.len() - 1 {
                let delta = (info.beats[i + 1] - info.beats[i]) as f64 / samplerate;
                if delta <= 0.0 {
                    continue;
                }
                let b_bpm = (60.0 / delta * 100.0).round() / 100.0;
                let change = match cur_bpm {
                    None => true,
                    Some(prev) => (b_bpm - prev).abs() >= 0.05,
                };
                if change {
                    cur_bpm = Some(b_bpm);
                    let inizio_sec = (info.beats[i] as f64 / samplerate).max(0.0);
                    let battito = 1 + ((((i as i32) - downbeat_idx) % 4 + 4) % 4);
                    tempos.push(TempoSection {
                        inizio_sec,
                        bpm: b_bpm,
                        battito,
                    });
                }
            }
            if !tempos.is_empty() {
                return tempos;
            }
        }

        let first_frame = info.first_beat_frame;
        let bpm = info.bpm.unwrap_or(fallback_bpm);
        let downbeat_idx = info.first_downbeat_index;
        let battito = 1 + (((-downbeat_idx) % 4 + 4) % 4);
        let inizio_sec = first_frame
            .map(|f| (f as f64 / samplerate).max(0.0))
            .unwrap_or(0.0);
        return vec![TempoSection {
            inizio_sec,
            bpm,
            battito,
        }];
    }

    vec![TempoSection {
        inizio_sec: 0.0,
        bpm: fallback_bpm,
        battito: 1,
    }]
}

#[derive(Debug, Clone)]
pub struct TrackExportData {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub duration: f64,
    pub bpm: Option<f64>,
    pub key: String,
    pub rating: i64,
    pub bitrate: i64,
    pub samplerate: i64,
    pub comment: String,
    pub location: String,
    pub beats: Option<Vec<u8>>,
    pub cues: Vec<CueExportData>,
}

#[derive(Debug, Clone)]
pub struct CueExportData {
    pub cue_type: i64, // 1 = HotCue, 4 = Loop
    // O Mixxx grava posições fracionárias (REAL) em colunas INTEGER.
    pub position: f64,
    pub length: f64,
    pub hotcue: i64,
    pub label: String,
    pub color: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct PlaylistExportData {
    pub name: String,
    pub track_ids: Vec<i64>,
}

/// Exporta dados do Mixxx SQLite para rekordbox.xml.
pub fn export_mixxx_to_rekordbox_xml(
    db_path: &Path,
    output_xml: &Path,
) -> Result<(usize, usize), SyncError> {
    if !db_path.is_file() {
        return Err(SyncError::Validation(format!(
            "Banco do Mixxx não encontrado: {}",
            db_path.display()
        )));
    }

    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    // 1. Carrega faixas
    let has_beats: bool = {
        let mut pragma = conn.prepare("PRAGMA table_info(library)")?;
        let rows = pragma.query_map([], |row| row.get::<_, String>(1))?;
        let mut found = false;
        for r in rows.flatten() {
            if r == "beats" {
                found = true;
                break;
            }
        }
        found
    };

    let query = if has_beats {
        "SELECT l.id, l.artist, l.title, l.album, l.genre, l.bpm, l.duration,
                l.key, l.rating, l.bitrate, l.samplerate, l.comment, tl.location, l.beats
         FROM library l
         JOIN track_locations tl ON l.location = tl.id
         WHERE tl.fs_deleted = 0
         ORDER BY l.id ASC"
    } else {
        "SELECT l.id, l.artist, l.title, l.album, l.genre, l.bpm, l.duration,
                l.key, l.rating, l.bitrate, l.samplerate, l.comment, tl.location, NULL
         FROM library l
         JOIN track_locations tl ON l.location = tl.id
         WHERE tl.fs_deleted = 0
         ORDER BY l.id ASC"
    };

    let mut stmt = conn.prepare(query)?;

    let track_rows = stmt.query_map([], |row| {
        Ok(TrackExportData {
            id: row.get(0)?,
            artist: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            title: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            album: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            genre: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
            bpm: row.get(5)?,
            duration: row.get::<_, Option<f64>>(6)?.unwrap_or(0.0),
            key: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
            rating: row.get::<_, Option<i64>>(8)?.unwrap_or(0),
            // `or` do Python: NULL e 0 usam o padrão.
            bitrate: row
                .get::<_, Option<i64>>(9)?
                .filter(|v| *v != 0)
                .unwrap_or(320),
            samplerate: row
                .get::<_, Option<i64>>(10)?
                .filter(|v| *v != 0)
                .unwrap_or(44100),
            comment: row.get::<_, Option<String>>(11)?.unwrap_or_default(),
            location: row.get::<_, Option<String>>(12)?.unwrap_or_default(),
            beats: row.get(13)?,
            cues: Vec::new(),
        })
    })?;

    let mut tracks = Vec::new();
    let mut track_id_map = HashMap::new();

    for (idx, t_res) in track_rows.enumerate() {
        let mut track = t_res?;
        let rb_id = (idx + 1) as i64;
        track_id_map.insert(track.id, rb_id);

        // Carrega cues e loops da faixa
        let mut cue_stmt = conn.prepare(
            "SELECT type, position, length, hotcue, label, color
             FROM cues
             WHERE track_id = ? AND (type = 1 OR type = 4)
             ORDER BY hotcue ASC, position ASC",
        )?;
        let cues = cue_stmt
            .query_map(params![track.id], |crow| {
                Ok(CueExportData {
                    cue_type: crow.get(0)?,
                    position: crow.get(1)?,
                    length: crow.get::<_, Option<f64>>(2)?.unwrap_or(0.0),
                    hotcue: crow.get(3)?,
                    label: crow.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    color: crow.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        track.cues = cues;
        tracks.push(track);
    }

    // 2. Carrega playlists de usuário
    let mut pl_stmt = conn.prepare(
        "SELECT id, name FROM Playlists
         WHERE hidden = 0
         ORDER BY position ASC, id ASC",
    )?;

    let mut playlists = Vec::new();
    let pl_rows = pl_stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;

    for pl in pl_rows {
        let (pl_id, name) = pl?;
        let mut pt_stmt = conn.prepare(
            "SELECT track_id FROM PlaylistTracks
             WHERE playlist_id = ?
             ORDER BY position ASC",
        )?;
        let tids = pt_stmt
            .query_map(params![pl_id], |prow| prow.get::<_, i64>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        playlists.push(PlaylistExportData {
            name,
            track_ids: tids,
        });
    }

    // 3. Carrega histórico recente
    let mut hist_stmt = conn.prepare(
        "SELECT id, name FROM Playlists
         WHERE hidden = 2
         ORDER BY id DESC
         LIMIT 20",
    )?;
    let mut history_playlists = Vec::new();
    let mut has_history = false;
    let hist_rows = hist_stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;

    for h in hist_rows {
        has_history = true;
        let (h_id, name) = h?;
        let mut pt_stmt = conn.prepare(
            "SELECT track_id FROM PlaylistTracks
             WHERE playlist_id = ?
             ORDER BY position ASC",
        )?;
        let tids = pt_stmt
            .query_map(params![h_id], |prow| prow.get::<_, i64>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        if !tids.is_empty() {
            history_playlists.push(PlaylistExportData {
                name,
                track_ids: tids,
            });
        }
    }

    // Gera o documento XML
    let mut xml_bytes = Vec::new();
    {
        let mut writer = Writer::new_with_indent(Cursor::new(&mut xml_bytes), b' ', 2);
        writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;

        // <DJ_PLAYLISTS Version="1.0.0">
        let mut dj = BytesStart::new("DJ_PLAYLISTS");
        dj.push_attribute(("Version", "1.0.0"));
        writer.write_event(Event::Start(dj))?;

        // <PRODUCT Name="rekordbox" Version="6.8.5" Company="AlphaTheta"/>
        let mut prod = BytesStart::new("PRODUCT");
        prod.push_attribute(("Name", "rekordbox"));
        prod.push_attribute(("Version", "6.8.5"));
        prod.push_attribute(("Company", "AlphaTheta"));
        writer.write_event(Event::Empty(prod))?;

        // <COLLECTION Entries="...">
        let mut coll = BytesStart::new("COLLECTION");
        let entries_str = tracks.len().to_string();
        coll.push_attribute(("Entries", entries_str.as_str()));
        writer.write_event(Event::Start(coll))?;

        for (idx, t) in tracks.iter().enumerate() {
            let track_id_str = (idx + 1).to_string();
            let loc_url = path_to_file_url(&t.location);
            let duration_sec = round_half_even(t.duration).to_string();
            let bpm_val = if let Some(b) = t.bpm.filter(|b| *b != 0.0) {
                format!("{:.2}", b)
            } else {
                "120.00".to_string()
            };
            let rating_str = t.rating.to_string();
            let bitrate_str = t.bitrate.to_string();
            let samplerate_str = t.samplerate.to_string();

            let name_val = if t.title.is_empty() {
                Path::new(&t.location)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string()
            } else {
                t.title.clone()
            };

            let mut track_el = BytesStart::new("TRACK");
            track_el.push_attribute(("TrackID", track_id_str.as_str()));
            track_el.push_attribute(("Name", name_val.as_str()));
            track_el.push_attribute(("Artist", t.artist.as_str()));
            track_el.push_attribute(("Album", t.album.as_str()));
            track_el.push_attribute(("Genre", t.genre.as_str()));
            track_el.push_attribute(("TotalTime", duration_sec.as_str()));
            track_el.push_attribute(("AverageBpm", bpm_val.as_str()));
            track_el.push_attribute(("Key", t.key.as_str()));
            track_el.push_attribute(("Rating", rating_str.as_str()));
            track_el.push_attribute(("BitRate", bitrate_str.as_str()));
            track_el.push_attribute(("SampleRate", samplerate_str.as_str()));
            track_el.push_attribute(("Comments", t.comment.as_str()));
            track_el.push_attribute(("Location", loc_url.as_str()));
            writer.write_event(Event::Start(track_el))?;

            // Beatgrid / TEMPO
            let beats_info = t.beats.as_deref().and_then(parse_beats_blob);
            let fallback_bpm = t.bpm.filter(|b| *b != 0.0).unwrap_or(120.0);
            let tempos = extract_tempos(beats_info.as_ref(), fallback_bpm, t.samplerate as f64);
            for ts in tempos {
                let mut tempo = BytesStart::new("TEMPO");
                let inizio_str = format!("{:.3}", ts.inizio_sec);
                let bpm_str = format!("{:.2}", ts.bpm);
                let battito_str = ts.battito.to_string();
                tempo.push_attribute(("Inizio", inizio_str.as_str()));
                tempo.push_attribute(("Bpm", bpm_str.as_str()));
                tempo.push_attribute(("Metro", "4/4"));
                tempo.push_attribute(("Battito", battito_str.as_str()));
                writer.write_event(Event::Empty(tempo))?;
            }

            // Cues / POSITION_MARK
            let sr = t.samplerate as f64;
            for c in &t.cues {
                let (r, g, b) = int_to_rgb(c.color);
                let time_start_sec = (c.position / (sr * 2.0)).max(0.0);
                let time_start_str = format!("{:.3}", time_start_sec);
                let num_str = c.hotcue.to_string();
                let r_str = r.to_string();
                let g_str = g.to_string();
                let b_str = b.to_string();

                let mut pos_mark = BytesStart::new("POSITION_MARK");
                if c.cue_type == 4 {
                    // Loop
                    let time_end_sec = ((c.position + c.length) / (sr * 2.0)).max(time_start_sec);
                    let time_end_str = format!("{:.3}", time_end_sec);
                    let label = if !c.label.is_empty() {
                        c.label.clone()
                    } else if c.hotcue >= 0 {
                        format!("Hot Loop {}", c.hotcue + 1)
                    } else {
                        "Loop".to_string()
                    };

                    pos_mark.push_attribute(("Name", label.as_str()));
                    pos_mark.push_attribute(("Type", "1"));
                    pos_mark.push_attribute(("Start", time_start_str.as_str()));
                    pos_mark.push_attribute(("End", time_end_str.as_str()));
                    pos_mark.push_attribute(("Num", num_str.as_str()));
                    pos_mark.push_attribute(("Red", r_str.as_str()));
                    pos_mark.push_attribute(("Green", g_str.as_str()));
                    pos_mark.push_attribute(("Blue", b_str.as_str()));
                } else {
                    // Hot Cue
                    let label = if !c.label.is_empty() {
                        c.label.clone()
                    } else if c.hotcue >= 0 {
                        format!("Hot Cue {}", c.hotcue + 1)
                    } else {
                        "Cue".to_string()
                    };

                    pos_mark.push_attribute(("Name", label.as_str()));
                    pos_mark.push_attribute(("Type", "0"));
                    pos_mark.push_attribute(("Start", time_start_str.as_str()));
                    pos_mark.push_attribute(("Num", num_str.as_str()));
                    pos_mark.push_attribute(("Red", r_str.as_str()));
                    pos_mark.push_attribute(("Green", g_str.as_str()));
                    pos_mark.push_attribute(("Blue", b_str.as_str()));
                }
                writer.write_event(Event::Empty(pos_mark))?;
            }

            writer.write_event(Event::End(BytesEnd::new("TRACK")))?;
        }
        writer.write_event(Event::End(BytesEnd::new("COLLECTION")))?;

        // <PLAYLISTS>
        writer.write_event(Event::Start(BytesStart::new("PLAYLISTS")))?;
        let mut root_node = BytesStart::new("NODE");
        root_node.push_attribute(("Type", "0"));
        root_node.push_attribute(("Name", "ROOT"));
        writer.write_event(Event::Start(root_node))?;

        for pl in &playlists {
            let valid_ids: Vec<i64> = pl
                .track_ids
                .iter()
                .copied()
                .filter(|tid| track_id_map.contains_key(tid))
                .collect();
            let mut pl_node = BytesStart::new("NODE");
            let entries_str = valid_ids.len().to_string();
            pl_node.push_attribute(("Name", pl.name.as_str()));
            pl_node.push_attribute(("Type", "1"));
            pl_node.push_attribute(("KeyType", "0"));
            pl_node.push_attribute(("Entries", entries_str.as_str()));
            writer.write_event(Event::Start(pl_node))?;

            for tid in &valid_ids {
                let rb_id = track_id_map[tid];
                let mut trk = BytesStart::new("TRACK");
                let k_str = rb_id.to_string();
                trk.push_attribute(("Key", k_str.as_str()));
                writer.write_event(Event::Empty(trk))?;
            }
            writer.write_event(Event::End(BytesEnd::new("NODE")))?;
        }

        // Como no Python: a pasta existe se há histórico, mesmo sem faixas.
        if has_history {
            let mut hist_folder = BytesStart::new("NODE");
            hist_folder.push_attribute(("Name", "Mixxx History"));
            hist_folder.push_attribute(("Type", "0"));
            writer.write_event(Event::Start(hist_folder))?;

            for pl in &history_playlists {
                let valid_ids: Vec<i64> = pl
                    .track_ids
                    .iter()
                    .copied()
                    .filter(|tid| track_id_map.contains_key(tid))
                    .collect();
                if !valid_ids.is_empty() {
                    let mut pl_node = BytesStart::new("NODE");
                    let entries_str = valid_ids.len().to_string();
                    pl_node.push_attribute(("Name", pl.name.as_str()));
                    pl_node.push_attribute(("Type", "1"));
                    pl_node.push_attribute(("KeyType", "0"));
                    pl_node.push_attribute(("Entries", entries_str.as_str()));
                    writer.write_event(Event::Start(pl_node))?;

                    for tid in &valid_ids {
                        let rb_id = track_id_map[tid];
                        let mut trk = BytesStart::new("TRACK");
                        let k_str = rb_id.to_string();
                        trk.push_attribute(("Key", k_str.as_str()));
                        writer.write_event(Event::Empty(trk))?;
                    }
                    writer.write_event(Event::End(BytesEnd::new("NODE")))?;
                }
            }
            writer.write_event(Event::End(BytesEnd::new("NODE")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("NODE")))?; // ROOT
        writer.write_event(Event::End(BytesEnd::new("PLAYLISTS")))?;
        writer.write_event(Event::End(BytesEnd::new("DJ_PLAYLISTS")))?;
    }

    if let Some(parent) = output_xml.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_xml, xml_bytes)?;

    Ok((tracks.len(), playlists.len()))
}

#[derive(Debug, Default, Clone)]
pub struct XmlTrackData {
    pub key: String,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub bpm: f64,
    pub key_name: String,
    pub rating: i64,
    pub cues: Vec<XmlCueData>,
}

#[derive(Debug, Default, Clone)]
pub struct XmlCueData {
    pub name: String,
    pub cue_type: i64, // 0 = HotCue, 1 = Loop
    pub start: f64,
    pub end: Option<f64>,
    pub num: i64,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug, Default, Clone)]
pub struct XmlPlaylistData {
    pub name: String,
    pub track_keys: Vec<String>,
}

/// Lê um `<TRACK>` da COLLECTION (com ou sem filhos).
fn parse_collection_track(e: &BytesStart) -> Result<XmlTrackData, SyncError> {
    let mut track = XmlTrackData::default();
    for attr in e.attributes().flatten() {
        let val = attr_text(&attr)?;
        match attr.key.as_ref() {
            b"TrackID" => track.key = val,
            b"Name" => track.title = val,
            b"Artist" => track.artist = val,
            b"Album" => track.album = val,
            b"Genre" => track.genre = val,
            b"AverageBpm" => track.bpm = val.parse().unwrap_or(0.0),
            b"Key" => track.key_name = val,
            b"Rating" => track.rating = val.parse().unwrap_or(0),
            b"Location" => track.path = file_url_to_path(&val),
            _ => {}
        }
    }
    Ok(track)
}

fn parse_position_mark(e: &BytesStart) -> Result<XmlCueData, SyncError> {
    let mut cue = XmlCueData {
        num: -1,
        green: 122,
        blue: 255,
        ..Default::default()
    };
    for attr in e.attributes().flatten() {
        let val = attr_text(&attr)?;
        match attr.key.as_ref() {
            b"Name" => cue.name = val,
            b"Type" => cue.cue_type = val.parse().unwrap_or(0),
            b"Start" => cue.start = val.parse().unwrap_or(0.0),
            b"End" => cue.end = val.parse().ok(),
            b"Num" => cue.num = val.parse().unwrap_or(-1),
            b"Red" => cue.red = val.parse().unwrap_or(0),
            b"Green" => cue.green = val.parse().unwrap_or(122),
            b"Blue" => cue.blue = val.parse().unwrap_or(255),
            _ => {}
        }
    }
    Ok(cue)
}

#[derive(Debug, Clone)]
enum NodeKind {
    Folder(String),
    Playlist(String),
}

fn parse_node_kind(e: &BytesStart) -> Result<Option<NodeKind>, SyncError> {
    let mut node_type: Option<u8> = None;
    let mut name = "Rekordbox Playlist".to_string();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"Type" => {
                if attr.value.as_ref() == b"0" {
                    node_type = Some(0);
                } else if attr.value.as_ref() == b"1" {
                    node_type = Some(1);
                }
            }
            b"Name" => name = attr_text(&attr)?,
            _ => {}
        }
    }
    match node_type {
        Some(0) => Ok(Some(NodeKind::Folder(name))),
        Some(1) => Ok(Some(NodeKind::Playlist(name))),
        _ => Ok(None),
    }
}

/// Lê COLLECTION e PLAYLISTS do XML, na ordem do documento.
/// Preserva a hierarquia de pastas com nomes no formato `Pasta/Set`.
pub fn parse_rekordbox_xml(
    xml_content: &str,
) -> Result<(HashMap<String, XmlTrackData>, Vec<XmlPlaylistData>), SyncError> {
    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);

    let mut xml_tracks = HashMap::new();
    let mut playlists = Vec::new();
    let mut current_track: Option<XmlTrackData> = None;
    let mut current_playlist: Option<XmlPlaylistData> = None;
    let mut in_collection = false;

    let mut folder_stack: Vec<String> = Vec::new();
    let mut open_node_kinds: Vec<NodeKind> = Vec::new();

    loop {
        match reader.read_event()? {
            Event::Start(ref e) => match e.name().as_ref() {
                b"COLLECTION" => in_collection = true,
                b"TRACK" if in_collection => current_track = Some(parse_collection_track(e)?),
                b"POSITION_MARK" => {
                    if let Some(ref mut trk) = current_track {
                        trk.cues.push(parse_position_mark(e)?);
                    }
                }
                b"NODE" => {
                    if let Some(kind) = parse_node_kind(e)? {
                        match &kind {
                            NodeKind::Folder(fname) => {
                                if fname != "ROOT" {
                                    folder_stack.push(fname.clone());
                                }
                            }
                            NodeKind::Playlist(pname) => {
                                let full_name = if folder_stack.is_empty() {
                                    pname.clone()
                                } else {
                                    format!("{}/{}", folder_stack.join("/"), pname)
                                };
                                current_playlist = Some(XmlPlaylistData {
                                    name: full_name,
                                    track_keys: Vec::new(),
                                });
                            }
                        }
                        open_node_kinds.push(kind);
                    }
                }
                _ => {}
            },
            Event::Empty(ref e) => match e.name().as_ref() {
                b"TRACK" if in_collection => {
                    let trk = parse_collection_track(e)?;
                    xml_tracks.insert(trk.key.clone(), trk);
                }
                b"TRACK" => {
                    if let Some(ref mut pl) = current_playlist {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"Key" {
                                pl.track_keys.push(attr_text(&attr)?);
                            }
                        }
                    }
                }
                b"POSITION_MARK" => {
                    if let Some(ref mut trk) = current_track {
                        trk.cues.push(parse_position_mark(e)?);
                    }
                }
                b"NODE" => {
                    if let Some(NodeKind::Playlist(pname)) = parse_node_kind(e)? {
                        let full_name = if folder_stack.is_empty() {
                            pname
                        } else {
                            format!("{}/{}", folder_stack.join("/"), pname)
                        };
                        playlists.push(XmlPlaylistData {
                            name: full_name,
                            track_keys: Vec::new(),
                        });
                    }
                }
                _ => {}
            },
            Event::End(ref e) => match e.name().as_ref() {
                b"COLLECTION" => in_collection = false,
                b"TRACK" => {
                    if let Some(trk) = current_track.take() {
                        xml_tracks.insert(trk.key.clone(), trk);
                    }
                }
                b"NODE" => {
                    if let Some(kind) = open_node_kinds.pop() {
                        match kind {
                            NodeKind::Folder(fname) => {
                                if fname != "ROOT" {
                                    folder_stack.pop();
                                }
                            }
                            NodeKind::Playlist(_) => {
                                if let Some(pl) = current_playlist.take() {
                                    playlists.push(pl);
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }
    Ok((xml_tracks, playlists))
}

/// Importa dados do rekordbox.xml para o mixxxdb.sqlite com as garantias de `protected_import`.
pub fn import_rekordbox_xml_to_mixxx(
    xml_path: &Path,
    db_path: &Path,
) -> Result<(usize, usize), SyncError> {
    if !xml_path.is_file() {
        return Err(SyncError::Validation(format!(
            "Arquivo XML do Rekordbox não encontrado: {}",
            xml_path.display()
        )));
    }

    let (xml_tracks, playlists) = parse_rekordbox_xml(&fs::read_to_string(xml_path)?)?;
    protected_import(db_path, |tx| write_import(tx, &xml_tracks, &playlists))
}

fn write_import(
    tx: &Transaction,
    xml_tracks: &HashMap<String, XmlTrackData>,
    playlists: &[XmlPlaylistData],
) -> Result<(usize, usize), SyncError> {
    let mut cues_synced = 0;
    let mut track_key_to_mixxx_id = HashMap::new();

    // Sincroniza Cues
    for (key, data) in xml_tracks {
        let loc_id: Option<i64> = tx
            .query_row(
                "SELECT id FROM track_locations WHERE location = ?",
                params![data.path],
                |row| row.get(0),
            )
            .optional()?;
        let Some(lid) = loc_id else { continue };

        let lib_res: Option<(i64, Option<i64>)> = tx
            .query_row(
                "SELECT id, samplerate FROM library WHERE location = ?",
                params![lid],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let Some((mixxx_id, sr_opt)) = lib_res else {
            continue;
        };

        let samplerate = sr_opt.filter(|v| *v != 0).unwrap_or(44100) as f64;
        track_key_to_mixxx_id.insert(key.clone(), mixxx_id);

        for cue in &data.cues {
            // Posições fracionárias, como o Python grava.
            let sample_pos = cue.start * (samplerate * 2.0);
            let length_samples = match cue.end {
                Some(end) if end != 0.0 => (end - cue.start) * (samplerate * 2.0),
                _ => 0.0,
            };
            let color_int = rgb_to_int(cue.red, cue.green, cue.blue);
            let mixxx_cue_type = if cue.cue_type == 1 { 4 } else { 1 }; // 4 = Loop, 1 = HotCue

            let existing_id: Option<i64> = if cue.num >= 0 {
                tx.query_row(
                    "SELECT id FROM cues WHERE track_id = ? AND hotcue = ?",
                    params![mixxx_id, cue.num],
                    |row| row.get(0),
                )
                .optional()?
            } else {
                let tolerance = samplerate * 2.0 * 0.001;
                let mut stmt =
                    tx.prepare("SELECT id, position FROM cues WHERE track_id = ? AND hotcue = -1")?;
                let mut rows = stmt.query(params![mixxx_id])?;
                let mut matched = None;
                while let Some(r) = rows.next()? {
                    let cid: i64 = r.get(0)?;
                    let pos: f64 = r.get(1)?;
                    if (pos - sample_pos).abs() <= tolerance {
                        matched = Some(cid);
                        break;
                    }
                }
                matched
            };

            if let Some(cid) = existing_id {
                tx.execute(
                    "UPDATE cues
                     SET position = ?, length = ?, type = ?, label = ?, color = ?
                     WHERE id = ?",
                    params![
                        sample_pos,
                        length_samples,
                        mixxx_cue_type,
                        cue.name,
                        color_int,
                        cid
                    ],
                )?;
            } else {
                tx.execute(
                    "INSERT INTO cues (track_id, type, position, length, hotcue, label, color)
                     VALUES (?, ?, ?, ?, ?, ?, ?)",
                    params![
                        mixxx_id,
                        mixxx_cue_type,
                        sample_pos,
                        length_samples,
                        cue.num,
                        cue.name,
                        color_int
                    ],
                )?;
            }
            cues_synced += 1;
        }
    }

    // Sincroniza Playlists
    let mut playlists_synced = 0;
    for pl in playlists {
        let existing: Option<i64> = tx
            .query_row(
                "SELECT id FROM Playlists WHERE name = ?",
                params![pl.name],
                |row| row.get(0),
            )
            .optional()?;
        let pl_id = match existing {
            Some(id) => id,
            None => {
                tx.execute(
                    "INSERT INTO Playlists (name, position, hidden) VALUES (?, 0, 0)",
                    params![pl.name],
                )?;
                tx.last_insert_rowid()
            }
        };

        tx.execute(
            "DELETE FROM PlaylistTracks WHERE playlist_id = ?",
            params![pl_id],
        )?;

        let mut pos = 1;
        for tkey in &pl.track_keys {
            if let Some(mid) = track_key_to_mixxx_id.get(tkey) {
                tx.execute(
                    "INSERT INTO PlaylistTracks (playlist_id, track_id, position)
                     VALUES (?, ?, ?)",
                    params![pl_id, mid, pos],
                )?;
                pos += 1;
            }
        }
        playlists_synced += 1;
    }

    Ok((cues_synced, playlists_synced))
}

#[derive(Debug, Default)]
pub struct SyncStatus {
    pub db_exists: bool,
    pub db_path: PathBuf,
    pub track_count: i64,
    pub playlist_count: i64,
    pub hotcue_count: i64,
    pub xml_exists: bool,
    pub xml_path: PathBuf,
    pub xml_size_kb: f64,
    pub xml_mtime: String,
    pub xml_entries: usize,
}

/// Obtém status atual das contagens do Mixxx DB e do XML.
pub fn get_status(db_path: &Path, xml_path: &Path) -> SyncStatus {
    let mut status = SyncStatus {
        db_path: db_path.to_path_buf(),
        xml_path: xml_path.to_path_buf(),
        ..Default::default()
    };

    if db_path.is_file() {
        if let Ok(conn) = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            status.db_exists = true;
            status.track_count = conn
                .query_row("SELECT count(*) FROM library", [], |row| row.get(0))
                .unwrap_or(0);
            status.playlist_count = conn
                .query_row(
                    "SELECT count(*) FROM Playlists WHERE hidden = 0",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            status.hotcue_count = conn
                .query_row("SELECT count(*) FROM cues WHERE hotcue >= 0", [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);
        }
    }

    if xml_path.is_file() {
        status.xml_exists = true;
        if let Ok(meta) = fs::metadata(xml_path) {
            status.xml_size_kb = meta.len() as f64 / 1024.0;
            if let Ok(modified) = meta.modified() {
                let dt: chrono::DateTime<Local> = modified.into();
                status.xml_mtime = dt.format("%Y-%m-%d %H:%M:%S").to_string();
            }
        }
        if let Ok(content) = fs::read_to_string(xml_path) {
            let mut reader = Reader::from_str(&content);
            let mut buf = Vec::new();
            while let Ok(event) = reader.read_event_into(&mut buf) {
                if let Event::Start(ref e) = event {
                    if e.name().as_ref() == b"COLLECTION" {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"Entries" {
                                let s = String::from_utf8_lossy(&attr.value);
                                status.xml_entries = s.parse().unwrap_or(0);
                            }
                        }
                        break;
                    }
                } else if let Event::Eof = event {
                    break;
                }
                buf.clear();
            }
        }
    }

    status
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db_with_value(dir: &Path) -> PathBuf {
        let db = dir.join("mixxxdb.sqlite");
        let conn = Connection::open(&db).unwrap();
        conn.execute_batch("CREATE TABLE test (value TEXT); INSERT INTO test VALUES ('original');")
            .unwrap();
        db
    }

    #[test]
    fn running_process_refuses_import() {
        let dir = tempfile::tempdir().unwrap();
        let db = db_with_value(dir.path());
        let running = || -> Result<(), SyncError> { Err(SyncError::Validation("running".into())) };
        let result = protected_import_checked(&db, &running, |_| -> Result<(), SyncError> {
            panic!("must not write")
        });
        assert!(result.is_err());
        assert!(!dir.path().join("backups-rekordbox").exists());
    }

    #[test]
    fn python_compatible_conversions() {
        assert_eq!(abspath("/a/./b/../c"), "/a/c");
        assert_eq!(
            path_to_file_url("/music/AC&DC 100% (live)/faixa ç.mp3"),
            "file://localhost/music/AC%26DC%20100%25%20%28live%29/faixa%20%C3%A7.mp3"
        );
        assert_eq!(
            file_url_to_path(
                "file://localhost/music/AC%26DC%20100%25%20%28live%29/faixa%20%C3%A7.mp3"
            ),
            "/music/AC&DC 100% (live)/faixa ç.mp3"
        );
        assert_eq!(
            path_to_file_url(r"C:\Música\a b\..\x.mp3"),
            "file://localhost/C:/M%C3%BAsica/x.mp3"
        );
        assert_eq!(path_to_file_url("C:/../x.mp3"), "file://localhost/C:/x.mp3");
        assert_eq!(
            file_url_to_path("file://localhost/C:/M%C3%BAsica/x.mp3"),
            "C:/Música/x.mp3"
        );
        assert_eq!(round_half_even(180.5), 180);
        assert_eq!(round_half_even(181.5), 182);
        assert_eq!(round_half_even(180.6), 181);
    }

    #[test]
    fn parser_reads_empty_tracks_entities_and_empty_playlists() {
        let xml = r#"<?xml version="1.0"?><DJ_PLAYLISTS><COLLECTION Entries="2">
            <TRACK TrackID="1" Name="Rock &amp; Roll" Location="file://localhost/m/a.mp3"/>
            <TRACK TrackID="2" Name="B" Location="file://localhost/m/b.mp3">
              <POSITION_MARK Name="Drop" Type="0" Start="1.5" Num="0"/>
            </TRACK></COLLECTION>
            <PLAYLISTS><NODE Type="0" Name="ROOT">
              <NODE Name="Vazia" Type="1" Entries="0"/>
              <NODE Name="Set" Type="1"><TRACK Key="2"/><TRACK Key="1"/></NODE>
            </NODE></PLAYLISTS></DJ_PLAYLISTS>"#;
        let (tracks, playlists) = parse_rekordbox_xml(xml).unwrap();
        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks["1"].title, "Rock & Roll");
        assert!(tracks["1"].cues.is_empty());
        assert_eq!(tracks["2"].cues.len(), 1);
        assert_eq!(playlists.len(), 2);
        assert_eq!(playlists[0].name, "Vazia");
        assert!(playlists[0].track_keys.is_empty());
        assert_eq!(playlists[1].track_keys, vec!["2", "1"]);
    }
}
