use crate::round3;

const BASE: [u8; 19] = [
    0xCB, 0xE1, 0xEE, 0xFA, 0xE5, 0xEE, 0xAD, 0xEE, 0xE9, 0xD2, 0xE9, 0xEB, 0xE1, 0xE9, 0xF3, 0xE8,
    0xE9, 0xF4, 0xE1,
];

#[derive(Debug, Clone, PartialEq)]
pub struct RawPhrase {
    pub index: u16,
    pub beat: u16,
    pub kind: String,
    pub kind_id: u16,
    pub start_time_s: Option<f64>,
}

fn resolve_kind_name(mood: u16, kind_id: u16) -> String {
    let name = match mood {
        1 => match kind_id {
            1 => Some("intro"),
            2 => Some("up"),
            3 => Some("down"),
            5 => Some("chorus"),
            6 => Some("outro"),
            _ => None,
        },
        2 => match kind_id {
            1 => Some("intro"),
            2 => Some("verse_1"),
            3 => Some("verse_2"),
            4 => Some("verse_3"),
            5 => Some("verse_4"),
            6 => Some("verse_5"),
            7 => Some("verse_6"),
            8 => Some("bridge"),
            9 => Some("chorus"),
            10 => Some("outro"),
            _ => None,
        },
        _ => match kind_id {
            1 => Some("intro"),
            2..=4 => Some("verse_1"),
            5..=7 => Some("verse_2"),
            8 => Some("bridge"),
            9 => Some("chorus"),
            10 => Some("outro"),
            _ => None,
        },
    };

    match name {
        Some(s) => s.to_string(),
        None => format!("phrase_{kind_id}"),
    }
}

pub fn parse_pssi(payload: &[u8], beat_times_ms: &[u32]) -> Vec<RawPhrase> {
    if payload.len() < 6 {
        return Vec::new();
    }

    let Some(len_entry_bytes) = payload.get(0..4) else {
        return Vec::new();
    };
    let Ok(len_entry_arr) = len_entry_bytes.try_into() else {
        return Vec::new();
    };
    let len_entry = u32::from_be_bytes(len_entry_arr) as usize;

    let Some(len_entries_bytes) = payload.get(4..6) else {
        return Vec::new();
    };
    let Ok(len_entries_arr) = len_entries_bytes.try_into() else {
        return Vec::new();
    };
    let len_entries = u16::from_be_bytes(len_entries_arr) as usize;

    if len_entries == 0 || len_entry < 6 {
        return Vec::new();
    }

    let Some(body_slice) = payload.get(6..) else {
        return Vec::new();
    };
    let mut body = body_slice.to_vec();
    if body.len() < 2 {
        return Vec::new();
    }

    let raw_mood = u16::from_be_bytes([body[0], body[1]]);
    if raw_mood > 20 {
        let c = (len_entries & 0xFF) as u8;
        for (i, b) in body.iter_mut().enumerate() {
            let mask_byte = BASE[i % 19].wrapping_add(c);
            *b ^= mask_byte;
        }
    }

    if body.len() < 14 {
        return Vec::new();
    }

    let mood = u16::from_be_bytes([body[0], body[1]]);
    let mut phrases = Vec::with_capacity(len_entries);

    for i in 0..len_entries {
        let Some(mult) = i.checked_mul(len_entry) else {
            break;
        };
        let Some(offset) = 14usize.checked_add(mult) else {
            break;
        };
        let Some(entry_end) = offset.checked_add(len_entry) else {
            break;
        };
        if entry_end > body.len() {
            break;
        }

        let Some(entry_bytes) = body.get(offset..entry_end) else {
            break;
        };
        if entry_bytes.len() < 6 {
            break;
        }

        let index = u16::from_be_bytes([entry_bytes[0], entry_bytes[1]]);
        let beat = u16::from_be_bytes([entry_bytes[2], entry_bytes[3]]);
        let kind_id = u16::from_be_bytes([entry_bytes[4], entry_bytes[5]]);

        let kind = resolve_kind_name(mood, kind_id);

        let start_time_s = if beat >= 1 && (beat as usize) <= beat_times_ms.len() {
            let time_ms = beat_times_ms[(beat - 1) as usize];
            Some(round3(time_ms as f64 / 1000.0))
        } else {
            None
        };

        phrases.push(RawPhrase {
            index,
            beat,
            kind,
            kind_id,
            start_time_s,
        });
    }

    phrases
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pssi(len_entry: usize, mood: u16, entries: &[(u16, u16, u16)], mask: bool) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend((len_entry as u32).to_be_bytes());
        payload.extend((entries.len() as u16).to_be_bytes());

        let mut body = Vec::new();
        body.extend(mood.to_be_bytes());
        body.extend([0u8; 12]); // 14 bytes de cabeçalho do body

        for &(index, beat, kind_id) in entries {
            let mut entry = Vec::new();
            entry.extend(index.to_be_bytes());
            entry.extend(beat.to_be_bytes());
            entry.extend(kind_id.to_be_bytes());
            if entry.len() < len_entry {
                entry.resize(len_entry, 0);
            }
            body.extend(entry);
        }

        if mask {
            let c = (entries.len() & 0xFF) as u8;
            for (i, b) in body.iter_mut().enumerate() {
                let mask_byte = BASE[i % 19].wrapping_add(c);
                *b ^= mask_byte;
            }
        }

        payload.extend(body);
        payload
    }

    #[test]
    fn pssi_sem_mascara_humor_alto() {
        let r = parse_pssi(
            &pssi(24, 1, &[(1, 1, 1), (2, 3, 5)], false),
            &[500, 1000, 1500],
        );
        assert_eq!(
            r.iter()
                .map(|p| (p.kind.as_str(), p.start_time_s))
                .collect::<Vec<_>>(),
            vec![("intro", Some(0.5)), ("chorus", Some(1.5))]
        );
    }

    #[test]
    fn pssi_mascarado_igual_ao_sem_mascara() {
        let plano = parse_pssi(&pssi(24, 2, &[(1, 1, 9)], false), &[500]);
        let mascarado = parse_pssi(&pssi(24, 2, &[(1, 1, 9)], true), &[500]);
        assert_eq!(plano, mascarado);
        assert_eq!(plano[0].kind, "chorus");
    }

    #[test]
    fn pssi_kind_desconhecido_e_batida_fora() {
        let r = parse_pssi(&pssi(24, 3, &[(1, 99, 42)], false), &[500]);
        assert_eq!((r[0].kind.as_str(), r[0].start_time_s), ("phrase_42", None));
    }

    #[test]
    fn pssi_truncado() {
        assert!(parse_pssi(&[0, 0, 0, 24, 0, 1, 0], &[]).is_empty());
    }
}
