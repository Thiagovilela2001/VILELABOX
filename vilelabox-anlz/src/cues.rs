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

fn be_u16(data: &[u8], range: std::ops::Range<usize>) -> Option<u16> {
    Some(u16::from_be_bytes(data.get(range)?.try_into().ok()?))
}

fn be_u32(data: &[u8], range: std::ops::Range<usize>) -> Option<u32> {
    Some(u32::from_be_bytes(data.get(range)?.try_into().ok()?))
}

fn decode_utf16be_label(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() || bytes.len() % 2 != 0 {
        return None;
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect();
    let decoded = String::from_utf16(&units).ok()?;
    let trimmed = decoded.trim_matches('\0');
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_pcpt_entry(entry: &[u8]) -> Option<RawCue> {
    if entry.len() < 28 {
        return None;
    }
    let hot_cue = be_u32(entry, 0..4)?;
    let status = be_u32(entry, 4..8)?;
    let kind = entry[16];
    if kind != 1 && kind != 2 {
        return None;
    }
    let time_ms = be_u32(entry, 20..24)?;
    let loop_ms = be_u32(entry, 24..28)?;
    let is_loop = kind == 2;

    Some(RawCue {
        hot_cue,
        status,
        is_loop,
        start_time_s: round3(time_ms as f64 / 1000.0),
        end_time_s: (is_loop && loop_ms > time_ms).then(|| round3(loop_ms as f64 / 1000.0)),
        label: None,
        color: None,
    })
}

fn parse_pcp2_entry(entry: &[u8]) -> Option<RawCue> {
    if entry.len() < 28 {
        return None;
    }
    let hot_cue = be_u32(entry, 0..4)?;
    let kind = entry[4];
    if kind != 1 && kind != 2 {
        return None;
    }
    let time_ms = be_u32(entry, 8..12)?;
    let loop_ms = be_u32(entry, 12..16)?;
    let is_loop = kind == 2;

    let mut label = None;
    let mut color = None;

    if entry.len() >= 32 {
        let len_comment = be_u16(entry, 30..32).unwrap_or(0) as usize;
        let comment_end = 32usize.checked_add(len_comment)?;
        if comment_end > entry.len() {
            return None;
        }
        if len_comment > 0 {
            label = decode_utf16be_label(&entry[32..comment_end]);
        }
        if let Some(color_slice) = entry.get(comment_end..comment_end.checked_add(4)?) {
            let code = color_slice[0];
            let r = color_slice[1];
            let g = color_slice[2];
            let b = color_slice[3];
            let rgb = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            if rgb != 0 || code != 0 {
                color = Some(rgb);
            }
        }
    }

    Some(RawCue {
        hot_cue,
        status: 0,
        is_loop,
        start_time_s: round3(time_ms as f64 / 1000.0),
        end_time_s: (is_loop && loop_ms > time_ms).then(|| round3(loop_ms as f64 / 1000.0)),
        label,
        color,
    })
}

/// Lê entradas `PCPT` (de seções `PCOB`) e `PCP2` (de seções `PCO2`).
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
        let tag = payload.get(pos..pos + 4);
        if tag != Some(b"PCPT") && tag != Some(b"PCP2") {
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
        let parsed = if tag == Some(b"PCP2") {
            parse_pcp2_entry(entry)
        } else {
            parse_pcpt_entry(entry)
        };
        if let Some(cue) = parsed {
            cues.push(cue);
        }

        pos = end;
    }

    cues
}

/// Alias explícito para leitura de seções estendidas `PCO2`/`PCP2`.
pub fn parse_cues_extended(payload: &[u8]) -> Vec<RawCue> {
    parse_cues(payload)
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
    fn cues_pcp2_invalido_encerra() {
        let mut payload = vec![0u8; 8];
        payload.extend(b"PCP2");
        payload.extend([0u8; 60]);
        assert!(parse_cues(&payload).is_empty());
    }

    type Pco2TestEntry<'a> = (u32, u8, u32, u32, &'a str, Option<(u8, u8, u8)>);

    fn pco2(entries: &[Pco2TestEntry<'_>]) -> Vec<u8> {
        let mut payload = vec![0u8; 8];
        payload[4..6].copy_from_slice(&(entries.len() as u16).to_be_bytes());
        for &(hot_cue, kind, time_ms, loop_ms, comment, rgb) in entries {
            let mut comment_bytes = Vec::new();
            if !comment.is_empty() {
                for code_unit in comment.encode_utf16() {
                    comment_bytes.extend(code_unit.to_be_bytes());
                }
                comment_bytes.extend(0u16.to_be_bytes());
            }
            let mut entry = vec![0u8; 32];
            entry[0..4].copy_from_slice(&hot_cue.to_be_bytes());
            entry[4] = kind;
            entry[6..8].copy_from_slice(&1000u16.to_be_bytes());
            entry[8..12].copy_from_slice(&time_ms.to_be_bytes());
            entry[12..16].copy_from_slice(&loop_ms.to_be_bytes());
            entry[30..32].copy_from_slice(&(comment_bytes.len() as u16).to_be_bytes());
            entry.extend(comment_bytes);
            if let Some((r, g, b)) = rgb {
                entry.extend([1u8, r, g, b]);
            }
            let len_entry = (12 + entry.len()) as u32;
            payload.extend(b"PCP2");
            payload.extend(16u32.to_be_bytes());
            payload.extend(len_entry.to_be_bytes());
            payload.extend(entry);
        }
        payload
    }

    #[test]
    fn pco2_com_comentario_e_cor() {
        let cues = parse_cues_extended(&pco2(&[
            (1, 1, 1500, 0, "Drop", Some((0x30, 0x5a, 0xff))),
            (0, 2, 4000, 8000, "Loop Intro", Some((0x00, 0xc8, 0x64))),
        ]));
        assert_eq!(cues.len(), 2);
        assert_eq!(
            (
                cues[0].hot_cue,
                cues[0].is_loop,
                cues[0].start_time_s,
                cues[0].label.as_deref(),
                cues[0].color
            ),
            (1, false, 1.5, Some("Drop"), Some(0x305aff))
        );
        assert_eq!(
            (
                cues[1].hot_cue,
                cues[1].is_loop,
                cues[1].start_time_s,
                cues[1].end_time_s,
                cues[1].label.as_deref(),
                cues[1].color
            ),
            (0, true, 4.0, Some(8.0), Some("Loop Intro"), Some(0x00c864))
        );
    }

    #[test]
    fn pco2_sem_comentario_e_sem_cor() {
        let mut payload = vec![0u8; 8];
        payload.extend(b"PCP2");
        payload.extend(16u32.to_be_bytes());
        payload.extend(40u32.to_be_bytes());
        let mut entry = vec![0u8; 28];
        entry[0..4].copy_from_slice(&2u32.to_be_bytes());
        entry[4] = 1;
        entry[8..12].copy_from_slice(&2000u32.to_be_bytes());
        payload.extend(entry);

        let cues = parse_cues(&payload);
        assert_eq!(cues.len(), 1);
        assert_eq!(
            (
                cues[0].hot_cue,
                cues[0].start_time_s,
                cues[0].label.as_deref(),
                cues[0].color
            ),
            (2, 2.0, None, None)
        );
    }

    #[test]
    fn pco2_comentario_truncado_ignorado_sem_panico() {
        let mut payload = vec![0u8; 8];
        payload.extend(b"PCP2");
        payload.extend(16u32.to_be_bytes());
        payload.extend(44u32.to_be_bytes());
        let mut entry = vec![0u8; 32];
        entry[0..4].copy_from_slice(&1u32.to_be_bytes());
        entry[4] = 1;
        entry[8..12].copy_from_slice(&1000u32.to_be_bytes());
        entry[30..32].copy_from_slice(&20u16.to_be_bytes()); // Excede o tamanho da entrada
        payload.extend(entry);

        assert!(parse_cues(&payload).is_empty());
    }
}
