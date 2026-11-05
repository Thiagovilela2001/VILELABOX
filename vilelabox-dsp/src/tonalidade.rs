//! Transformada de Q constante da passada C (P4.1): núcleos montados em
//! `0x14117ff60` e consumidos por lote em `0x1411803f0`. Dossiê: seção
//! "Tonalidade" de `spec/rekordbox/evidence/p4-1-analise-batida.md`.

/// Bins do núcleo (`0x48`): 6 oitavas × 12.
pub const BINS: usize = 72;
/// Primeiro bin calculado por `0x1411803f0` (`0x17`); os anteriores ficam 0.
const PRIMEIRO_BIN: usize = 23;
/// Salto entre quadros (`0x100`; `0x145414f10` = 1/256).
const SALTO: usize = 256;

/// Janela de Hann simétrica (`0x14117dfc0`):
/// `w[i − 1] = w[L − i] = (1 − cos(i·2π·(1/(L + 1))))·0,5`; com `L` ímpar o
/// meio é `fma(−cos(m·2π/(L + 1)), 0,5, 0,5)`, `m = (L + 1)/2`.
pub fn hann(l: usize) -> Vec<f64> {
    let dois_pi = 2.0 * std::f64::consts::PI;
    let mut w = vec![0.0; l];
    if l % 2 == 0 {
        let inv = 1.0 / (l + 1) as f64;
        for i in 1..=l / 2 {
            let v = (1.0 - (i as f64 * dois_pi * inv).cos()) * 0.5;
            w[i - 1] = v;
            w[l - i] = v;
        }
    } else {
        let m = l.div_ceil(2);
        let d = (l + 1) as f64;
        let inv = 1.0 / d;
        for i in 1..m {
            let v = (1.0 - (i as f64 * dois_pi * inv).cos()) * 0.5;
            w[i - 1] = v;
            w[l - i] = v;
        }
        w[m - 1] = (-((m as f64 * dois_pi) / d).cos()).mul_add(0.5, 0.5);
    }
    w
}

/// Estado da transformada (`motor+0x1998`–`+0x1a34`).
#[derive(Debug, Clone)]
pub struct Tonalidade {
    /// Comprimento de cada núcleo (`+0x19c0`).
    pub comprimentos: Vec<usize>,
    /// Peso `L0 / Lk` (`+0x19e0`; o bin 0 vale 1).
    pub pesos: Vec<f64>,
    /// Núcleos complexos `(re, im)` janelados, só as `Lk` primeiras amostras
    /// (`+0x1a00`; o consumidor nunca lê além delas).
    nucleos: Vec<Vec<(f64, f64)>>,
    /// Sobra do lote anterior (`+0x1998`, quantidade em `+0x19b8`).
    sobra: Vec<f64>,
}

/// Resultado de um lote (`+0x1a18`, `+0x1a30`, `+0x1a34`).
#[derive(Debug, Clone, PartialEq)]
pub struct QuadrosTonalidade {
    /// Matriz `BINS × quadros`, linha por bin.
    pub matriz: Vec<f64>,
    pub quadros: usize,
    /// `+0x1a34`.
    pub deslocamento: i32,
}

impl Tonalidade {
    /// Init (`0x14117ff60`) à taxa `fs` (taxa/8). Frequências
    /// `f0 = 27,5` e `fk = 2^(k·(1/12))·27,5` (`0x1435381e0`, `0x1435381c0`,
    /// `0x143538200`), k = 1..72; `Lk = (int)(2fs / (f(k+1) − fk))`. Bins 0–11:
    /// `(cos, sen)(2π·fk·t)`, `t = (j + 1)·(1/fs)`, `j < L0`; bins 12–71: quadrado
    /// complexo do bin `k − 12`, `re = fma(re, re, −im²)`, `im = (re + re)·im`.
    /// Depois cada núcleo é multiplicado pela Hann de `Lk`.
    ///
    /// LIMITAÇÃO: o original calcula `2^x` com um `pow` vetorial próprio
    /// (`0x1426086a0`); aqui é `f64::powf`, que pode diferir no último bit.
    pub fn novo(fs: f64) -> Option<Self> {
        let mut f = vec![27.5];
        f.extend((1..=BINS).map(|k| 2.0f64.powf(k as f64 * (1.0 / 12.0)) * 27.5));
        let dois_fs = fs + fs;
        let comprimentos: Vec<usize> = (0..BINS)
            .map(|k| usize::try_from((dois_fs / (f[k + 1] - f[k])) as i32).ok())
            .collect::<Option<_>>()?;
        let l0 = comprimentos[0];
        let inv = 1.0 / fs;
        let t: Vec<f64> = (1..=l0).map(|j| j as f64 * inv).collect();
        let dois_pi = 2.0 * std::f64::consts::PI;
        let mut nucleos: Vec<Vec<(f64, f64)>> = (0..12)
            .map(|k| {
                t.iter()
                    .map(|&tj| {
                        let a = dois_pi * f[k] * tj;
                        (a.cos(), a.sin())
                    })
                    .collect()
            })
            .collect();
        for k in 12..BINS {
            let q = nucleos[k - 12]
                .iter()
                .map(|&(re, im)| (re.mul_add(re, -(im * im)), (re + re) * im))
                .collect();
            nucleos.push(q);
        }
        for (k, n) in nucleos.iter_mut().enumerate() {
            let lk = comprimentos[k];
            n.truncate(lk);
            for (v, w) in n.iter_mut().zip(hann(lk)) {
                v.0 *= w;
                v.1 *= w;
            }
        }
        let pesos = (0..BINS)
            .map(|k| {
                if k == 0 {
                    1.0
                } else {
                    l0 as f64 / comprimentos[k] as f64
                }
            })
            .collect();
        Some(Self {
            comprimentos,
            pesos,
            nucleos,
            sobra: Vec::new(),
        })
    }

    /// Lote (`0x1411803f0`). `banda` = `+0xaa0` (taxa/8), `estado` =
    /// `+0xb40`. Sinal = sobra + banda; `quadros = (int)((len − L0)·(1/256))`.
    /// Quadro `m`, bin `k` em 23..72: produto do núcleo com o sinal a partir de
    /// `(int)(L0·0,5) − (int)(Lk·0,5) + 256m`, módulo `sqrt(fma(im, im, re²))`
    /// vezes o peso. Bins 0–22 ficam 0. Com `estado < 2`, guarda
    /// `x[256·quadros..]` como sobra. `deslocamento =
    /// ceil(fma(1/bpm, (p5 − 1)·0,234375, p4·(1/256000))·fs)` (`0x1435381b8`,
    /// `0x1435381b0`).
    ///
    /// ADAPTAÇÃO: janela fora do sinal devolve `None`; o original lê além dele.
    pub fn processar(
        &mut self,
        banda: &[f64],
        bpm: f64,
        p4: i32,
        p5: i32,
        fs: f64,
        estado: i32,
    ) -> Option<QuadrosTonalidade> {
        let deslocamento =
            ((1.0 / bpm).mul_add((p5 - 1) as f64 * 0.234_375, p4 as f64 * 3.906_25e-6) * fs).ceil()
                as i32;
        let mut x = std::mem::take(&mut self.sobra);
        x.extend_from_slice(banda);
        let l0 = self.comprimentos[0];
        let quadros = usize::try_from(((x.len() as i32 - l0 as i32) as f64 * 0.003_906_25) as i32)
            .unwrap_or(0);
        let mut matriz = vec![0.0; BINS * quadros];
        let centro = (l0 as f64 * 0.5) as i32;
        for m in 0..quadros {
            for k in PRIMEIRO_BIN..BINS {
                let lk = self.comprimentos[k];
                let ini = centro - (lk as f64 * 0.5) as i32 + (SALTO * m) as i32;
                let ini = usize::try_from(ini).ok()?;
                let seg = x.get(ini..ini + lk)?;
                let (re, im) = produto(seg, &self.nucleos[k]);
                matriz[k * quadros + m] = im.mul_add(im, re * re).sqrt() * self.pesos[k];
            }
        }
        if estado < 2 {
            self.sobra = x[(SALTO * quadros).min(x.len())..].to_vec();
        }
        Some(QuadrosTonalidade {
            matriz,
            quadros,
            deslocamento,
        })
    }
}

/// Produto interno complexo do laço de `0x141180660`: blocos de 4 com a
/// ordem `re += x0·k0`, `re = x1·k1 + re`, `re = fma(x2, k2, re)`,
/// `re = fma(x3, k3, re)` (idem para `im`); cauda `acc += x·k` sem fusão.
fn produto(x: &[f64], k: &[(f64, f64)]) -> (f64, f64) {
    let (mut re, mut im) = (0.0f64, 0.0f64);
    let mut i = 0;
    if x.len() > 3 {
        let blocos = (x.len() - 4) / 4 + 1;
        for _ in 0..blocos {
            re += x[i] * k[i].0;
            im += x[i] * k[i].1;
            re += x[i + 1] * k[i + 1].0;
            im += x[i + 1] * k[i + 1].1;
            re = x[i + 2].mul_add(k[i + 2].0, re);
            im = x[i + 2].mul_add(k[i + 2].1, im);
            re = x[i + 3].mul_add(k[i + 3].0, re);
            im = x[i + 3].mul_add(k[i + 3].1, im);
            i += 4;
        }
    }
    while i < x.len() {
        re += x[i] * k[i].0;
        im += x[i] * k[i].1;
        i += 1;
    }
    (re, im)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hann_simetrica_par_e_impar() {
        let w = hann(4);
        assert_eq!(w[0], w[3]);
        assert_eq!(w[1], w[2]);
        assert!((w[0] - (1.0 - (0.4 * std::f64::consts::PI).cos()) * 0.5).abs() < 1e-15);
        let w = hann(5);
        assert_eq!(w[0], w[4]);
        assert!((w[2] - 1.0).abs() < 1e-15);
    }

    #[test]
    fn tom_cai_no_bin_certo() {
        // A4 = 440 Hz = 27,5·2^4: bin 48.
        let fs = 5512.5;
        let mut t = Tonalidade::novo(fs).unwrap();
        assert_eq!(t.pesos[0], 1.0);
        let l0 = t.comprimentos[0];
        let n = l0 + SALTO * 4;
        let x: Vec<f64> = (0..n)
            .map(|i| (2.0 * std::f64::consts::PI * 440.0 * i as f64 / fs).sin())
            .collect();
        let q = t.processar(&x, 120.0, 0, 1, fs, 0).unwrap();
        assert_eq!(q.quadros, 4);
        assert_eq!(q.deslocamento, 0);
        let col: Vec<f64> = (0..BINS).map(|k| q.matriz[k * q.quadros]).collect();
        let max = (0..BINS)
            .max_by(|&a, &b| col[a].total_cmp(&col[b]))
            .unwrap();
        assert_eq!(max, 48);
        assert!(col[..PRIMEIRO_BIN].iter().all(|v| *v == 0.0));
        // Sobra: o que passa dos 4 quadros.
        assert_eq!(t.sobra.len(), n - SALTO * 4);
    }
}
