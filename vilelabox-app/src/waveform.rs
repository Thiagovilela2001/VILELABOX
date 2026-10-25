//! Renderização das formas de onda no modo RGB, a partir das tags do `.EXT`
//! já interpretadas por `vilelabox_anlz::waveform`.
//!
//! - Geral (PWV4): 1200 colunas, barra ancorada embaixo; cor = bytes R, G, B
//!   normalizados pelo maior dos três, altura = o maior (0..127).
//! - Ampliada (PWV5): 150 colunas por segundo, barra espelhada no centro; cor =
//!   os três campos de 3 bits (R, G, B), altura = campo de 5 bits.
//!
//! Mapeamento e janela conferidos contra a tela do rekordbox 7.2.8 com a mesma
//! faixa (2026-10-24).
//!
//! O resultado é um buffer RGBA que o Slint estica até o tamanho da área.

use slint::{Rgba8Pixel, SharedPixelBuffer};
use vilelabox_anlz::{WaveformDetail, WaveformPreview};

/// Altura em pixels dos buffers; o Slint estica para a altura real.
pub const ALTURA: u32 = 64;
/// Entradas do PWV5 por segundo.
pub const DETALHE_POR_SEGUNDO: usize = 150;
/// Janela da forma de onda ampliada no zoom padrão, em segundos (na tela de
/// referência, 24 s ocupam a largura da linha).
pub const JANELA_S: usize = 24;

const FUNDO: Rgba8Pixel = Rgba8Pixel {
    r: 0,
    g: 0,
    b: 0,
    a: 0,
};

fn coluna(buf: &mut SharedPixelBuffer<Rgba8Pixel>, x: u32, y0: u32, y1: u32, cor: Rgba8Pixel) {
    let w = buf.width();
    let px = buf.make_mut_slice();
    for y in y0..y1.min(ALTURA) {
        px[(y * w + x) as usize] = cor;
    }
}

/// Visão geral: uma coluna por entrada do PWV4.
pub fn render_preview(p: &WaveformPreview) -> SharedPixelBuffer<Rgba8Pixel> {
    let n = p.red.len().min(p.green.len()).min(p.blue.len());
    let mut buf = SharedPixelBuffer::new(n.max(1) as u32, ALTURA);
    buf.make_mut_slice().fill(FUNDO);
    for i in 0..n {
        let (r, g, b) = (
            f32::from(p.red[i]),
            f32::from(p.green[i]),
            f32::from(p.blue[i]),
        );
        let pico = r.max(g).max(b);
        if pico <= 0.0 {
            continue;
        }
        let cor = Rgba8Pixel {
            r: (255.0 * r / pico) as u8,
            g: (255.0 * g / pico) as u8,
            b: (255.0 * b / pico) as u8,
            a: 255,
        };
        let h = ((pico / 127.0).min(1.0) * ALTURA as f32).round() as u32;
        coluna(&mut buf, i as u32, ALTURA - h, ALTURA, cor);
    }
    buf
}

/// Forma de onda ampliada: `colunas` entradas do PWV5 a partir de `inicio`
/// (pode ser negativo, antes do começo da faixa, que fica vazio).
pub fn render_detail(
    d: &WaveformDetail,
    inicio: isize,
    colunas: usize,
) -> SharedPixelBuffer<Rgba8Pixel> {
    let mut buf = SharedPixelBuffer::new(colunas.max(1) as u32, ALTURA);
    buf.make_mut_slice().fill(FUNDO);
    let meio = ALTURA / 2;
    for c in 0..colunas {
        let Ok(i) = usize::try_from(inicio + c as isize) else {
            continue;
        };
        let (Some(&h), Some(&r), Some(&g), Some(&b)) =
            (d.height.get(i), d.red.get(i), d.green.get(i), d.blue.get(i))
        else {
            break;
        };
        // PWV3 não tem cor: cinza claro.
        let cor = if r | g | b == 0 {
            Rgba8Pixel {
                r: 200,
                g: 200,
                b: 200,
                a: 255,
            }
        } else {
            let canal = |v: u8| (u16::from(v.min(7)) * 255 / 7) as u8;
            Rgba8Pixel {
                r: canal(r),
                g: canal(g),
                b: canal(b),
                a: 255,
            }
        };
        let meia = (u32::from(h) * meio).div_ceil(31);
        coluna(&mut buf, c as u32, meio - meia.min(meio), meio + meia, cor);
    }
    buf
}

/// Janela (início, duração) em segundos da forma de onda ampliada.
pub fn janela(posicao_s: f64) -> (f64, f64) {
    (posicao_s - JANELA_S as f64 / 2.0, JANELA_S as f64)
}

/// Colunas por bloco: abaixo do limite de textura comum (8192) das GPUs.
pub const BLOCO: usize = 4096;

/// Faixa inteira em blocos `(coluna inicial, buffer)`; renderizada uma vez por
/// carga, a UI só desloca os blocos conforme a posição.
pub fn detail_blocks(d: &WaveformDetail) -> Vec<(usize, SharedPixelBuffer<Rgba8Pixel>)> {
    let total = d.height.len();
    (0..total)
        .step_by(BLOCO)
        .map(|i| (i, render_detail(d, i as isize, BLOCO.min(total - i))))
        .collect()
}


#[cfg(test)]
mod tests {
    use super::*;

    fn px(buf: &SharedPixelBuffer<Rgba8Pixel>, x: u32, y: u32) -> Rgba8Pixel {
        buf.as_slice()[(y * buf.width() + x) as usize]
    }

    #[test]
    fn geral_ancora_embaixo_e_colore_por_banda() {
        let p = WaveformPreview {
            red: vec![127, 0],
            green: vec![0, 0],
            blue: vec![64, 0],
            raw: Vec::new(),
        };
        let b = render_preview(&p);
        assert_eq!(b.width(), 2);
        let base = px(&b, 0, ALTURA - 1);
        assert_eq!((base.r, base.g, base.a), (255, 0, 255));
        assert!(base.b > 125 && base.b < 132);
        assert_eq!(px(&b, 0, 0).a, 255); // altura total
        assert_eq!(px(&b, 1, ALTURA - 1).a, 0); // silêncio
    }

    #[test]
    fn blocos_cobrem_a_faixa_inteira() {
        let n = BLOCO * 2 + 10;
        let d = WaveformDetail {
            height: vec![31; n],
            red: vec![7; n],
            green: vec![0; n],
            blue: vec![0; n],
        };
        let b = detail_blocks(&d);
        let v: Vec<(usize, u32)> = b.iter().map(|(i, buf)| (*i, buf.width())).collect();
        assert_eq!(v, [(0, 4096), (4096, 4096), (8192, 10)]);
    }

    #[test]
    fn ampliada_espelha_e_respeita_inicio_negativo() {
        let d = WaveformDetail {
            height: vec![31, 0],
            red: vec![7, 0],
            green: vec![0, 0],
            blue: vec![0, 0],
        };
        let b = render_detail(&d, -1, 3);
        assert_eq!(px(&b, 0, ALTURA / 2).a, 0); // antes do começo
        let c = px(&b, 1, 0);
        assert_eq!((c.r, c.g, c.b, c.a), (255, 0, 0, 255)); // vermelho, altura cheia
        assert_eq!(px(&b, 1, ALTURA - 1).a, 255);
        assert_eq!(px(&b, 2, ALTURA / 2).a, 0); // altura zero
    }
}
