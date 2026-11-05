//! Extratores da passada C (P4.1) que não cabem nos módulos de filtro,
//! tonalidade e espectro. Dossiê: seção "Passada C por bloco" de
//! `spec/rekordbox/evidence/p4-1-analise-batida.md`.

use crate::estagios::{maximo, PrimeiraOrdem};
use crate::filtro::{passa_baixa_2, pre_distorcao, Biquad};

/// `0x1434e55e8` (2π·100) e `0x1435380c8` (2π·5).
const W_100HZ: f64 = 628.318_530_717_958_7;
const W_5HZ: f64 = 31.415_926_535_897_93;

/// Saídas de `PicosGraves::processar`.
#[derive(Debug, Clone, PartialEq)]
pub struct PicosGraves {
    /// Pico por batida (`+0x11f8`, copiado a `+0xc00`).
    pub batidas: Vec<f64>,
    /// Pico por quarto de batida (`+0x1218`, copiado a `+0xc40`).
    pub quartos: Vec<f64>,
    /// Marca de subida/queda por batida (`+0x1238`, copiada a `+0xc20`).
    pub marcas: Vec<f64>,
}

/// Picos de graves por batida e por quarto (`0x141188050`, passada C),
/// conferido no assembly. Banda de taxa/32 (`+0xac0`) → passa-baixa de 2ª
/// ordem de 100 Hz (`+0x1188`) → valor absoluto → 1ª ordem de 5 Hz
/// (`+0x11c0`), projetados em `0x141187f60` e mantidos entre lotes.
#[derive(Debug, Clone)]
pub struct FiltroGraves {
    lp100: Biquad,
    lp5: PrimeiraOrdem,
}

impl FiltroGraves {
    pub fn novo(fs: f64) -> Self {
        let o5 = pre_distorcao(W_5HZ, fs);
        Self {
            lp100: passa_baixa_2(W_100HZ, fs),
            lp5: PrimeiraOrdem::bilinear(0.0, o5, 1.0, o5, fs),
        }
    }

    /// `grade` = `+0x1160` (as `n` primeiras entram), `anterior` = último pico
    /// de batida do lote anterior (`+0xb88`, 0 no início).
    ///
    /// - Grade local: `g[..n]` mais `g[n − 1] + (int)((60/bpm)·fs)`, sem limite
    ///   no comprimento.
    /// - Batida `i`: máximo de `env[g[i] − 1 .. g[i + 1] − 1]` se
    ///   `0 < g[i] <= len` e `g[i + 1] − 1 <= len`; senão 0.
    /// - Quarto `k` da batida: `q = (int)fma(g' − g, 0,25, 0,5)`
    ///   (`0x145415208`), de `g + k·q` até `g + (k + 1)·q − 1` (o quarto
    ///   `k = 3` vai até `g' − 1`), com o mesmo teste de limites.
    /// - Marca da batida `j` contra o pico anterior `p` (na primeira, o do lote
    ///   anterior, ignorado se for 0): 0,5 se `v < p` e `2v < p`; 1,0 se
    ///   `v > p·1,77` (`0x143538350`); depois × 0,8 (`0x145415338`) se `v` e
    ///   `p` forem `<= 0,09` (`0x143538238`).
    ///
    /// Segmento de comprimento negativo faz o original falhar: `None`.
    pub fn processar(
        &mut self,
        sinal: &[f64],
        grade: &[i32],
        n: usize,
        bpm: f64,
        fs: f64,
        anterior: f64,
    ) -> Option<PicosGraves> {
        let mut env = vec![0.0; sinal.len()];
        self.lp100.processar(sinal, &mut env);
        for v in env.iter_mut() {
            *v = v.abs();
        }
        self.lp5.processar_no_lugar(&mut env);
        if n == 0 {
            return Some(PicosGraves {
                batidas: Vec::new(),
                quartos: Vec::new(),
                marcas: Vec::new(),
            });
        }
        let len = env.len() as i32;
        let mut g = grade[..n].to_vec();
        g.push(g[n - 1] + ((60.0 / bpm) * fs) as i32);
        let pico = |a: i32, b: i32| -> Option<f64> {
            if a < 1 || a > len || b - 1 > len {
                return Some(0.0);
            }
            let ini = (a - 1) as usize;
            let fim = ini.checked_add(usize::try_from(b - a).ok()?)?;
            Some(maximo(&env[ini..fim]))
        };
        let batidas = g
            .windows(2)
            .map(|w| pico(w[0], w[1]))
            .collect::<Option<Vec<_>>>()?;
        let mut quartos = Vec::with_capacity(n * 4);
        for w in g.windows(2) {
            let (a, b) = (w[0], w[1]);
            let q = ((b - a) as f64).mul_add(0.25, 0.5) as i32;
            for k in 0..4 {
                let fim = if k == 3 { b } else { a + (k + 1) * q };
                quartos.push(pico(a + k * q, fim)?);
            }
        }
        let marca = |v: f64, p: f64| {
            let mut m = 0.0;
            if v <= p {
                if v < p && v + v < p {
                    m = 0.5;
                }
            } else if p * 1.77 < v {
                m = 1.0;
            }
            if v <= 0.09 && p <= 0.09 {
                m *= 0.8;
            }
            m
        };
        let mut marcas = vec![0.0; n];
        if anterior != 0.0 {
            marcas[0] = marca(batidas[0], anterior);
        }
        for j in 1..n {
            marcas[j] = marca(batidas[j], batidas[j - 1]);
        }
        Some(PicosGraves {
            batidas,
            quartos,
            marcas,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picos_graves_marcam_subida_e_queda() {
        let fs = 1378.125;
        // Batidas de 400 amostras; amplitude 0,2 / 1,0 / 0,3 em 60 Hz.
        let amp = [0.2, 1.0, 0.3];
        let sinal: Vec<f64> = (0..1200)
            .map(|i| amp[i / 400] * (2.0 * std::f64::consts::PI * 60.0 * i as f64 / fs).sin())
            .collect();
        let mut f = FiltroGraves::novo(fs);
        let bpm = 60.0 * fs / 400.0;
        let r = f
            .processar(&sinal, &[1, 401, 801], 3, bpm, fs, 0.0)
            .unwrap();
        assert_eq!(r.batidas.len(), 3);
        assert_eq!(r.quartos.len(), 12);
        // Sem anterior a 1ª marca é 0; sobe > 1,77× → 1. A 3ª não cai à
        // metade: o filtro de 5 Hz ainda carrega o pico da batida 2.
        assert_eq!(r.marcas[..2], [0.0, 1.0]);
        assert!(r.batidas[2] * 2.0 >= r.batidas[1] && r.marcas[2] == 0.0);
        assert!(r.quartos[4..8].iter().all(|v| *v <= r.batidas[1]));
    }

    #[test]
    fn marca_e_atenuada_abaixo_do_piso() {
        let mut f = FiltroGraves::novo(1378.125);
        let r = f
            .processar(
                &vec![0.0; 1200],
                &[1, 401, 801],
                3,
                206.71875,
                1378.125,
                0.05,
            )
            .unwrap();
        // 0 < 0,05/2 → 0,5, × 0,8 abaixo do piso; depois 0 contra 0 → 0.
        assert_eq!(r.marcas, vec![0.4, 0.0, 0.0]);
        assert!(f
            .processar(&[0.0; 10], &[5, 2], 2, 120.0, 100.0, 0.0)
            .is_none());
    }
}
