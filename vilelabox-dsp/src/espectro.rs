//! FFT própria do detector legado e espectro por compasso da passada C
//! (P4.1). Dossiê: "FFT e espectro por compasso" em
//! `spec/rekordbox/evidence/p4-1-analise-batida.md`.

/// FFT radix-2 com decimação no tempo (`0x1411645f0` init,
/// `0x141165390` janela, `0x1411655f0` aplica a janela, `0x141164e90`
/// permutação e borboletas).
#[derive(Debug, Clone)]
pub struct Fft {
    n: usize,
    /// `sen((2π/N)·i)`, `i < N + N/4` (`+0x30`); o cosseno é `seno[i + N/4]`.
    seno: Vec<f64>,
    /// `fma(cos((i − h)·π / (h + 1)), 0,5, 0,5)`, `h = (N − 1)·0,5` (`+0x08`).
    janela: Vec<f64>,
}

impl Fft {
    /// `n` potência de 2 e maior que 7 (`0x1411645f0`).
    ///
    /// LIMITAÇÃO: a janela usa um `cos` vetorial próprio (`0x142608cd0`);
    /// aqui é `f64::cos`, que pode diferir no último bit.
    pub fn nova(n: usize) -> Option<Self> {
        if n < 8 || !n.is_power_of_two() {
            return None;
        }
        let passo = 2.0 * std::f64::consts::PI / n as f64;
        let seno = (0..n + n / 4).map(|i| (passo * i as f64).sin()).collect();
        let h = (n - 1) as f64 * 0.5;
        let janela = (0..n)
            .map(|i| {
                (((i as f64 - h) * std::f64::consts::PI) / (h + 1.0))
                    .cos()
                    .mul_add(0.5, 0.5)
            })
            .collect();
        Some(Self { n, seno, janela })
    }

    /// Janela, permutação por bits invertidos e borboletas, no lugar. Por
    /// estágio de meia largura `l`, gêmeo `m < l` usa
    /// `(c, s) = (seno[m·N/(2l) + N/4], seno[m·N/(2l)])`,
    /// `tr = fma(s, im, c·re)`, `ti = fma(c, im, −(s·re))`,
    /// `x[j] = x[j − l] − t`, `x[j − l] = t + x[j − l]`.
    pub fn processar(&self, x: &mut [(f64, f64)]) {
        let n = self.n;
        for (v, w) in x.iter_mut().zip(&self.janela) {
            v.0 *= w;
            v.1 *= w;
        }
        let bits = n.trailing_zeros();
        for i in 0..n {
            let j = i.reverse_bits() >> (usize::BITS - bits);
            if i < j {
                x.swap(i, j);
            }
        }
        let quarto = n / 4;
        let mut l = 1;
        while l < n {
            let passo = n / (2 * l);
            for m in 0..l {
                let s = self.seno[m * passo];
                let c = self.seno[m * passo + quarto];
                let mut j = m + l;
                while j < n {
                    let (re, im) = x[j];
                    let tr = s.mul_add(im, c * re);
                    let ti = c.mul_add(im, -(s * re));
                    let (ar, ai) = x[j - l];
                    x[j] = (ar - tr, ai - ti);
                    x[j - l] = (tr + ar, ti + ai);
                    j += 2 * l;
                }
            }
            l *= 2;
        }
    }
}

/// Espectro em dB de cada meio compasso (`0x14117b1b0`, passada C),
/// conferido no assembly. Chamada `(obj, +0xaa0, fs = taxa/8, bpm, s, fim,
/// grade +0x1140)`; FFT de 4096 em `+0x1a38` (init `0x14117b170`).
///
/// `C = (fim − s + 1) / 4` compassos; buffer local de
/// `4·ceil((60/bpm)·fs) + 4097` amostras. Compasso `c` (1-based):
/// `g0 = grade[s − 1 + 4(c − 1)]`, próximo `grade[s + 3 + 4(c − 1)]` (no
/// último, `g0 + len_buffer`); copia `x[g0 − 1 ..= min(próximo − 1, len) − 1]`
/// para o buffer zerado. Duas FFTs, a partir de `0` e de
/// `grade[s + 1 + 4(c − 1)] − g0`, com 4096 amostras reais; linha
/// `2(c − 1) + k`, 2048 bins: `20·log10(sqrt(fma(im, im, re²)))` se o módulo
/// for `>= 1e−10` (`0x143538100`, `0x1454155b0`), senão −200 (`0x143538108`).
/// Matriz `+0x1a70` com `2C × 2048`, linha por meio compasso.
///
/// ADAPTAÇÃO: leitura fora do sinal ou do buffer devolve `None`.
pub fn espectro_por_compasso(
    fft: &Fft,
    sinal: &[f64],
    fs: f64,
    bpm: f64,
    grade: &[i32],
    s: i32,
    fim: i32,
) -> Option<Vec<f64>> {
    const N: usize = 4096;
    const BINS: usize = 2048;
    let compassos = (fim - s + 1) / 4;
    let batida = (60.0 / bpm) * fs;
    let len_buffer = usize::try_from(4 * batida.ceil() as i32 + 0x1001).ok()?;
    let mut b: Vec<f64> = Vec::new();
    let mut out = Vec::with_capacity(usize::try_from(compassos).unwrap_or(0) * 2 * BINS);
    let idx = |i: i32| -> Option<i32> { grade.get(usize::try_from(i).ok()?).copied() };
    let mut buf = vec![(0.0, 0.0); N];
    for c in 1..=compassos {
        if b.len() < len_buffer {
            b.resize(len_buffer, 0.0);
        }
        b[..len_buffer].fill(0.0);
        let base = s - 1 + 4 * (c - 1);
        let g0 = idx(base)?;
        let prox = if c < compassos {
            idx(base + 4)?
        } else {
            g0 + b.len() as i32
        };
        let ate = (prox - 1).min(sinal.len() as i32);
        let qtd = ate - g0 + 1;
        if qtd > b.len() as i32 {
            b.resize(qtd as usize, 0.0);
        }
        if qtd > 0 {
            let ini = usize::try_from(g0 - 1).ok()?;
            let qtd = qtd as usize;
            b[..qtd].copy_from_slice(sinal.get(ini..ini + qtd)?);
        }
        for desl in [0, idx(base + 2)? - g0] {
            let desl = usize::try_from(desl).ok()?;
            let seg = b.get(desl..desl + N)?;
            for (v, &x) in buf.iter_mut().zip(seg) {
                *v = (x, 0.0);
            }
            fft.processar(&mut buf);
            out.extend(buf[..BINS].iter().map(|&(re, im)| {
                let m = im.mul_add(im, re * re).sqrt();
                if m >= 1e-10 {
                    m.log10() * 20.0
                } else {
                    -200.0
                }
            }));
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fft_de_seno_cai_no_bin() {
        let fft = Fft::nova(64).unwrap();
        // Sem a janela, um cosseno puro no bin 5 teria |X| = 32 só ali; com a
        // janela de Hann o pico continua no bin 5.
        let mut x: Vec<(f64, f64)> = (0..64)
            .map(|i| {
                (
                    (2.0 * std::f64::consts::PI * 5.0 * i as f64 / 64.0).cos(),
                    0.0,
                )
            })
            .collect();
        fft.processar(&mut x);
        let mag: Vec<f64> = x.iter().map(|v| v.0.hypot(v.1)).collect();
        let max = (0..32).max_by(|&a, &b| mag[a].total_cmp(&mag[b])).unwrap();
        assert_eq!(max, 5);
        assert!(Fft::nova(12).is_none());
    }

    #[test]
    fn fft_bate_com_dft_direta() {
        let n = 16;
        let fft = Fft::nova(n).unwrap();
        let x0: Vec<f64> = (0..n).map(|i| ((i * 7 % 5) as f64) - 2.0).collect();
        let mut x: Vec<(f64, f64)> = x0.iter().map(|&v| (v, 0.0)).collect();
        fft.processar(&mut x);
        for (k, xk) in x.iter().enumerate() {
            let (mut re, mut im) = (0.0, 0.0);
            for (j, v) in x0.iter().enumerate() {
                let w = fft.janela[j] * v;
                let a = -2.0 * std::f64::consts::PI * (j * k) as f64 / n as f64;
                re += w * a.cos();
                im += w * a.sin();
            }
            assert!((xk.0 - re).abs() < 1e-9 && (xk.1 - im).abs() < 1e-9, "{k}");
        }
    }

    #[test]
    fn espectro_por_compasso_gera_duas_linhas_por_compasso() {
        let fft = Fft::nova(4096).unwrap();
        let fs = 5512.5;
        let sinal: Vec<f64> = (0..40000)
            .map(|i| (2.0 * std::f64::consts::PI * 1000.0 * i as f64 / fs).sin())
            .collect();
        // 120 BPM: batida 2756,25 amostras.
        let grade: Vec<i32> = (0..12).map(|i| 1 + (i as f64 * 2756.25) as i32).collect();
        let m = espectro_por_compasso(&fft, &sinal, fs, 120.0, &grade, 1, 8).unwrap();
        assert_eq!(m.len(), 2 * 2 * 2048);
        // 1 kHz: bin 1000·4096/5512,5 ≈ 743.
        let linha = &m[..2048];
        let max = (0..2048)
            .max_by(|&a, &b| linha[a].total_cmp(&linha[b]))
            .unwrap();
        assert!((742..=744).contains(&max), "{max}");
    }
}
