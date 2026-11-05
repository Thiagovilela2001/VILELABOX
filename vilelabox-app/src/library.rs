use std::fs;
use std::path::{Path, PathBuf};
use vilelabox_anlz::{parse_ppth, parse_pqtz, parse_pssi, sections, Beat, Grid, RawCue, RawPhrase};

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub title: String,
    pub dat: Option<PathBuf>,
    pub ext: Option<PathBuf>,
    pub grid: Option<Grid>,
    /// Caminho do áudio (PPTH do ANLZ ou Location do XML).
    pub path: Option<String>,
    /// Metadados do documento XML, quando a coleção vem dele.
    pub meta: Option<Meta>,
    /// Duração medida no cabeçalho do áudio (`.wav`), quando acessível no scan.
    pub audio_ms: Option<u64>,
}

/// Metadados de uma faixa vindos do XML (12-xml).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Meta {
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub tonality: String,
    /// 0..5 estrelas.
    pub stars: u8,
    pub bpm: f64,
    pub total_s: u32,
    pub cues: Vec<RawCue>,
}

impl Track {
    /// (intérprete, título): do XML ou, sem ele, do nome do arquivo.
    pub fn artist_title(&self) -> (String, String) {
        match &self.meta {
            Some(m) => (m.artist.clone(), self.title.clone()),
            None => crate::state::split_artist_title(&self.title),
        }
    }

    pub fn bpm(&self) -> f64 {
        match &self.meta {
            Some(m) if m.bpm > 0.0 => m.bpm,
            _ => self.grid.as_ref().map_or(0.0, |g| g.bpm),
        }
    }

    pub fn dur_ms(&self) -> u64 {
        match &self.meta {
            Some(m) if m.total_s > 0 => u64::from(m.total_s) * 1000,
            _ => self
                .audio_ms
                .unwrap_or_else(|| crate::state::duration_ms(self.grid.as_ref())),
        }
    }
}

/// Lista de reprodução achatada: nome com o caminho das pastas e índices das
/// faixas na coleção, na ordem da lista.
#[derive(Debug, Clone, PartialEq)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<usize>,
}

/// Chave de casamento XML ↔ análise: nome do arquivo em minúsculas. O `PPTH`
/// das análises locais guarda só `?/<nome do arquivo>` (observado), não a pasta.
fn chave_arquivo(p: &str) -> String {
    p.rsplit(['/', '\\']).next().unwrap_or(p).to_lowercase()
}

/// Batidas a partir dos trechos `TEMPO` (cada um vale até o próximo).
fn beats_de_tempos(tempos: &[vilelabox_xml::Tempo], total_s: f64) -> Vec<Beat> {
    let mut out = Vec::new();
    for (i, t) in tempos.iter().enumerate() {
        if t.bpm <= 0.0 {
            continue;
        }
        let fim = tempos.get(i + 1).map_or(total_s, |p| p.inizio);
        let passo = 60.0 / t.bpm;
        let mut n = u16::from(t.battito.clamp(1, 4));
        let mut s = t.inizio;
        while s < fim && out.len() < 100_000 {
            out.push(Beat {
                beat_number: n,
                bpm: t.bpm,
                time_ms: (s * 1000.0).round() as u32,
            });
            n = n % 4 + 1;
            s += passo;
        }
    }
    out
}

fn cue_de_marca(m: &vilelabox_xml::Mark) -> RawCue {
    RawCue {
        hot_cue: u32::try_from(m.num + 1).unwrap_or(0),
        status: 0,
        is_loop: m.kind == 4,
        start_time_s: m.start,
        end_time_s: m.end,
        label: Some(m.name.clone()).filter(|n| !n.is_empty()),
        color: m
            .rgb
            .map(|(r, g, b)| (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)),
    }
}

fn achatar(
    n: &vilelabox_xml::Node,
    prefixo: &str,
    ids: &std::collections::HashMap<String, usize>,
    out: &mut Vec<Playlist>,
) {
    match n {
        vilelabox_xml::Node::Folder { name, children } => {
            let p = if name == "ROOT" || prefixo.is_empty() && name.is_empty() {
                String::new()
            } else {
                format!("{prefixo}{name} / ")
            };
            for c in children {
                achatar(c, &p, ids, out);
            }
        }
        vilelabox_xml::Node::Playlist {
            name,
            key_type,
            keys,
        } => out.push(Playlist {
            name: format!("{prefixo}{name}"),
            // KeyType 1 (Location) não ocorre na exportação real; só TrackID.
            tracks: keys
                .iter()
                .filter(|_| *key_type == 0)
                .filter_map(|k| ids.get(k).copied())
                .collect(),
        }),
    }
}

/// Coleção a partir de um documento XML. Grade e formas de onda vêm das
/// análises em `anlz_dir`, casadas pelo caminho do áudio (`PPTH` = `Location`);
/// sem análise, a grade é montada dos `TEMPO` do XML.
pub fn load_xml(
    xml: &str,
    anlz_dir: &Path,
) -> Result<(Vec<Track>, Vec<Playlist>), vilelabox_xml::XmlError> {
    let doc = vilelabox_xml::parse(xml)?;
    let mut analises: std::collections::HashMap<String, Vec<Track>> =
        std::collections::HashMap::new();
    for t in scan(anlz_dir) {
        if let Some(p) = t.path.as_deref() {
            analises.entry(chave_arquivo(p)).or_default().push(t);
        }
    }
    let mut ids = std::collections::HashMap::new();
    let tracks: Vec<Track> = doc
        .tracks
        .iter()
        .enumerate()
        .map(|(i, x)| {
            ids.insert(x.track_id.clone(), i);
            let anlz = x
                .location
                .as_deref()
                .and_then(|l| analises.get(&chave_arquivo(l)))
                .and_then(|c| {
                    // mesmo nome em pastas diferentes: desempata pela duração (±2 s)
                    let perto = |a: &&Track| {
                        let ms = crate::state::duration_ms(a.grid.as_ref()) as i64;
                        (ms - i64::from(x.total_time) * 1000).abs() <= 2000
                    };
                    if c.len() == 1 {
                        c.first()
                    } else {
                        c.iter().find(perto)
                    }
                });
            let grid = anlz.and_then(|a| a.grid.clone()).or_else(|| {
                let beats = beats_de_tempos(&x.tempos, f64::from(x.total_time));
                let primeira = beats.first()?.time_ms;
                Some(Grid {
                    bpm: x.tempos.first()?.bpm,
                    variable: x.tempos.windows(2).any(|w| w[0].bpm != w[1].bpm),
                    first_beat_time_s: f64::from(primeira) / 1000.0,
                    downbeat_index: 0,
                    beats,
                })
            });
            Track {
                title: x.name.clone(),
                audio_ms: anlz.and_then(|a| a.audio_ms),
                dat: anlz.and_then(|a| a.dat.clone()),
                ext: anlz.and_then(|a| a.ext.clone()),
                grid,
                path: x.location.clone(),
                meta: Some(Meta {
                    artist: x.artist.clone(),
                    album: x.album.clone(),
                    genre: x.genre.clone(),
                    tonality: x.tonality.clone(),
                    stars: ((u32::from(x.rating) + 25) / 51).min(5) as u8,
                    bpm: x.average_bpm,
                    total_s: x.total_time,
                    cues: x.marks.iter().map(cue_de_marca).collect(),
                }),
            }
        })
        .collect();
    let mut playlists = Vec::new();
    achatar(&doc.playlists, "", &ids, &mut playlists);
    Ok((tracks, playlists))
}

/// Coleção a partir da cópia sem criptografia do catálogo (10-master-db,
/// `vilelabox-catalog`). O caminho do áudio vem completo (`FolderPath`); a
/// análise é o `.DAT` apontado pela faixa, relativo à pasta `share`, que é o
/// avô de `anlz_dir` (`share/PIONEER/USBANLZ`).
pub fn load_catalogo(
    db: &Path,
    anlz_dir: &Path,
) -> Result<(Vec<Track>, Vec<Playlist>), vilelabox_catalog::ErroCatalogo> {
    let cat = vilelabox_catalog::abrir(db)?;
    let share = anlz_dir.parent().and_then(Path::parent).unwrap_or(anlz_dir);
    let chave = |p: &Path| p.to_string_lossy().replace('\\', "/").to_lowercase();
    let mut analises: std::collections::HashMap<String, Track> = scan(anlz_dir)
        .into_iter()
        .filter_map(|t| Some((chave(t.dat.as_deref()?), t)))
        .collect();
    let mut ids = std::collections::HashMap::new();
    let tracks: Vec<Track> = cat
        .faixas
        .into_iter()
        .enumerate()
        .map(|(i, f)| {
            ids.insert(f.id.clone(), i);
            let anlz = f
                .analise
                .as_deref()
                .and_then(|a| analises.remove(&chave(&share.join(a.trim_start_matches('/')))));
            Track {
                title: f.titulo,
                audio_ms: anlz.as_ref().and_then(|a| a.audio_ms),
                dat: anlz.as_ref().and_then(|a| a.dat.clone()),
                ext: anlz.as_ref().and_then(|a| a.ext.clone()),
                grid: anlz.and_then(|a| a.grid),
                path: Some(f.caminho).filter(|c| !c.is_empty()),
                meta: Some(Meta {
                    artist: f.artista,
                    album: f.album,
                    genre: f.genero,
                    tonality: f.tom,
                    stars: f.estrelas,
                    bpm: f.bpm,
                    total_s: f.duracao_s,
                    cues: f
                        .cues
                        .into_iter()
                        .map(|c| RawCue {
                            hot_cue: c.tipo,
                            status: 1,
                            is_loop: c.saida_ms.is_some(),
                            start_time_s: c.entrada_ms as f64 / 1000.0,
                            end_time_s: c.saida_ms.map(|s| s as f64 / 1000.0),
                            label: c.rotulo,
                            // `djmdCue.Color` é um código, não RGB; sem
                            // mapeamento observado, fica sem cor própria.
                            color: None,
                        })
                        .collect(),
                }),
            }
        })
        .collect();
    let playlists = cat
        .listas
        .into_iter()
        .map(|l| Playlist {
            name: l.nome,
            tracks: l
                .faixas
                .iter()
                .filter_map(|id| ids.get(id).copied())
                .collect(),
        })
        .collect();
    Ok((tracks, playlists))
}

pub struct PhraseSpan {
    pub start: f32,
    pub end: f32,
    pub kind: String,
}

pub struct Timeline {
    pub info: String,
    /// Posição relativa (0..1) de cada compasso.
    pub bars: Vec<f32>,
    pub phrases: Vec<PhraseSpan>,
}

fn dat_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            dat_files(&path, out);
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("dat"))
        {
            out.push(path);
        }
    }
}

fn title_from_path(raw: &str) -> String {
    raw.rsplit(['/', '\\']).next().unwrap_or(raw).to_string()
}

/// Lê todas as análises `.DAT` (e o `.EXT` irmão, se houver) sob `dir`.
/// Arquivos ilegíveis ou sem caminho de áudio entram com o nome do arquivo.
pub fn scan(dir: &Path) -> Vec<Track> {
    let mut files = Vec::new();
    dat_files(dir, &mut files);
    let mut tracks: Vec<Track> = files
        .into_iter()
        .filter_map(|dat_path| {
            let dat = fs::read(&dat_path).ok()?;
            let ext_path = dat_path.with_extension("EXT");
            let ext = fs::read(&ext_path).ok();
            let dat_secs = sections(&dat);
            let ext_secs = ext.as_deref().map(sections).unwrap_or_default();
            let ppth = ext_secs
                .get(b"PPTH")
                .or(dat_secs.get(b"PPTH"))
                .and_then(|p| parse_ppth(p));
            let title = ppth
                .as_deref()
                .map_or_else(|| dat_path.display().to_string(), title_from_path);
            let grid = dat_secs.get(b"PQTZ").and_then(|p| parse_pqtz(p));
            // A grade e a forma de onda do rekordbox seguem o tamanho declarado
            // no WAV; com `0xFFFFFFFF` elas passam de 6 h. O cabeçalho
            // limitado ao arquivo dá a duração real. O PPTH pode vir sem pasta
            // (`?/nome.wav`): o arquivo é localizado como no carregamento do deck.
            let audio_ms = ppth
                .as_deref()
                .filter(|p| p.to_lowercase().ends_with(".wav"))
                .and_then(|p| crate::find_audio_file(Some(p), &title))
                .and_then(|p| vilelabox_audio::decode::duracao_wav(&p))
                .map(|s| (s * 1000.0) as u64);
            Some(Track {
                path: ppth.clone(),
                meta: None,
                audio_ms,
                title,
                dat: Some(dat_path.clone()),
                ext: ext.is_some().then_some(ext_path),
                grid,
            })
        })
        .collect();
    tracks.sort_by_key(|t| t.title.to_lowercase());
    tracks
}

fn canonical_kind(kind: &str) -> String {
    if kind.starts_with("verse") {
        "VERSE".into()
    } else {
        kind.to_uppercase()
    }
}

/// Monta a linha do tempo: compassos e frases em posição relativa à duração
/// coberta pela grade (última batida + uma batida).
pub fn build_timeline(grid: &Grid, phrases: &[RawPhrase]) -> (Vec<f32>, Vec<PhraseSpan>) {
    let Some(last) = grid.beats.last() else {
        return (Vec::new(), Vec::new());
    };
    let beat_ms = if grid.bpm > 0.0 {
        60_000.0 / grid.bpm
    } else {
        0.0
    };
    let total_ms = last.time_ms as f64 + beat_ms;
    if total_ms <= 0.0 {
        return (Vec::new(), Vec::new());
    }
    let rel = |ms: f64| (ms / total_ms).clamp(0.0, 1.0) as f32;

    let bars = grid
        .beats
        .iter()
        .filter(|b| b.beat_number == 1)
        .map(|b| rel(b.time_ms as f64))
        .collect();

    let starts: Vec<(f32, String)> = phrases
        .iter()
        .filter_map(|p| Some((rel(p.start_time_s? * 1000.0), canonical_kind(&p.kind))))
        .collect();
    let spans = starts
        .iter()
        .enumerate()
        .map(|(i, (start, kind))| PhraseSpan {
            start: *start,
            end: starts.get(i + 1).map_or(1.0, |n| n.0),
            kind: kind.clone(),
        })
        .collect();
    (bars, spans)
}

pub fn timeline(track: &Track) -> Timeline {
    let Some(grid) = &track.grid else {
        return Timeline {
            info: "Sem grade de batidas nesta análise.".into(),
            bars: Vec::new(),
            phrases: Vec::new(),
        };
    };
    let beat_times: Vec<u32> = grid.beats.iter().map(|b| b.time_ms).collect();
    let raw = track
        .ext
        .as_ref()
        .and_then(|p| fs::read(p).ok())
        .map(|ext| {
            sections(&ext)
                .get(b"PSSI")
                .map(|p| parse_pssi(p, &beat_times))
                .unwrap_or_default()
        })
        .unwrap_or_default();
    let (bars, phrases) = build_timeline(grid, &raw);
    let tipo = if grid.variable { "variável" } else { "fixo" };
    Timeline {
        info: format!(
            "{:.2} BPM ({tipo})  ·  {} batidas  ·  {} compassos  ·  {} frases  ·  1ª batida {:.3} s",
            grid.bpm,
            grid.beats.len(),
            bars.len(),
            phrases.len(),
            grid.first_beat_time_s
        ),
        bars,
        phrases,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vilelabox_anlz::Beat;

    #[test]
    fn xml_vira_colecao_com_grade_cues_e_listas() {
        let xml = r#"<DJ_PLAYLISTS Version="1.0.0"><PRODUCT Name="rekordbox" Version="7.2.8" Company="AlphaTheta"/>
<COLLECTION Entries="2">
 <TRACK TrackID="7" Name="Faixa" Artist="DJ" Album="Al" Genre="Funk" AverageBpm="120.00" TotalTime="4" Rating="153" Tonality="Am" Location="file://localhost/C:/sem/analise.mp3">
  <TEMPO Inizio="0.500" Bpm="120.00" Metro="4/4" Battito="3"/>
  <POSITION_MARK Name="" Type="0" Start="1.000" Num="0" Red="255" Green="0" Blue="0"/>
  <POSITION_MARK Name="" Type="4" Start="2.000" End="3.000" Num="-1"/>
 </TRACK>
 <TRACK TrackID="9" Name="Outra" Location="file://localhost/C:/b.mp3"/>
</COLLECTION>
<PLAYLISTS><NODE Type="0" Name="ROOT" Count="1"><NODE Type="0" Name="Sets" Count="1">
 <NODE Name="Abertura" Type="1" KeyType="0" Entries="3"><TRACK Key="9"/><TRACK Key="7"/><TRACK Key="404"/></NODE>
</NODE></NODE></PLAYLISTS></DJ_PLAYLISTS>"#;
        let (tracks, listas) = load_xml(xml, Path::new("nao/existe")).unwrap();
        let t = &tracks[0];
        let m = t.meta.as_ref().unwrap();
        assert_eq!(
            (m.artist.as_str(), m.tonality.as_str(), m.stars),
            ("DJ", "Am", 3)
        );
        assert_eq!((t.bpm(), t.dur_ms()), (120.0, 4000));
        // grade dos TEMPO: 0.5 s na batida 3, depois 4, 1, 2...
        let g = t.grid.as_ref().unwrap();
        let b: Vec<(u16, u32)> = g.beats.iter().map(|b| (b.beat_number, b.time_ms)).collect();
        assert_eq!(&b[..3], &[(3, 500), (4, 1000), (1, 1500)]);
        // Num 0 = hot cue A (hot_cue 1); Num -1 = memória
        assert_eq!((m.cues[0].hot_cue, m.cues[0].color), (1, Some(0xFF0000)));
        assert_eq!(
            (m.cues[1].hot_cue, m.cues[1].is_loop, m.cues[1].end_time_s),
            (0, true, Some(3.0))
        );
        // lista na ordem dela, chave inexistente descartada
        assert_eq!(
            listas,
            vec![Playlist {
                name: "Sets / Abertura".into(),
                tracks: vec![1, 0]
            }]
        );
    }

    fn grid(beats: &[(u16, u32)], bpm: f64) -> Grid {
        Grid {
            bpm,
            variable: false,
            first_beat_time_s: beats[0].1 as f64 / 1000.0,
            downbeat_index: 0,
            beats: beats
                .iter()
                .map(|&(n, t)| Beat {
                    beat_number: n,
                    bpm,
                    time_ms: t,
                })
                .collect(),
        }
    }

    fn phrase(beat: u16, kind: &str, start: Option<f64>) -> RawPhrase {
        RawPhrase {
            index: 0,
            beat,
            kind: kind.into(),
            kind_id: 0,
            start_time_s: start,
        }
    }

    #[test]
    fn compassos_e_frases_relativos() {
        // 120 BPM, 8 batidas a cada 500 ms: duração 3500 + 500 = 4000 ms
        let g = grid(
            &[
                (1, 0),
                (2, 500),
                (3, 1000),
                (4, 1500),
                (1, 2000),
                (2, 2500),
                (3, 3000),
                (4, 3500),
            ],
            120.0,
        );
        let p = [
            phrase(1, "intro", Some(0.0)),
            phrase(5, "verse_2", Some(2.0)),
            phrase(99, "outro", None),
        ];
        let (bars, spans) = build_timeline(&g, &p);
        assert_eq!(bars, vec![0.0, 0.5]);
        assert_eq!(spans.len(), 2);
        assert_eq!(
            (spans[0].start, spans[0].end, spans[0].kind.as_str()),
            (0.0, 0.5, "INTRO")
        );
        assert_eq!(
            (spans[1].start, spans[1].end, spans[1].kind.as_str()),
            (0.5, 1.0, "VERSE")
        );
    }

    #[test]
    fn titulo_pelo_nome_do_arquivo() {
        assert_eq!(title_from_path("C:\\Music\\Faixa A.mp3"), "Faixa A.mp3");
        assert_eq!(title_from_path("/music/b.flac"), "b.flac");
    }
}
