//! Cues, hot cues e loops por faixa.

use crate::error::{mapear_constraint, ErroStorage};
use rusqlite::{params, Connection};
use vilelabox_types::{Cue, CueKind, CueOrigin, Frame, TrackId};

fn kind_str(kind: CueKind) -> &'static str {
    match kind {
        CueKind::CuePoint => "cue",
        CueKind::HotCue => "hot",
        CueKind::MemoryCue => "memory",
        CueKind::Loop => "loop",
        CueKind::FadeIn => "fade_in",
        CueKind::FadeOut => "fade_out",
    }
}

fn kind_from(s: &str) -> Result<CueKind, ErroStorage> {
    Ok(match s {
        "cue" => CueKind::CuePoint,
        "hot" => CueKind::HotCue,
        "memory" => CueKind::MemoryCue,
        "loop" => CueKind::Loop,
        "fade_in" => CueKind::FadeIn,
        "fade_out" => CueKind::FadeOut,
        outro => {
            return Err(ErroStorage::Validacao(format!(
                "tipo de cue desconhecido: {outro}"
            )))
        }
    })
}

fn origin_str(origin: CueOrigin) -> &'static str {
    match origin {
        CueOrigin::User => "user",
        CueOrigin::Automatic => "auto",
        CueOrigin::Imported => "imported",
    }
}

fn origin_from(s: &str) -> Result<CueOrigin, ErroStorage> {
    Ok(match s {
        "user" => CueOrigin::User,
        "auto" => CueOrigin::Automatic,
        "imported" => CueOrigin::Imported,
        outro => {
            return Err(ErroStorage::Validacao(format!(
                "origem de cue desconhecida: {outro}"
            )))
        }
    })
}

fn inserir_cue(tx: &rusqlite::Transaction, faixa: TrackId, c: &Cue) -> Result<(), ErroStorage> {
    tx.execute(
        "INSERT INTO cues (track_id, kind, slot, position_frames, end_position_frames,
            label, color, origin)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            faixa.0,
            kind_str(c.kind),
            c.slot,
            c.position.get(),
            c.end_position.map(|f| f.get()),
            c.label,
            c.color,
            origin_str(c.origin),
        ],
    )
    .map_err(mapear_constraint)?;
    Ok(())
}

/// Substitui atomicamente o conjunto de cues da faixa.
pub fn substituir(con: &Connection, faixa: TrackId, cues: &[Cue]) -> Result<(), ErroStorage> {
    let tx = con.unchecked_transaction()?;
    tx.execute("DELETE FROM cues WHERE track_id = ?1", params![faixa.0])
        .map_err(mapear_constraint)?;
    for c in cues {
        inserir_cue(&tx, faixa, c)?;
    }
    tx.commit()?;
    Ok(())
}

/// Substitui só os cues da origem dada (ex.: reimportação), preservando os
/// das demais origens. Os inseridos entram sempre com a origem declarada.
pub fn substituir_por_origem(
    con: &Connection,
    faixa: TrackId,
    origem: CueOrigin,
    cues: &[Cue],
) -> Result<(), ErroStorage> {
    let tx = con.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM cues WHERE track_id = ?1 AND origin = ?2",
        params![faixa.0, origin_str(origem)],
    )
    .map_err(mapear_constraint)?;
    for c in cues {
        inserir_cue(
            &tx,
            faixa,
            &Cue {
                origin: origem,
                ..c.clone()
            },
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Cues da faixa em ordem de posição.
pub fn listar(con: &Connection, faixa: TrackId) -> Result<Vec<Cue>, ErroStorage> {
    let mut st = con.prepare(
        "SELECT id, kind, slot, position_frames, end_position_frames, label, color, origin
         FROM cues WHERE track_id = ?1 ORDER BY position_frames, id",
    )?;
    let rows = st.query_map(params![faixa.0], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<u8>>(2)?,
            r.get::<_, i64>(3)?,
            r.get::<_, Option<i64>>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, Option<u32>>(6)?,
            r.get::<_, String>(7)?,
        ))
    })?;
    let mut cues = Vec::new();
    for row in rows {
        let (id, kind, slot, pos, fim, label, color, origin) = row?;
        cues.push(Cue {
            id: Some(id),
            slot,
            kind: kind_from(&kind)?,
            position: Frame::new(pos),
            end_position: fim.map(Frame::new),
            label,
            color,
            origin: origin_from(&origin)?,
        });
    }
    Ok(cues)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tipos_e_origens_tem_ida_e_volta() {
        for k in [
            CueKind::CuePoint,
            CueKind::HotCue,
            CueKind::MemoryCue,
            CueKind::Loop,
            CueKind::FadeIn,
            CueKind::FadeOut,
        ] {
            assert_eq!(kind_from(kind_str(k)).unwrap(), k);
        }
        for o in [CueOrigin::User, CueOrigin::Automatic, CueOrigin::Imported] {
            assert_eq!(origin_from(origin_str(o)).unwrap(), o);
        }
        assert!(kind_from("??").is_err());
        assert!(origin_from("??").is_err());
    }
}
