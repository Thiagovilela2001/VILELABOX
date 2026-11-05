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

/// Ataques contados por batida (`0x14117e850`, passada C), conferido no
/// assembly. Chamada `(obj, +0xa80, bpm, _, fs = taxa/2, grade +0x1120, n)`;
/// saída `+0x1978` (`n` bytes, copiada a `+0xdc0`).
///
/// 1. `x = |banda|` (cópia, sem filtro).
/// 2. Dois seguidores, rápido `r` e lento `l`, partindo de `ini` (`+0x1968`,
///    `+0x1970`, ambos 0 em `0x14117e830`). Coeficiente por amostra: rápido
///    `2000/fs` se `r <= x`, senão `20/fs`; lento `1000/fs` se `l <= x`,
///    senão `13,33…/fs` (`0x143538188`, `0x1454155b0`, `0x1454157b8`,
///    `0x143537b30`). Nas amostras dos blocos de 4 do laço desenrolado,
///    `y = fma(a, x, fma(−y, a, y))`; nas `len mod 4` finais,
///    `y = (1 − a)·y + x·a` sem fusão.
/// 3. Ataque na posição `i + 1` quando `r > l` e `r > 0,06` (`0x143538178`).
/// 4. Mantém o ataque se `(double)(pos − último_mantido) > (13,33…/bpm)·fs`
///    (`0x143538180`; o último começa em 0).
/// 5. Conta (`u8`, com volta) por batida `j`: `g[j] <= pos < g[j + 1]`, e a
///    última batida recebe todo `pos >= g[n − 1]`.
pub fn ataques_por_batida(
    banda: &[f64],
    grade: &[i32],
    n: usize,
    bpm: f64,
    fs: f64,
    ini: (f64, f64),
) -> Vec<u8> {
    let inv = 1.0 / fs;
    let (sobe_r, desce_r) = (inv * 2000.0, inv * 20.0);
    let (sobe_l, desce_l) = (inv * 1000.0, inv * 13.333_333_333_333_334);
    let minimo = (13.333_333_333_333_332 / bpm) * fs;
    let len = banda.len();
    let fundidas = if len >= 4 { len - len % 4 } else { 0 };
    let (mut r, mut l) = ini;
    let mut ataques = Vec::new();
    for (i, v) in banda.iter().enumerate() {
        let x = v.abs();
        let ar = if r <= x { sobe_r } else { desce_r };
        let al = if l <= x { sobe_l } else { desce_l };
        if i < fundidas {
            r = ar.mul_add(x, (-r).mul_add(ar, r));
            l = al.mul_add(x, (-l).mul_add(al, l));
        } else {
            r = (1.0 - ar) * r + x * ar;
            l = (1.0 - al) * l + x * al;
        }
        if l < r && 0.06 < r {
            ataques.push(i as i32 + 1);
        }
    }
    let mut ultimo = 0;
    let mantidos: Vec<i32> = ataques
        .into_iter()
        .filter(|&p| {
            let ok = minimo < f64::from(p - ultimo);
            if ok {
                ultimo = p;
            }
            ok
        })
        .collect();
    let mut cont = vec![0u8; n];
    if n == 0 {
        return cont;
    }
    for p in mantidos {
        for j in 0..n - 1 {
            if grade[j] <= p && p < grade[j + 1] {
                cont[j] = cont[j].wrapping_add(1);
            }
        }
        if grade[n - 1] <= p {
            cont[n - 1] = cont[n - 1].wrapping_add(1);
        }
    }
    cont
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ataques_caem_nas_batidas_certas() {
        let fs = 22050.0;
        // Cliques de 1 ms a cada 0,5 s (120 BPM), começando em 0,1 s.
        let mut x = vec![0.0; 44100];
        for k in 0..4 {
            let i0 = 2205 + k * 11025;
            x[i0..i0 + 22].fill(1.0);
        }
        let grade = [1, 11026, 22051, 33076];
        let c = ataques_por_batida(&x, &grade, 4, 120.0, fs, (0.0, 0.0));
        assert_eq!(c, vec![1, 1, 1, 1]);
        // Silêncio: nada.
        assert_eq!(
            ataques_por_batida(&vec![0.0; 1000], &grade, 4, 120.0, fs, (0.0, 0.0)),
            vec![0; 4]
        );
    }

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
