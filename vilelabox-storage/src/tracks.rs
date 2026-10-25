//! Faixas, metadados e localizações de arquivo.

use crate::error::{mapear_constraint, ErroStorage};
use rusqlite::{params, Connection, OptionalExtension};
use vilelabox_types::{AudioIdentity, Bpm, Frame, SampleRate, TrackId, TrackMetadata};

/// Dados de uma faixa ainda sem `id`; a identidade acústica (`sha256`,
/// duração, taxa, canais) fica vazia até o áudio ser lido.
#[derive(Debug, Clone, Default)]
pub struct NovaFaixa {
    pub metadados: TrackMetadata,
    pub identidade: Option<AudioIdentity>,
    pub bpm: Option<Bpm>,
    pub tom: Option<String>,
    /// 0..=5.
    pub estrelas: u8,
}

/// Faixa persistida.
#[derive(Debug, Clone, PartialEq)]
pub struct FaixaRow {
    pub id: TrackId,
    pub metadados: TrackMetadata,
    pub identidade: Option<AudioIdentity>,
    pub bpm: Option<Bpm>,
    pub tom: Option<String>,
    pub estrelas: u8,
}

/// Uma localização conhecida do arquivo de áudio da faixa.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalizacaoRow {
    pub id: i64,
    pub caminho: String,
    /// Arquivo ausente ou movido na última verificação.
    pub ausente: bool,
}

#[allow(clippy::type_complexity)]
type FaixaCrua = (
    i64,
    TrackMetadata,
    Option<String>,
    Option<i64>,
    Option<u32>,
    Option<u16>,
    Option<f64>,
    Option<String>,
    u8,
);

fn ler_faixa(r: &rusqlite::Row) -> rusqlite::Result<FaixaCrua> {
    Ok((
        r.get(0)?,
        TrackMetadata {
            title: r.get(1)?,
            artist: r.get(2)?,
            album: r.get(3)?,
            genre: r.get(4)?,
            year: r.get(5)?,
            comment: r.get(6)?,
        },
        r.get(7)?,
        r.get(8)?,
        r.get(9)?,
        r.get(10)?,
        r.get(11)?,
        r.get(12)?,
        r.get(13)?,
    ))
}

fn montar_faixa(crua: FaixaCrua) -> Result<FaixaRow, ErroStorage> {
    let (id, metadados, sha256, duracao, taxa, canais, bpm, tom, estrelas) = crua;
    let identidade = match taxa {
        Some(hz) => Some(AudioIdentity {
            sha256,
            duration_frames: Frame::new(duracao.unwrap_or(0)),
            sample_rate: SampleRate::new(hz)?,
            channels: canais.unwrap_or(0),
        }),
        None => None,
    };
    let bpm = bpm.map(Bpm::new).transpose()?;
    Ok(FaixaRow {
        id: TrackId(id),
        metadados,
        identidade,
        bpm,
        tom,
        estrelas,
    })
}

const COLUNAS: &str = "id, title, artist, album, genre, year, comment, sha256,
    duration_frames, sample_rate, channels, bpm, key_text, rating";

pub fn inserir(con: &Connection, faixa: &NovaFaixa) -> Result<TrackId, ErroStorage> {
    if faixa.estrelas > 5 {
        return Err(ErroStorage::Validacao(format!(
            "estrelas fora de 0..=5: {}",
            faixa.estrelas
        )));
    }
    let (sha256, duracao, taxa, canais) = match &faixa.identidade {
        Some(i) => (
            i.sha256.clone(),
            Some(i.duration_frames.get()),
            Some(i.sample_rate.get()),
            Some(i.channels),
        ),
        None => (None, None, None, None),
    };
    con.execute(
        "INSERT INTO tracks (sha256, title, artist, album, genre, year, comment,
            duration_frames, sample_rate, channels, bpm, key_text, rating)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            sha256,
            faixa.metadados.title,
            faixa.metadados.artist,
            faixa.metadados.album,
            faixa.metadados.genre,
            faixa.metadados.year,
            faixa.metadados.comment,
            duracao,
            taxa,
            canais,
            faixa.bpm.map(|b| b.get()),
            faixa.tom,
            faixa.estrelas,
        ],
    )
    .map_err(mapear_constraint)?;
    Ok(TrackId(con.last_insert_rowid()))
}
/// Localização com a faixa dona (varredura de ausentes e movidos).
#[derive(Debug, Clone, PartialEq)]
pub struct LocalizacaoCompleta {
    pub id: i64,
    pub faixa: TrackId,
    pub caminho: String,
    pub ausente: bool,
}

/// Todas as faixas do banco (`None`) ou as de uma playlist na ordem dela.
pub fn listar(con: &Connection, lista: Option<i64>) -> Result<Vec<FaixaRow>, ErroStorage> {
    let linhas = match lista {
        Some(l) => {
            let sql = format!(
                "SELECT {COLUNAS} FROM tracks t
                 JOIN playlist_tracks p ON p.track_id = t.id
                 WHERE p.playlist_id = ?1 ORDER BY p.track_no"
            );
            let mut st = con.prepare(&sql)?;
            let x = st
                .query_map(params![l], ler_faixa)?
                .collect::<Result<Vec<_>, _>>()?;
            x
        }
        None => {
            let sql = format!("SELECT {COLUNAS} FROM tracks ORDER BY id");
            let mut st = con.prepare(&sql)?;
            let x = st
                .query_map([], ler_faixa)?
                .collect::<Result<Vec<_>, _>>()?;
            x
        }
    };
    linhas.into_iter().map(montar_faixa).collect()
}

/// Atualiza metadados, BPM, tom e estrelas, tocando `updated_at`; a
/// identidade acústica não é alterada (use [`atualizar_identidade`]).
pub fn atualizar(con: &Connection, id: TrackId, faixa: &NovaFaixa) -> Result<bool, ErroStorage> {
    if faixa.estrelas > 5 {
        return Err(ErroStorage::Validacao(format!(
            "estrelas fora de 0..=5: {}",
            faixa.estrelas
        )));
    }
    let n = con
        .execute(
            "UPDATE tracks SET title = ?2, artist = ?3, album = ?4, genre = ?5, year = ?6,
                comment = ?7, bpm = ?8, key_text = ?9, rating = ?10,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?1",
            params![
                id.0,
                faixa.metadados.title,
                faixa.metadados.artist,
                faixa.metadados.album,
                faixa.metadados.genre,
                faixa.metadados.year,
                faixa.metadados.comment,
                faixa.bpm.map(|b| b.get()),
                faixa.tom,
                faixa.estrelas,
            ],
        )
        .map_err(mapear_constraint)?;
    Ok(n > 0)
}

/// Grava a identidade acústica (hash, duração, taxa, canais) da faixa.
pub fn atualizar_identidade(
    con: &Connection,
    id: TrackId,
    identidade: &AudioIdentity,
) -> Result<bool, ErroStorage> {
    let n = con.execute(
        "UPDATE tracks SET sha256 = ?2, duration_frames = ?3, sample_rate = ?4, channels = ?5,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?1",
        params![
            id.0,
            identidade.sha256,
            identidade.duration_frames.get(),
            identidade.sample_rate.get(),
            identidade.channels,
        ],
    )?;
    Ok(n > 0)
}

/// Faixa dona de uma localização pelo caminho exato.
pub fn id_por_localizacao(con: &Connection, caminho: &str) -> Result<Option<TrackId>, ErroStorage> {
    Ok(con
        .query_row(
            "SELECT track_id FROM track_locations WHERE path = ?1",
            params![caminho],
            |r| r.get::<_, i64>(0),
        )
        .optional()?
        .map(TrackId))
}

/// Todas as localizações com as faixas donas.
pub fn listar_localizacoes(con: &Connection) -> Result<Vec<LocalizacaoCompleta>, ErroStorage> {
    let mut st =
        con.prepare("SELECT id, track_id, path, missing FROM track_locations ORDER BY id")?;
    let rows = st.query_map([], |r| {
        Ok(LocalizacaoCompleta {
            id: r.get(0)?,
            faixa: TrackId(r.get(1)?),
            caminho: r.get(2)?,
            ausente: r.get::<_, i64>(3)? != 0,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Marca uma localização como ausente ou presente.
pub fn marcar_localizacao(con: &Connection, id: i64, ausente: bool) -> Result<(), ErroStorage> {
    con.execute(
        "UPDATE track_locations SET missing = ?2 WHERE id = ?1",
        params![id, ausente],
    )?;
    Ok(())
}

pub fn obter(con: &Connection, id: TrackId) -> Result<Option<FaixaRow>, ErroStorage> {
    let sql = format!("SELECT {COLUNAS} FROM tracks WHERE id = ?1");
    let crua = con.query_row(&sql, params![id.0], ler_faixa).optional()?;
    crua.map(montar_faixa).transpose()
}

/// Remove a faixa; cues, grade, frases, localizações e vínculos de playlist
/// caem em cascata. Retorna se a faixa existia.
pub fn remover(con: &Connection, id: TrackId) -> Result<bool, ErroStorage> {
    let n = con.execute("DELETE FROM tracks WHERE id = ?1", params![id.0])?;
    Ok(n > 0)
}

pub fn adicionar_localizacao(
    con: &Connection,
    faixa: TrackId,
    caminho: &str,
) -> Result<i64, ErroStorage> {
    con.execute(
        "INSERT INTO track_locations (track_id, path) VALUES (?1, ?2)",
        params![faixa.0, caminho],
    )
    .map_err(mapear_constraint)?;
    Ok(con.last_insert_rowid())
}

pub fn localizacoes(con: &Connection, faixa: TrackId) -> Result<Vec<LocalizacaoRow>, ErroStorage> {
    let mut st = con
        .prepare("SELECT id, path, missing FROM track_locations WHERE track_id = ?1 ORDER BY id")?;
    let rows = st.query_map(params![faixa.0], |r| {
        Ok(LocalizacaoRow {
            id: r.get(0)?,
            caminho: r.get(1)?,
            ausente: r.get::<_, i64>(2)? != 0,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;

    #[test]
    fn faixa_sem_identidade_le_e_escreve() {
        let db = Database::open_in_memory().unwrap();
        let id = db
            .inserir_faixa(&NovaFaixa {
                metadados: TrackMetadata {
                    title: Some("Faixa".into()),
                    ..TrackMetadata::default()
                },
                bpm: Some(Bpm::new(128.5).unwrap()),
                ..NovaFaixa::default()
            })
            .unwrap();
        let f = db.faixa(id).unwrap().unwrap();
        assert_eq!(f.metadados.title.as_deref(), Some("Faixa"));
        assert_eq!(f.bpm.unwrap().get(), 128.5);
        assert!(f.identidade.is_none());
    }
}
