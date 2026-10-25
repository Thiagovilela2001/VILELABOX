//! Frases musicais por faixa.

use crate::error::{mapear_constraint, ErroStorage};
use rusqlite::{params, Connection};
use vilelabox_types::{Frame, Phrase, PhraseKind, TrackId};

fn kind_str(kind: &PhraseKind) -> &str {
    match kind {
        PhraseKind::Intro => "intro",
        PhraseKind::Verse => "verse",
        PhraseKind::Chorus => "chorus",
        PhraseKind::Bridge => "bridge",
        PhraseKind::Up => "up",
        PhraseKind::Down => "down",
        PhraseKind::Outro => "outro",
        PhraseKind::Other(s) => s.as_str(),
    }
}

/// Texto desconhecido volta como `Other`; os nomes canônicos têm preferência
/// na leitura (um `Other("intro")` gravado é lido como `Intro`).
fn kind_from(s: &str) -> PhraseKind {
    match s {
        "intro" => PhraseKind::Intro,
        "verse" => PhraseKind::Verse,
        "chorus" => PhraseKind::Chorus,
        "bridge" => PhraseKind::Bridge,
        "up" => PhraseKind::Up,
        "down" => PhraseKind::Down,
        "outro" => PhraseKind::Outro,
        outro => PhraseKind::Other(outro.to_string()),
    }
}

/// Substitui atomicamente as frases da faixa.
pub fn substituir(con: &Connection, faixa: TrackId, frases: &[Phrase]) -> Result<(), ErroStorage> {
    let tx = con.unchecked_transaction()?;
    tx.execute("DELETE FROM phrases WHERE track_id = ?1", params![faixa.0])
        .map_err(mapear_constraint)?;
    for (i, p) in frases.iter().enumerate() {
        tx.execute(
            "INSERT INTO phrases (track_id, idx, phrase_id, start_frames, end_frames, kind, beats)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                faixa.0,
                i as i64,
                p.id,
                p.start.get(),
                p.end.get(),
                kind_str(&p.kind),
                p.beats,
            ],
        )
        .map_err(mapear_constraint)?;
    }
    tx.commit()?;
    Ok(())
}

/// Frases da faixa na ordem em que foram gravadas.
pub fn listar(con: &Connection, faixa: TrackId) -> Result<Vec<Phrase>, ErroStorage> {
    let mut st = con.prepare(
        "SELECT phrase_id, start_frames, end_frames, kind, beats
         FROM phrases WHERE track_id = ?1 ORDER BY idx",
    )?;
    let rows = st
        .query_map(params![faixa.0], |r| {
            Ok((
                r.get::<_, u32>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, u32>(4)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut frases = Vec::with_capacity(rows.len());
    for (id, inicio, fim, kind, beats) in rows {
        frases.push(Phrase::new(
            id,
            Frame::new(inicio),
            Frame::new(fim),
            kind_from(&kind),
            beats,
        )?);
    }
    Ok(frases)
}
