use crate::round3;

#[derive(Debug, Clone, PartialEq)]
pub struct RawCue {
    pub hot_cue: u32,
    pub status: u32,
    pub is_loop: bool,
    pub start_time_s: f64,
    pub end_time_s: Option<f64>,
    pub label: Option<String>,
    pub color: Option<u32>,
}

fn be_u32(data: &[u8], range: std::ops::Range<usize>) -> Option<u32> {
    Some(u32::from_be_bytes(data.get(range)?.try_into().ok()?))
}

/// Lê entradas `PCPT` de uma seção `PCOB`.
///
/// Entradas desconhecidas são ignoradas. Comprimentos inválidos encerram a
/// leitura para evitar laços infinitos e acessos fora do buffer.
pub fn parse_cues(payload: &[u8]) -> Vec<RawCue> {
    if payload.len() < 8 {
        return Vec::new();
    }

    let mut cues = Vec::new();
    let mut pos = 8usize;

    while pos.saturating_add(40) <= payload.len() {
        if payload.get(pos..pos + 4) != Some(b"PCPT") {
            pos += 1;
            continue;
        }

        let Some(len_entry) = be_u32(payload, pos + 8..pos + 12).map(|v| v as usize) else {
            break;
        };
        if len_entry < 12 {
            break;
        }
        let Some(end) = pos.checked_add(len_entry) else {
            break;
        };
        if end > payload.len() {
            break;
        }

        let Some(entry) = payload.get(pos + 12..end) else {
            break;
        };
        if entry.len() >= 28 {
            let hot_cue = be_u32(entry, 0..4).unwrap_or(0);
            let status = be_u32(entry, 4..8).unwrap_or(0);
            let kind = entry[16];
            let time_ms = be_u32(entry, 20..24).unwrap_or(0);
            let loop_ms = be_u32(entry, 24..28).unwrap_or(0);
            let is_loop = kind == 2;

            cues.push(RawCue {
                hot_cue,
                status,
                is_loop,
                start_time_s: round3(time_ms as f64 / 1000.0),
                end_time_s: (is_loop && loop_ms > time_ms).then(|| round3(loop_ms as f64 / 1000.0)),
                label: None,
                color: None,
            });
        }

        pos = end;
    }

    cues
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pcob(entries: &[(u32, u8, u32, u32)]) -> Vec<u8> {
        let mut payload = vec![0u8; 8];
        for &(hot_cue, kind, time_ms, loop_ms) in entries {
            payload.extend(b"PCPT");
            payload.extend(12u32.to_be_bytes());
            payload.extend(56u32.to_be_bytes());
            let mut entry = vec![0u8; 44];
            entry[0..4].copy_from_slice(&hot_cue.to_be_bytes());
            entry[4..8].copy_from_slice(&1u32.to_be_bytes());
            entry[16] = kind;
            entry[20..24].copy_from_slice(&time_ms.to_be_bytes());
            entry[24..28].copy_from_slice(&loop_ms.to_be_bytes());
            payload.extend(entry);
        }
        payload
    }

    #[test]
    fn cues_memoria_hot_e_loop() {
        let cues = parse_cues(&pcob(&[
            (0, 1, 1000, 0),
            (2, 1, 2500, 0),
            (0, 2, 4000, 6000),
        ]));
        assert_eq!(cues.len(), 3);
        assert_eq!((cues[1].hot_cue, cues[1].start_time_s), (2, 2.5));
        assert_eq!((cues[2].is_loop, cues[2].end_time_s), (true, Some(6.0)));
    }

    #[test]
    fn cues_loop_sem_fim_valido() {
        let cues = parse_cues(&pcob(&[(0, 2, 4000, 3000)]));
        assert_eq!(cues[0].end_time_s, None);
    }

    #[test]
    fn cues_len_entry_invalido_encerra() {
        let mut payload = vec![0u8; 8];
        payload.extend(b"PCPT");
        payload.extend(12u32.to_be_bytes());
        payload.extend(0u32.to_be_bytes());
        payload.extend([0u8; 40]);
        assert!(parse_cues(&payload).is_empty());
    }

    #[test]
    fn cues_ignora_pcp2() {
        let mut payload = vec![0u8; 8];
        payload.extend(b"PCP2");
        payload.extend([0u8; 60]);
        assert!(parse_cues(&payload).is_empty());
    }
}
