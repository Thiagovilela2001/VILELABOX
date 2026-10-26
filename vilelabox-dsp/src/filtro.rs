//! Butterworth de 8ª ordem com decimação do detector legado de BPM.
//!
//! Evidência: `spec/rekordbox/evidence/p4-1-analise-batida.md`, seção
//! "Filtros do detector legado". Endereços citados em cada item.
//!
//! ADAPTAÇÃO: `tan` e `cos` vêm da biblioteca padrão do Rust; o programa de
//! referência usa a CRT da Microsoft. Os resultados podem diferir no último
//! bit dos coeficientes.

/// Seção biquad em forma direta II transposta (`0x141166570`).
/// Coeficientes `[b0, b1, b2, a1, a2]` normalizados por `a0`.
#[derive(Debug, Clone, PartialEq)]
pub struct Biquad {
    pub c: [f64; 5],
    s1: f64,
    s2: f64,
}

impl Biquad {
    /// Transformada bilinear do protótipo
    /// `(b2·s² + b1·s + b0) / (a2·s² + a1·s + a0)` à taxa `fs`, na ordem de
    /// operações de `0x141165eb0`.
    #[allow(clippy::too_many_arguments)]
    pub fn bilinear(b2: f64, b1: f64, b0: f64, a2: f64, a1: f64, a0: f64, fs: f64) -> Self {
        let a2 = 4.0 * a2;
        let b2 = b2 * 4.0;
        let t = 1.0 / fs;
        let d = 1.0 / (t * (t * a0 + a1 + a1) + a2);
        let tb0 = t * b0 * t;
        let ta0 = t * a0 * t;
        let c0 = (t * (t * b0 + b1 + b1) + b2) * d;
        let c1 = ((tb0 - b2) + (tb0 - b2)) * d;
        let c2 = (-(t * (b1 + b1)) + tb0 + b2) * d;
        let c3 = ((ta0 - a2) + (ta0 - a2)) * d;
        let c4 = (-(t * (a1 + a1)) + ta0 + a2) * d;
        Self {
            c: [c0, c1, c2, c3, c4],
            s1: 0.0,
            s2: 0.0,
        }
    }

    /// Filtra `x` em `y` (mesmo comprimento), mantendo o estado entre blocos.
    pub fn processar(&mut self, x: &[f64], y: &mut [f64]) {
        let [b0, b1, b2, a1, a2] = self.c;
        for (xi, yi) in x.iter().zip(y.iter_mut()) {
            let v = xi * b0 + self.s1;
            self.s1 = (xi * b1 - v * a1) + self.s2;
            self.s2 = xi * b2 - v * a2;
            *yi = v;
        }
    }

    fn processar_no_lugar(&mut self, y: &mut [f64]) {
        let [b0, b1, b2, a1, a2] = self.c;
        for yi in y.iter_mut() {
            let xi = *yi;
            let v = xi * b0 + self.s1;
            self.s1 = (xi * b1 - v * a1) + self.s2;
            self.s2 = xi * b2 - v * a2;
            *yi = v;
        }
    }
}

/// Quatro biquads em cascata (Butterworth de 8ª ordem) seguidos de
/// decimação por `fator` (`0x141191270`, `0x14118ad30`, `0x14117cbb0`).
#[derive(Debug, Clone, PartialEq)]
pub struct Butterworth8 {
    pub secoes: [Biquad; 4],
    pub fator: usize,
}

/// `π/8` e `π/16` como gravados em `0x1435380a8` e `0x143538388`.
const PASSO_POLO: f64 = std::f64::consts::FRAC_PI_8;
const INICIO_POLO: f64 = std::f64::consts::FRAC_PI_8 / 2.0;

impl Butterworth8 {
    fn novo(corte_hz: f64, fs: f64, fator: i32, passa_alta: bool) -> Option<Self> {
        if fs.is_nan() || fs <= 0.0 || fator < 2 {
            return None;
        }
        // Pré-distorção `0x141165dc0`: 2fs · tan(ω / 2fs), ω = 2π · corte.
        let dois_fs = fs + fs;
        let w = (corte_hz * std::f64::consts::TAU / dois_fs).tan() * dois_fs;
        let w = if w.is_finite() { w } else { return None };
        let w2 = w * w;
        let secao = |i: i32| {
            let k = (i as f64 * PASSO_POLO + INICIO_POLO).cos();
            let a1 = (k + k) * w;
            if passa_alta {
                Biquad::bilinear(1.0, 0.0, 0.0, 1.0, a1, w2, fs)
            } else {
                Biquad::bilinear(0.0, 0.0, w2, 1.0, a1, w2, fs)
            }
        };
        Some(Self {
            secoes: [secao(0), secao(1), secao(2), secao(3)],
            fator: fator as usize,
        })
    }

    /// Passa-baixa (`0x141191270`): numerador `Ω²`.
    pub fn passa_baixa(corte_hz: f64, fs: f64, fator: i32) -> Option<Self> {
        Self::novo(corte_hz, fs, fator, false)
    }

    /// Passa-alta (`0x14118ad30`): numerador `s²`.
    pub fn passa_alta(corte_hz: f64, fs: f64, fator: i32) -> Option<Self> {
        Self::novo(corte_hz, fs, fator, true)
    }

    /// Filtra o bloco e devolve `n / fator` amostras: a de índice
    /// `i · fator + fator − 1` de cada grupo. O resto do bloco é descartado,
    /// como no original (`0x14117cbb0`).
    pub fn processar(&mut self, x: &[f64]) -> Vec<f64> {
        let mut y = vec![0.0; x.len()];
        let (primeira, demais) = self.secoes.split_first_mut().expect("4 seções");
        primeira.processar(x, &mut y);
        for s in demais {
            s.processar_no_lugar(&mut y);
        }
        let m = self.fator;
        (0..x.len() / m).map(|i| y[i * m + m - 1]).collect()
    }
}

/// Bandas do detector legado, a partir da taxa `fs` (`0x1411952f0`,
/// `0x141195ed0`): passada A = passa-baixa em `fs/8 · 0,4`, decimação 8;
/// passada B = passa-alta em `fs/2 · 0,55` (÷2), passa-baixa em `fs/8 · 0,4`
/// (÷8) e passa-baixa em `fs/32 · 0,4` (÷32).
pub fn bandas_passada_b(fs: f64) -> Option<[Butterworth8; 3]> {
    Some([
        Butterworth8::passa_alta(fs * 0.5 * 0.55, fs, 2)?,
        Butterworth8::passa_baixa(fs * 0.125 * 0.4, fs, 8)?,
        Butterworth8::passa_baixa(fs * 0.031_25 * 0.4, fs, 32)?,
    ])
}

/// Filtro da passada A (`0x1411952f0`). INFERIDO: decimação 8, igual ao
/// campo `+0x7c` usado no tamanho do buffer; o argumento não aparece na
/// decompilação.
pub fn filtro_passada_a(fs: f64) -> Option<Butterworth8> {
    Butterworth8::passa_baixa(fs * 0.125 * 0.4, fs, 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ganho_dc(c: &[f64; 5]) -> f64 {
        (c[0] + c[1] + c[2]) / (1.0 + c[3] + c[4])
    }

    #[test]
    fn coeficientes_tem_a_forma_de_butterworth() {
        let lp = Butterworth8::passa_baixa(2205.0, 44100.0, 8).unwrap();
        for s in &lp.secoes {
            assert_eq!(s.c[0], s.c[2], "numerador simétrico");
            assert!((ganho_dc(&s.c) - 1.0).abs() < 1e-9);
        }
        let hp = Butterworth8::passa_alta(12127.5, 44100.0, 2).unwrap();
        for s in &hp.secoes {
            assert!(ganho_dc(&s.c).abs() < 1e-12, "passa-alta bloqueia DC");
        }
        assert!(Butterworth8::passa_baixa(100.0, 0.0, 8).is_none());
        assert!(Butterworth8::passa_baixa(100.0, 44100.0, 1).is_none());
    }

    #[test]
    fn decima_pegando_a_ultima_amostra_de_cada_grupo() {
        let mut f = Butterworth8::passa_baixa(2205.0, 44100.0, 8).unwrap();
        let x = vec![1.0; 8 * 400 + 5];
        let y = f.processar(&x);
        // Resto de 5 amostras descartado.
        assert_eq!(y.len(), 400);
        // DC passa com ganho 1 depois do transitório.
        assert!((y[399] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn estado_continua_entre_blocos() {
        let x: Vec<f64> = (0..1024).map(|i| ((i * 37) % 101) as f64 - 50.0).collect();
        let mut inteiro = Butterworth8::passa_baixa(2205.0, 44100.0, 8).unwrap();
        let mut partes = inteiro.clone();
        let a = inteiro.processar(&x);
        let mut b = partes.processar(&x[..512]);
        b.extend(partes.processar(&x[512..]));
        assert_eq!(a, b);
    }

    #[test]
    fn constantes_batem_com_o_binario() {
        // Bits lidos de 0x1435380a8, 0x143538388 e 0x1454154f8.
        assert_eq!(PASSO_POLO.to_bits(), 0x3fd921fb54442d18);
        assert_eq!(INICIO_POLO.to_bits(), 0x3fc921fb54442d18);
        assert_eq!(std::f64::consts::TAU.to_bits(), 0x401921fb54442d18);
    }

    #[test]
    fn bandas_da_passada_b() {
        let [hp, lp8, lp32] = bandas_passada_b(44100.0).unwrap();
        assert_eq!((hp.fator, lp8.fator, lp32.fator), (2, 8, 32));
        assert!(filtro_passada_a(44100.0).is_some());
    }
}
