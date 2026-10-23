use std::collections::HashMap;

pub type FourCc = [u8; 4];

pub fn all_sections(data: &[u8]) -> Vec<(FourCc, &[u8])> {
    if data.len() < 12 || data.get(..4) != Some(b"PMAI") {
        return Vec::new();
    }
    let Some(header_bytes) = data.get(4..8) else {
        return Vec::new();
    };
    let Ok(header_arr) = header_bytes.try_into() else {
        return Vec::new();
    };
    let len_header = u32::from_be_bytes(header_arr) as usize;

    let mut pos = len_header;
    let mut result = Vec::new();

    while pos.saturating_add(12) <= data.len() {
        let Some(fourcc_slice) = data.get(pos..pos + 4) else {
            break;
        };
        let Ok(fourcc) = fourcc_slice.try_into() else {
            break;
        };

        let Some(len_tag_bytes) = data.get(pos + 8..pos + 12) else {
            break;
        };
        let Ok(len_tag_arr) = len_tag_bytes.try_into() else {
            break;
        };
        let len_tag = u32::from_be_bytes(len_tag_arr) as usize;

        if len_tag < 12 {
            break;
        }

        let Some(end_pos) = pos.checked_add(len_tag) else {
            break;
        };
        if end_pos > data.len() {
            break;
        }

        let Some(payload) = data.get(pos + 12..end_pos) else {
            break;
        };

        result.push((fourcc, payload));
        pos = end_pos;
    }

    result
}

pub fn sections(data: &[u8]) -> HashMap<FourCc, &[u8]> {
    let mut map = HashMap::new();
    for (fourcc, payload) in all_sections(data) {
        map.insert(fourcc, payload);
    }
    map
}

pub fn parse_ppth(payload: &[u8]) -> Option<String> {
    if payload.len() < 4 {
        return None;
    }
    let len_path_bytes = payload.get(..4)?.try_into().ok()?;
    let len_path = u32::from_be_bytes(len_path_bytes) as usize;

    let total_needed = 4usize.checked_add(len_path)?;
    if payload.len() < total_needed {
        return None;
    }

    let text_len = len_path.saturating_sub(2);
    if text_len % 2 != 0 {
        return None;
    }

    let raw_text = payload.get(4..4 + text_len)?;
    let u16s: Vec<u16> = raw_text
        .chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();

    let text = String::from_utf16(&u16s).ok()?;
    Some(text.trim_matches('\0').to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secoes_basicas() {
        let payload = b"Hello, Pioneer!";
        let mut data = b"PMAI".to_vec();
        data.extend(16u32.to_be_bytes());
        data.extend(((16 + 12 + payload.len()) as u32).to_be_bytes());
        data.extend([0u8; 4]);
        data.extend(b"TEST");
        data.extend(12u32.to_be_bytes());
        data.extend(((12 + payload.len()) as u32).to_be_bytes());
        data.extend(payload);
        assert_eq!(sections(&data).get(b"TEST").copied(), Some(&payload[..]));
    }

    #[test]
    fn secoes_ultima_vence_e_all_mantem_todas() {
        let mut data = b"PMAI".to_vec();
        data.extend(12u32.to_be_bytes());
        data.extend(0u32.to_be_bytes());

        // Primeira seção PCOB com payload b"a"
        data.extend(b"PCOB");
        data.extend(12u32.to_be_bytes());
        data.extend(13u32.to_be_bytes());
        data.extend(b"a");

        // Segunda seção PCOB com payload b"b"
        data.extend(b"PCOB");
        data.extend(12u32.to_be_bytes());
        data.extend(13u32.to_be_bytes());
        data.extend(b"b");

        assert_eq!(sections(&data).get(b"PCOB").copied(), Some(&b"b"[..]));
        let all = all_sections(&data);
        assert_eq!(all.len(), 2);
        assert_eq!(all[0], (*b"PCOB", &b"a"[..]));
        assert_eq!(all[1], (*b"PCOB", &b"b"[..]));
    }

    #[test]
    fn secoes_tag_estourada() {
        let mut data = b"PMAI".to_vec();
        data.extend(12u32.to_be_bytes());
        data.extend(0u32.to_be_bytes());
        data.extend(b"TEST");
        data.extend(12u32.to_be_bytes());
        data.extend(1000u32.to_be_bytes()); // Maior que o tamanho do buffer
        data.extend([0u8; 10]);
        assert!(sections(&data).is_empty());
    }

    #[test]
    fn sem_pmai_nao_tem_secoes() {
        assert!(sections(b"XXXX0000000000000000").is_empty());
    }

    #[test]
    fn ppth_ok() {
        let text = "/music/sample.mp3";
        let mut u16_bytes = Vec::new();
        for c in text.encode_utf16() {
            u16_bytes.extend(c.to_be_bytes());
        }
        u16_bytes.extend(0u16.to_be_bytes()); // NUL terminator
        let len_path = u16_bytes.len() as u32;

        let mut payload = Vec::new();
        payload.extend(len_path.to_be_bytes());
        payload.extend(u16_bytes);

        assert_eq!(parse_ppth(&payload), Some("/music/sample.mp3".to_string()));
    }

    #[test]
    fn ppth_invalido() {
        assert_eq!(parse_ppth(&[0, 0]), None);
        assert_eq!(parse_ppth(&[0, 0, 0, 5, 0, 0x41, 0]), None);
        assert_eq!(parse_ppth(&[0, 0, 0, 4, 0xD8, 0x00, 0, 0]), None);
    }
}
