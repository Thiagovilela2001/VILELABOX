//! Grade de batidas por faixa, com bloqueio de edição.

use crate::error::{mapear_constraint, ErroStorage};
use rusqlite::{params, Connection, OptionalExtension};
use vilelabox_types::{Beat, BeatGrid, Bpm, Frame, SampleRate, TempoChange, TrackId};

/// Grade persistida e o estado de bloqueio da edição.
#[derive(Debug, Clone, PartialEq)]
pub struct GradeRow {
    pub grade: BeatGrid,
    pub bloqueada: bool,
}

/// Grava a grade da faixa (batidas e mapa de tempo substituídos na mesma
/// transação) junto com o bloqueio de edição.
pub fn salvar(
    con: &Connection,
    faixa: TrackId,
    grade: &BeatGrid,
    bloqueada: bool,
) -> Result<(), ErroStorage> {
    let tx = con.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO beatgrids (track_id, sample_rate, downbeat_frame, locked, updated_at)
         VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         ON CONFLICT(track_id) DO UPDATE SET
            sample_rate = excluded.sample_rate,
            downbeat_frame = excluded.downbeat_frame,
            locked = excluded.locked,
            updated_at = excluded.updated_at",
        params![
            faixa.0,
            grade.sample_rate().get(),
            grade.downbeat().map(|f| f.get()),
            bloqueada,
        ],
    )
    .map_err(mapear_constraint)?;
    tx.execute(
        "DELETE FROM grid_beats WHERE track_id = ?1",
        params![faixa.0],
    )?;
    for (i, b) in grade.beats().iter().enumerate() {
        tx.execute(
            "INSERT INTO grid_beats (track_id, idx, position_frames, beat_num)
             VALUES (?1, ?2, ?3, ?4)",
            params![faixa.0, i as i64, b.position.get(), b.beat_num],
        )?;
    }
    tx.execute(
        "DELETE FROM grid_tempo WHERE track_id = ?1",
        params![faixa.0],
    )?;
    for (i, t) in grade.tempo_map().iter().enumerate() {
        tx.execute(
            "INSERT INTO grid_tempo (track_id, idx, position_frames, bpm)
             VALUES (?1, ?2, ?3, ?4)",
            params![faixa.0, i as i64, t.position.get(), t.bpm.get()],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Grade e bloqueio da faixa; `None` quando a faixa nunca teve grade gravada.
/// Batidas fora de ordem (banco corrompido) voltam como erro tipado.
pub fn obter(con: &Connection, faixa: TrackId) -> Result<Option<GradeRow>, ErroStorage> {
    let cab = con
        .query_row(
            "SELECT sample_rate, downbeat_frame, locked FROM beatgrids WHERE track_id = ?1",
            params![faixa.0],
            |r| {
                Ok((
                    r.get::<_, u32>(0)?,
                    r.get::<_, Option<i64>>(1)?,
                    r.get::<_, i64>(2)? != 0,
                ))
            },
        )
        .optional()?;
    let Some((taxa, downbeat, bloqueada)) = cab else {
        return Ok(None);
    };
    let mut st = con.prepare(
        "SELECT position_frames, beat_num FROM grid_beats WHERE track_id = ?1 ORDER BY idx",
    )?;
    let beats = st
        .query_map(params![faixa.0], |r| {
            Ok(Beat {
                position: Frame::new(r.get(0)?),
                beat_num: r.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut st = con
        .prepare("SELECT position_frames, bpm FROM grid_tempo WHERE track_id = ?1 ORDER BY idx")?;
    let tempo_rows = st
        .query_map(params![faixa.0], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut tempo = Vec::with_capacity(tempo_rows.len());
    for (pos, bpm) in tempo_rows {
        tempo.push(TempoChange {
            position: Frame::new(pos),
            bpm: Bpm::new(bpm)?,
        });
    }
    let grade = BeatGrid::new(
        SampleRate::new(taxa)?,
        beats,
        tempo,
        downbeat.map(Frame::new),
    )?;
    Ok(Some(GradeRow { grade, bloqueada }))
}
