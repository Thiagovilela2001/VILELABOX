//! Waveforms das tags PWV3, PWV4 e PWV5 do `.EXT`.
//!
//! Conteúdo de cada tag (depois do cabeçalho de 12 bytes da tag ANLZ):
//! - 0..4: u32 BE, bytes por entrada;
//! - 4..8: u32 BE, número de entradas;
//! - 8..12: desconhecido (no PWV5 vale `00 96 03 05`; 0x96 = 150 entradas/s);
//! - a seguir, as entradas.
//!
//! Layout conferido em arquivos `.EXT` reais do rekordbox.

use crate::container::{sections, FourCc};

/// Visão geral (PWV4): 1200 entradas de 6 bytes independentes.
///
/// Bytes 3, 4 e 5 (0..127) são vermelho, verde e azul do modo RGB: a cor é a
/// razão entre eles e a altura acompanha o maior. Conferido contra a tela do
/// rekordbox 7.2.8 com a mesma faixa (2026-10-24). Bytes 0..2 têm significado
/// desconhecido (0 e 2 vão até 127, 1 até 255) e ficam guardados em `raw`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WaveformPreview {
    pub red: Vec<u8>,
    pub green: Vec<u8>,
    pub blue: Vec<u8>,
    pub raw: Vec<[u8; 6]>,
}

/// Detalhe (PWV5 ou PWV3), ~150 entradas por segundo.
///
/// PWV5: u16 BE `w`; `height = (w >> 2) & 0x1F`, `red = (w >> 13) & 7`,
/// `green = (w >> 10) & 7`, `blue = (w >> 7) & 7`. Os três campos são a cor do
/// modo RGB, não magnitude (conferido contra a tela do rekordbox 7.2.8); a
/// altura correlaciona 0,95 com a do PWV3 nos arquivos reais.
/// PWV3: 1 byte, `height = byte & 0x1F`, cor a zero.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WaveformDetail {
    pub height: Vec<u8>,
    pub red: Vec<u8>,
    pub green: Vec<u8>,
    pub blue: Vec<u8>,
}

/// Devolve as entradas se o cabeçalho for coerente com `entry_len` e com o
/// tamanho do conteúdo; caso contrário, `None`.
fn entries(payload: &[u8], entry_len: usize) -> Option<std::slice::ChunksExact<'_, u8>> {
    let len_entry_bytes = u32::from_be_bytes(payload.get(0..4)?.try_into().ok()?) as usize;
    let len_entries = u32::from_be_bytes(payload.get(4..8)?.try_into().ok()?) as usize;
    if len_entry_bytes != entry_len {
        return None;
    }
    let end = len_entries.checked_mul(entry_len)?.checked_add(12)?;
    Some(payload.get(12..end)?.chunks_exact(entry_len))
}

pub fn parse_pwv4(payload: &[u8]) -> WaveformPreview {
    let mut out = WaveformPreview::default();
    for e in entries(payload, 6).into_iter().flatten() {
        out.red.push(e[3]);
        out.green.push(e[4]);
        out.blue.push(e[5]);
        out.raw.push([e[0], e[1], e[2], e[3], e[4], e[5]]);
    }
    out
}

pub fn parse_pwv5(payload: &[u8]) -> WaveformDetail {
    let mut out = WaveformDetail::default();
    for e in entries(payload, 2).into_iter().flatten() {
        let w = u16::from_be_bytes([e[0], e[1]]);
        out.height.push(((w >> 2) & 0x1F) as u8);
        out.red.push(((w >> 13) & 0x7) as u8);
        out.green.push(((w >> 10) & 0x7) as u8);
        out.blue.push(((w >> 7) & 0x7) as u8);
    }
    out
}

pub fn parse_pwv3(payload: &[u8]) -> WaveformDetail {
    let height: Vec<u8> = entries(payload, 1)
        .into_iter()
        .flatten()
        .map(|e| e[0] & 0x1F)
        .collect();
    let zeros = vec![0; height.len()];
    WaveformDetail {
        height,
        red: zeros.clone(),
        green: zeros.clone(),
        blue: zeros,
    }
}

/// Visão geral (PWV4) e detalhe (PWV5, ou PWV3 na falta dele) de um `.EXT`.
/// Tags ausentes ou inconsistentes dão `None`.
pub fn waveform_from_ext(ext: &[u8]) -> (Option<WaveformPreview>, Option<WaveformDetail>) {
    let secs = sections(ext);
    let tag = |t: &FourCc| secs.get(t).copied();
    let preview = tag(b"PWV4").map(parse_pwv4).filter(|p| !p.red.is_empty());
    let detail = tag(b"PWV5")
        .map(parse_pwv5)
        .filter(|d| !d.height.is_empty())
        .or_else(|| {
            tag(b"PWV3")
                .map(parse_pwv3)
                .filter(|d| !d.height.is_empty())
        });
    (preview, detail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::build_anlz_container;

    fn tag(entry_len: u32, count: u32, body: &[u8]) -> Vec<u8> {
        let mut p = entry_len.to_be_bytes().to_vec();
        p.extend(count.to_be_bytes());
        p.extend([0x00, 0x96, 0x03, 0x05]);
        p.extend_from_slice(body);
        p
    }

    #[test]
    fn pwv4_cor_nos_bytes_3_a_5() {
        let p = tag(6, 2, &[9, 200, 8, 1, 2, 3, 0, 0, 0, 0x7F, 0, 0x40]);
        let w = parse_pwv4(&p);
        assert_eq!(w.red, vec![1, 0x7F]);
        assert_eq!(w.green, vec![2, 0]);
        assert_eq!(w.blue, vec![3, 0x40]);
        assert_eq!(w.raw[0], [9, 200, 8, 1, 2, 3]);
    }

    #[test]
    fn pwv5_campos_de_bits() {
        // red=5, green=3, blue=6, height=17
        let w: u16 = (5 << 13) | (3 << 10) | (6 << 7) | (17 << 2);
        let d = parse_pwv5(&tag(2, 1, &w.to_be_bytes()));
        assert_eq!(
            (d.height, d.red, d.blue, d.green),
            (vec![17], vec![5], vec![6], vec![3])
        );
    }

    #[test]
    fn pwv3_so_altura() {
        let d = parse_pwv3(&tag(1, 2, &[0xFF, 0x05]));
        assert_eq!(d.height, vec![0x1F, 5]);
        assert_eq!(d.red, vec![0, 0]);
    }

    #[test]
    fn tamanho_inconsistente_da_vazio() {
        // bytes por entrada errado
        assert!(parse_pwv4(&tag(4, 1, &[0; 6])).red.is_empty());
        // entradas declaradas a mais que o conteúdo
        assert!(parse_pwv5(&tag(2, 10, &[0; 4])).height.is_empty());
        // contagem absurda não estoura
        assert!(parse_pwv3(&tag(1, u32::MAX, &[0])).height.is_empty());
        assert!(parse_pwv3(&[0, 0]).height.is_empty());
    }

    #[test]
    fn ext_prefere_pwv5_e_cai_para_pwv3() {
        let pwv4 = tag(6, 1, &[0, 0, 0, 9, 0, 0]);
        let pwv3 = tag(1, 1, &[7]);
        let pwv5 = tag(2, 1, &(20u16 << 2).to_be_bytes());

        let ext = build_anlz_container(&[(b"PWV4", &pwv4), (b"PWV3", &pwv3), (b"PWV5", &pwv5)]);
        let (p, d) = waveform_from_ext(&ext);
        assert_eq!(p.unwrap().red, vec![9]);
        assert_eq!(d.unwrap().height, vec![20]);

        let ext = build_anlz_container(&[(b"PWV3", &pwv3)]);
        let (p, d) = waveform_from_ext(&ext);
        assert!(p.is_none());
        assert_eq!(d.unwrap().height, vec![7]);

        let bad = tag(3, 1, &[0; 3]);
        let ext = build_anlz_container(&[(b"PWV5", &bad)]);
        assert_eq!(waveform_from_ext(&ext), (None, None));
    }
}
