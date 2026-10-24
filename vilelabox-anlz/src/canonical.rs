use crate::{parse_cues, parse_pqtz, parse_pssi, sections, AnlzError, FourCc, RawCue, RawPhrase};
use std::collections::HashMap;
use vilelabox_reference::{ReferenceCue, ReferencePhrase, ReferenceTrack, SCHEMA_VERSION};

pub fn canonical_phrases(raw: &[RawPhrase]) -> Result<Vec<ReferencePhrase>, AnlzError> {
    let available: Vec<&RawPhrase> = raw.iter().filter(|p| p.start_time_s.is_some()).collect();
    let mut result = Vec::with_capacity(available.len());

    for (index, phrase) in available.iter().enumerate() {
        let next = available.get(index + 1);
        let (end, bars) = if let Some(next) = next {
            let beat_delta = i64::from(next.beat) - i64::from(phrase.beat);
            if beat_delta < 0 {
                return Err(AnlzError::Inconsistente(format!(
                    "frase {} tem batida posterior menor que a atual",
                    phrase.index
                )));
            }
            (next.start_time_s, Some(beat_delta.div_euclid(4) as u32))
        } else {
            (None, None)
        };

        let type_name = if phrase.kind.starts_with("verse") {
            "VERSE".to_string()
        } else {
            phrase.kind.to_uppercase()
        };

        result.push(ReferencePhrase {
            start: phrase.start_time_s.expect("filtrado acima"),
            end,
            bars,
            type_name,
            confidence: None,
        });
    }

    Ok(result)
}

pub fn canonical_cues(raw: &[RawCue]) -> Vec<ReferenceCue> {
    let mut cues: Vec<_> = raw
        .iter()
        .map(|cue| ReferenceCue {
            type_name: if cue.is_loop {
                "loop"
            } else if cue.hot_cue != 0 {
                "hotcue"
            } else {
                "memory"
            }
            .to_string(),
            position_s: cue.start_time_s,
            length_s: cue
                .end_time_s
                .map(|end| crate::round3(end - cue.start_time_s)),
            hotcue: if cue.hot_cue == 0 {
                None
            } else {
                Some(cue.hot_cue - 1)
            },
            label: cue.label.clone(),
            color: cue.color,
        })
        .collect();

    cues.sort_by(|left, right| {
        left.position_s
            .total_cmp(&right.position_s)
            .then_with(|| left.type_name.cmp(&right.type_name))
            .then_with(|| left.hotcue.cmp(&right.hotcue))
    });
    cues
}

pub fn cue_payloads<'a>(
    dat: &HashMap<FourCc, &'a [u8]>,
    ext: &HashMap<FourCc, &'a [u8]>,
) -> Vec<&'a [u8]> {
    [*b"PCO2", *b"PCOB"]
        .into_iter()
        .filter_map(|tag| ext.get(&tag).or_else(|| dat.get(&tag)).copied())
        .collect()
}

pub fn track_from_anlz(
    dat: &[u8],
    ext: Option<&[u8]>,
    sha256: &str,
    source_ref: &str,
) -> Result<ReferenceTrack, AnlzError> {
    let dat_sections = sections(dat);
    let ext_sections = ext.map(sections).unwrap_or_default();
    let grid = dat_sections
        .get(b"PQTZ")
        .and_then(|payload| parse_pqtz(payload));

    let phrases = if let (Some(grid), Some(payload)) = (&grid, ext_sections.get(b"PSSI")) {
        let beat_times: Vec<u32> = grid.beats.iter().map(|beat| beat.time_ms).collect();
        canonical_phrases(&parse_pssi(payload, &beat_times))?
    } else {
        Vec::new()
    };

    let raw_cues: Vec<_> = cue_payloads(&dat_sections, &ext_sections)
        .into_iter()
        .flat_map(parse_cues)
        .collect();

    let track = ReferenceTrack {
        schema_version: SCHEMA_VERSION,
        name: None,
        sha256: sha256.to_string(),
        source: Some("rekordbox_anlz".into()),
        source_ref: Some(source_ref.into()),
        sample_rate: None,
        duration: None,
        bpm: grid.as_ref().map(|g| g.bpm),
        tempo_type: if grid.as_ref().is_some_and(|g| g.variable) {
            "variable"
        } else {
            "fixed"
        }
        .into(),
        sections: None,
        first_beat_time_s: grid.as_ref().map(|g| g.first_beat_time_s),
        downbeat_index: grid.as_ref().map(|g| g.downbeat_index),
        key: None,
        energy: None,
        phrases,
        cues: canonical_cues(&raw_cues),
        has_vocal: None,
        vocal_segments: None,
    };
    track.validate()?;
    Ok(track)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn phrase(beat: u16, kind: &str, start: Option<f64>) -> RawPhrase {
        RawPhrase {
            index: beat,
            beat,
            kind: kind.into(),
            kind_id: 0,
            start_time_s: start,
        }
    }

    fn cue(hot_cue: u32, is_loop: bool, start: f64, end: Option<f64>) -> RawCue {
        RawCue {
            hot_cue,
            status: 1,
            is_loop,
            start_time_s: start,
            end_time_s: end,
            label: None,
            color: None,
        }
    }

    #[test]
    fn frases_canonicas() {
        let raw = vec![
            phrase(1, "intro", Some(0.076)),
            phrase(29, "up", Some(12.076)),
            phrase(61, "verse_2", Some(25.79)),
            phrase(99, "outro", None),
        ];
        let phrases = canonical_phrases(&raw).unwrap();
        assert_eq!(phrases.len(), 3);
        assert_eq!(
            (
                phrases[0].type_name.as_str(),
                phrases[0].end,
                phrases[0].bars
            ),
            ("INTRO", Some(12.076), Some(7))
        );
        assert_eq!(
            (
                phrases[2].type_name.as_str(),
                phrases[2].end,
                phrases[2].bars
            ),
            ("VERSE", None, None)
        );
    }

    #[test]
    fn frase_com_batida_decrescente_falha() {
        let raw = vec![phrase(9, "intro", Some(1.0)), phrase(1, "up", Some(2.0))];
        assert!(matches!(
            canonical_phrases(&raw),
            Err(AnlzError::Inconsistente(_))
        ));
    }

    #[test]
    fn cues_canonicas_ordenadas() {
        let cues = canonical_cues(&[
            cue(2, false, 2.5, None),
            cue(0, true, 1.0, Some(3.0)),
            cue(0, false, 1.0, None),
        ]);
        let actual: Vec<_> = cues
            .iter()
            .map(|c| (c.type_name.as_str(), c.position_s, c.hotcue, c.length_s))
            .collect();
        assert_eq!(
            actual,
            vec![
                ("loop", 1.0, None, Some(2.0)),
                ("memory", 1.0, None, None),
                ("hotcue", 2.5, Some(1), None),
            ]
        );
    }

    #[test]
    fn faixa_sem_grade_nao_tem_frases() {
        let mut empty = b"PMAI".to_vec();
        empty.extend(12u32.to_be_bytes());
        empty.extend(12u32.to_be_bytes());
        let track = track_from_anlz(&empty, None, &"a".repeat(64), "x/ANLZ0000.DAT").unwrap();
        assert_eq!(
            (track.bpm, track.tempo_type.as_str(), track.phrases.len()),
            (None, "fixed", 0)
        );
    }
}
