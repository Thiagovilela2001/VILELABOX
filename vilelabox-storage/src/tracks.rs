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
