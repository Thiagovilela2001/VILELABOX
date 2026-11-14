//! Extratores da passada C (P4.1) que não cabem nos módulos de filtro,
//! tonalidade e espectro. Dossiê: seção "Passada C por bloco" de
//! `spec/rekordbox/evidence/p4-1-analise-batida.md`.

use crate::estagios::{maximo, PrimeiraOrdem};
use crate::filtro::{passa_baixa_2, pre_distorcao, Biquad, FiltrosBanda, FiltrosEnvelope};

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

/// Elemento `i` (base 0) ou `None` fora do vetor: onde o original leria fora
/// do buffer, o porte desiste.
fn em(v: &[f64], i: i32) -> Option<f64> {
    v.get(usize::try_from(i).ok()?).copied()
}

/// Fatia `v[ini .. ini + len]`, ou `None` fora do vetor.
fn fatia(v: &[f64], ini: i32, len: i32) -> Option<&[f64]> {
    let ini = usize::try_from(ini).ok()?;
    v.get(ini..ini.checked_add(usize::try_from(len).ok()?)?)
}

/// Primeiro máximo por `>` estrito e seu índice; vazio dá `(0, 0)`.
fn primeiro_maximo(x: &[f64]) -> (usize, f64) {
    let Some(&v0) = x.first() else {
        return (0, 0.0);
    };
    let (mut i, mut m) = (0, v0);
    for (k, &v) in x.iter().enumerate().skip(1) {
        if v > m {
            i = k;
        }
        if m < v {
            m = v;
        }
    }
    (i, m)
}

/// Primeiro mínimo (`0x1412c6820` → `0x14280a400`); vazio dá 0.
fn primeiro_minimo(x: &[f64]) -> f64 {
    x.iter()
        .copied()
        .reduce(|m, v| if v < m { v } else { m })
        .unwrap_or(0.0)
}

/// Busca de alinhamento de `0x14118d310` (7.2.19: `0x1412ec9d0`), conferida
/// no assembly. `b` = envelope `|200 Hz| → 10 Hz`, `u` = início em amostras
/// (base 1), `compassos` = `C >= 2`. Devolve o deslocamento em amostras.
///
/// 1. `seg = b[barI − 1 + u ..][..(C − 1)·barI]`, com `barI = (int)(4·batida)`.
/// 2. Até 8 candidatos. Antes de cada um, zera o máximo do anterior. Pega o
///    primeiro máximo `m` de `seg` (índice `p`, base 1) e
///    `c = ((int)(p / barra) + 1)·barI`.
/// 3. Segue se `b[c + u − 2] > 0,2·m` (`0x1457ff7a0`; fora do sinal vale o
///    último), `c < len(seg) − barI` e `b[c+u−2] <= b[c+u−1] <= b[c+u] <=
///    b[c+u+1]`. Com `aceita_descida` (`0x14118e450`), uma descida estrita
///    `b[c+u−2] > b[c+u−1] > b[c+u] > b[c+u+1]` também segue.
/// 4. Pico: na subida, primeira queda `b[c+u−1+r] < b[c+u−2+r]`, `r = 1..=w`
///    (`w = (int)(batida/4)`), com `pos = c + r`. Na descida, a busca vai
///    para trás: `b[c+u−3−r] < b[c+u−2−r]`, com `pos = c − r`. Nos dois
///    casos, `pv = b[pos + u − 2]`. Sem queda, `pos` e `pv` ficam os do
///    candidato anterior (0 no início).
/// 5. Segue se `pv > 0,595662·m` (`0x143826758`). Limiar `(m + mín)·0,5`, com
///    o mínimo de `b[c + u − 2 ..][..barI + 1]`.
/// 6. Volta de `pos` por até `(int)(1,25·batida)` amostras (`0x1438264e8`).
///    O vale é a primeira amostra `y <= anterior` e `y <` limiar. Depois, cada
///    amostra `<= 1,12·vale` (`0x143826280`) conta como quieta; uma maior
///    reinicia a busca do vale. Com `(int)(batida/16)` quietas: vale 0 dá
///    deslocamento 0. Se `pv/vale > 2,11` (`0x143826768`), o deslocamento é
///    `min((int)(c − j + 0,5), u)`, com `j` = posição do vale. Senão, vai
///    para o próximo candidato.
/// 7. Sem decisão nos 8 candidatos: 0.
// As negações reproduzem os saltos do `comisd`: com NaN, o original segue o outro ramo.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
pub fn deslocamento_de_alinhamento(
    b: &[f64],
    u: i32,
    batida: f64,
    compassos: i32,
    aceita_descida: bool,
) -> Option<i32> {
    let barra = batida * 4.0;
    let bar_i = barra as i32;
    let w = (batida * 0.25) as i32;
    let quietas = (batida * 0.25 * 0.25) as i32;
    let voltas = (batida * 1.25) as i32;
    let len_b = b.len() as i32;
    let len_seg = (compassos - 1) * bar_i;
    let mut seg = fatia(b, bar_i - 1 + u, len_seg)?.to_vec();
    let mut idx = [0i32; 8];
    let (mut pos, mut pv) = (0i32, 0.0f64);
    for k in 0..8 {
        if k > 0 && idx[k - 1] <= len_seg {
            // shortcut: índice 0 (seg vazio) escreveria em seg[−1] no original; aqui é ignorado.
            if let Some(v) = usize::try_from(idx[k - 1] - 1)
                .ok()
                .and_then(|i| seg.get_mut(i))
            {
                *v = 0.0;
            }
        }
        let m = if seg.is_empty() {
            0.0
        } else {
            let (i, m) = primeiro_maximo(&seg);
            idx[k] = i as i32 + 1;
            m
        };
        let c = if barra != 0.0 {
            (f64::from(idx[k]) / barra) as i32
        } else {
            0xffff
        };
        let c = (c + 1) * bar_i;
        let v0 = if c + u - 1 <= len_b {
            em(b, c + u - 2)?
        } else {
            em(b, len_b - 1)?
        };
        if !(v0 > m * 0.2) || c >= len_seg - bar_i {
            continue;
        }
        let (x0, x1, x2) = (em(b, c + u - 1)?, em(b, c + u)?, em(b, c + u + 1)?);
        let sobe = if v0 > x0 {
            if !aceita_descida || !(x0 > x1) || !(x1 > x2) {
                continue;
            }
            false
        } else {
            if x0 > x1 || x1 > x2 {
                continue;
            }
            true
        };
        for r in 1..=w {
            let d = if sobe {
                em(b, c + u - 1 + r)? - em(b, c + u - 2 + r)?
            } else {
                em(b, c + u - 3 - r)? - em(b, c + u - 2 - r)?
            };
            if !(d >= 0.0) {
                pos = if sobe { c + r } else { c - r };
                pv = em(b, pos + u - 2)?;
                break;
            }
        }
        if !(pv > m * 0.595_662) || voltas <= 0 {
            continue;
        }
        // O original empresta esta janela antes da subida e só a lê aqui.
        let limiar = (m + primeiro_minimo(fatia(b, c + u - 2, bar_i + 1)?)) * 0.5;
        let (mut achou, mut quietos, mut vale, mut j) = (false, 0, m, 0);
        for t in 1..=voltas {
            let y = em(b, pos - t + u - 2)?;
            if !achou {
                if y > em(b, pos - t + u - 3)? || y >= limiar {
                    continue;
                }
                achou = true;
                j = pos - t;
                vale = y;
            } else if y > vale * 1.12 {
                achou = false;
                quietos = 0;
            } else {
                quietos += 1;
                if quietos >= quietas {
                    if !(vale != 0.0) {
                        return Some(0);
                    }
                    if pv / vale > 2.11 {
                        return Some((((c as f64) - (j as f64)) + 0.5) as i32).map(|d| d.min(u));
                    }
                    break;
                }
            }
        }
    }
    Some(0)
}

/// Parâmetros de um lote, iguais em `0x14118d310` e `0x14118e450`.
///
/// `batida = (60/bpm)·fs`, `w = (int)(batida/4)`, `barI = (int)(4·batida)`,
/// `u = (int)fma(batida, s − 1, (int)fma(ms·0,001, fs, 0,5) + 1)`.
/// `limite = g[n−1] − 1 + (int)batida`, ou `g[n−1]` se passar de `len`.
/// `C = (n − s + 1)/4`, um a menos se `barI·C − 1 + u > len`. A referência
/// volta a 1e−4 com `lote <= 1`.
struct Lote {
    len: i32,
    batida: f64,
    quarto: f64,
    w: i32,
    u: i32,
    limite: i32,
    compassos: i32,
    referencia: f64,
}

impl Lote {
    #[allow(clippy::too_many_arguments)]
    fn novo(
        len: usize,
        bpm: f64,
        ms: i32,
        s: i32,
        fs: f64,
        lote: i32,
        grade: &[i32],
        n: usize,
        referencia: f64,
    ) -> Option<Self> {
        let len = len as i32;
        let batida = (60.0 / bpm) * fs;
        let quarto = batida * 0.25;
        let bar_i = (batida * 4.0) as i32;
        let k0 = (f64::from(ms) * 0.001).mul_add(fs, 0.5) as i32 + 1;
        let u = batida.mul_add(f64::from(s - 1), f64::from(k0)) as i32;
        let gl = *grade.get(n.checked_sub(1)?)?;
        let limite = match gl - 1 + batida as i32 {
            v if v <= len => v,
            _ => gl,
        };
        let mut compassos = (n as i32 - s + 1) / 4;
        if bar_i * compassos - 1 + u > len {
            compassos -= 1;
        }
        Some(Self {
            len,
            batida,
            quarto,
            w: quarto as i32,
            u,
            limite,
            compassos,
            referencia: if lote > 1 { referencia } else { 1e-4 },
        })
    }

    /// Recorte da diferença `d` pelo deslocamento `desl`. Se `u > desl` e
    /// `limite − desl <= len`, devolve `limite − u + 1` amostras a partir de
    /// `u − desl − 1`. Se `u < desl` e `limite − desl − 1 <= len`, põe
    /// `desl − u + 1` zeros antes do início da diferença; o resto do buffer
    /// fica como estava. Nos outros casos, devolve a diferença inteira.
    fn recortar(&self, d: Vec<f64>, desl: i32) -> Option<Vec<f64>> {
        let (u, limite, len) = (self.u, self.limite, self.len);
        if u > desl && limite - desl <= len {
            return Some(fatia(&d, u - desl - 1, limite - u + 1)?.to_vec());
        }
        if limite - desl - 1 <= len && u < desl {
            let inicio = fatia(&d, 0, limite - desl - 1)?.to_vec();
            let mut v = d;
            let novo_len = usize::try_from(limite - u).ok()?;
            if v.len() < novo_len {
                v.resize(novo_len, 0.0);
            }
            let zeros = (desl - u + 1) as usize;
            v[..zeros].fill(0.0);
            v[zeros..zeros + inicio.len()].copy_from_slice(&inicio);
            return Some(v);
        }
        Some(d)
    }

    /// Compasso `k`, batida `i` (número `s + 4k + i`; para quando passa do
    /// tamanho da grade) e avo `e`: máximo de `W[ini ..][..w]`, com
    /// `ini = (int)((e·batida/4 + (g − u)) + 0,5)`. Também para quando
    /// `ini + w > len(W)`. Em cada rodada, o primeiro máximo dos 16 marca 1 e
    /// é zerado se for `> 1e−4`, `> fator·`(máximo da 1ª rodada) e
    /// `> 0,178·ref` (`0x143826750`). Depois, `ref = max(máximo da 1ª rodada,
    /// ref)`. Devolve as marcas e a referência final.
    fn marcar(
        &self,
        sinal: &[f64],
        grade: &[i32],
        s: i32,
        rodadas: usize,
        fator: f64,
    ) -> Option<(Vec<u8>, f64)> {
        let len_w = sinal.len() as i32;
        let mut referencia = self.referencia;
        let mut marcas = vec![0u8; self.compassos as usize * 16];
        'compassos: for k in 0..self.compassos {
            let mut v = [0.0f64; 16];
            for i in 0..4 {
                let numero = s + 4 * k + i;
                if numero > grade.len() as i32 {
                    break 'compassos;
                }
                let g = *grade.get(usize::try_from(numero - 1).ok()?)? - self.u;
                for e in 0..4 {
                    let ini = ((f64::from(e) * self.quarto + f64::from(g)) + 0.5) as i32;
                    if ini + self.w > len_w {
                        break 'compassos;
                    }
                    v[(i * 4 + e) as usize] = maximo(fatia(sinal, ini, self.w)?);
                }
            }
            let mut primeiro = 0.0;
            for rodada in 0..rodadas {
                let (i, m) = primeiro_maximo(&v);
                if rodada == 0 {
                    primeiro = m;
                }
                if m > 1e-4 && m > primeiro * fator && m > referencia * 0.178 {
                    marcas[k as usize * 16 + i] = 1;
                    v[i] = 0.0;
                }
            }
            referencia = if primeiro > referencia {
                primeiro
            } else {
                referencia
            };
        }
        Some((marcas, referencia))
    }
}

/// Marcas por 16 avos de compasso da banda de 110 a 200 Hz (`0x14118d310`;
/// 7.2.19: `0x1412ec9d0`), conferido no assembly. Chamada `(obj, +0xaa0,
/// bpm, ms, s = +0xb98, fs = taxa/8, lote = +0xb80, grade +0x1140, n)`.
/// Filtros de `0x14118cf70` (`FiltrosBanda`). A init da B grava o estado
/// inicial `(0, 0, 1e−4)`.
#[derive(Debug, Clone)]
pub struct MarcasPorDezesseisAvos {
    filtros: FiltrosBanda,
    /// Última amostra do envelope de 110 Hz antes da diferença (`+0x1930`).
    pub anterior: f64,
    /// Deslocamento achado no último lote (`+0x1938`).
    pub deslocamento: i32,
    /// Pico de referência (`+0x1940`).
    pub referencia: f64,
}

impl MarcasPorDezesseisAvos {
    /// `None` para `fs <= 0`.
    pub fn novo(fs: f64) -> Option<Self> {
        Some(Self {
            filtros: FiltrosBanda::novo(fs)?,
            anterior: 0.0,
            deslocamento: 0,
            referencia: 1e-4,
        })
    }

    /// Um byte por 16 avo dos `C` compassos do lote (`+0x1948`), com os
    /// parâmetros de `Lote`.
    ///
    /// 1. O deslocamento guardado é zerado na entrada. Banda → 200 Hz (4ª ou
    ///    2ª ordem; outra taxa devolve `None`), `b = |·| → 10 Hz`.
    /// 2. `C <= 0`: devolve `[0]`. Guarda `anterior = b[len − 1]` (o envelope
    ///    de 200 Hz, não o de 110 Hz) e restaura o deslocamento. O passa-alta
    ///    não roda.
    /// 3. `C >= 2`: `deslocamento_de_alinhamento(b, …, false)`; `C = 1`: 0. O
    ///    valor é guardado, mas com `lote > 1` o recorte usa o do lote
    ///    anterior.
    /// 4. Saída do 200 Hz → passa-alta de 110 Hz → `|·|` → 10 Hz → diferença
    ///    com `anterior` → `Lote::recortar` → `Lote::marcar` com 12 rodadas e
    ///    fator 0,5.
    ///
    /// Leitura fora do buffer no original (índices negativos) devolve `None`.
    #[allow(clippy::too_many_arguments)]
    pub fn processar(
        &mut self,
        banda: &[f64],
        bpm: f64,
        ms: i32,
        s: i32,
        fs: f64,
        lote: i32,
        grade: &[i32],
        n: usize,
    ) -> Option<Vec<u8>> {
        let p = Lote::novo(banda.len(), bpm, ms, s, fs, lote, grade, n, self.referencia)?;
        let deslocamento_anterior = self.deslocamento;
        let anterior = self.anterior;
        self.deslocamento = 0;

        let f = &mut self.filtros;
        let (primeira, demais) = f.base.passa_baixa_200.split_first_mut()?;
        let mut a = vec![0.0; banda.len()];
        primeira.processar(banda, &mut a);
        for sec in demais {
            sec.processar_no_lugar(&mut a);
        }
        let mut b: Vec<f64> = a.iter().map(|v| v.abs()).collect();
        f.base.passa_baixa_10.processar_no_lugar(&mut b);

        if p.compassos <= 0 {
            self.anterior = em(&b, p.len - 1)?;
            self.referencia = p.referencia;
            self.deslocamento = deslocamento_anterior;
            return Some(vec![0]);
        }
        let novo = if p.compassos >= 2 {
            deslocamento_de_alinhamento(&b, p.u, p.batida, p.compassos, false)?
        } else {
            0
        };
        self.deslocamento = novo;
        let desl = if lote > 1 {
            deslocamento_anterior
        } else {
            novo
        };

        for sec in &mut f.passa_alta_110 {
            sec.processar_no_lugar(&mut a);
        }
        let mut e2: Vec<f64> = a.iter().map(|v| v.abs()).collect();
        f.passa_baixa_10_b.processar_no_lugar(&mut e2);
        self.anterior = em(&e2, p.len - 1)?;
        let sinal = p.recortar(crate::filtro::diferenca(&e2, anterior), desl)?;
        let (marcas, referencia) = p.marcar(&sinal, grade, s, 12, 0.5)?;
        self.referencia = referencia;
        Some(marcas)
    }
}

/// Marcas por 16 avos de compasso do envelope abaixo de 200 Hz
/// (`0x14118e450`; 7.2.19: `0x1412edb10`), conferido no assembly. É a gêmea
/// de `MarcasPorDezesseisAvos`, com a mesma chamada. Filtros de `0x14118e120`
/// (`FiltrosEnvelope`, em `+0x15a0`…`+0x1688`); saída em `+0x16d8`, copiada
/// a `+0xd80`. Estado em `+0x16c0`/`+0x16c8`/`+0x16d0`.
#[derive(Debug, Clone)]
pub struct MarcasPorDezesseisAvosGraves {
    filtros: FiltrosEnvelope,
    /// Última amostra do envelope antes da diferença (`+0x16c0`).
    pub anterior: f64,
    /// Deslocamento achado no último lote (`+0x16c8`).
    pub deslocamento: i32,
    /// Pico de referência (`+0x16d0`).
    pub referencia: f64,
}

impl MarcasPorDezesseisAvosGraves {
    /// Estado inicial `(0, 0, 1e−4)`, passado pela init da B à
    /// `0x14118e120` (7.2.19: `0x1412ed7e0`). `None` para `fs <= 0`.
    pub fn novo(fs: f64) -> Option<Self> {
        Some(Self {
            filtros: FiltrosEnvelope::novo(fs)?,
            anterior: 0.0,
            deslocamento: 0,
            referencia: 1e-4,
        })
    }

    /// Mesmo fluxo de `MarcasPorDezesseisAvos::processar`, com quatro
    /// diferenças:
    /// - cadeia única (200 Hz → `|·|` → 10 Hz), usada tanto na busca quanto
    ///   na diferença;
    /// - o deslocamento guardado não é zerado na entrada;
    /// - a busca aceita também uma descida (`aceita_descida = true`);
    /// - a marcação tem 6 rodadas e fator 0,596 (`0x143826760`).
    #[allow(clippy::too_many_arguments)]
    pub fn processar(
        &mut self,
        banda: &[f64],
        bpm: f64,
        ms: i32,
        s: i32,
        fs: f64,
        lote: i32,
        grade: &[i32],
        n: usize,
    ) -> Option<Vec<u8>> {
        let p = Lote::novo(banda.len(), bpm, ms, s, fs, lote, grade, n, self.referencia)?;
        let deslocamento_anterior = self.deslocamento;
        let anterior = self.anterior;
        let b = self.filtros.envelope(banda)?;
        if p.compassos <= 0 {
            self.anterior = em(&b, p.len - 1)?;
            self.referencia = p.referencia;
            self.deslocamento = deslocamento_anterior;
            return Some(vec![0]);
        }
        let novo = if p.compassos >= 2 {
            deslocamento_de_alinhamento(&b, p.u, p.batida, p.compassos, true)?
        } else {
            0
        };
        self.deslocamento = novo;
        let desl = if lote > 1 {
            deslocamento_anterior
        } else {
            novo
        };
        self.anterior = em(&b, p.len - 1)?;
        let sinal = p.recortar(crate::filtro::diferenca(&b, anterior), desl)?;
        let (marcas, referencia) = p.marcar(&sinal, grade, s, 6, 0.596)?;
        self.referencia = referencia;
        Some(marcas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Envelope feito à mão para a busca: batida 16 (barI 64, w 4, 1,25·batida
    /// = 20 voltas, 1 quieta), base 0,1, subida no início do 2º compasso de
    /// `b` e vale 3 amostras antes dela. `o` desloca tudo (u − 1).
    fn envelope_de_busca(o: usize, antes_do_pico: [f64; 3]) -> Vec<f64> {
        let mut b = vec![0.1; 320];
        b[50 + o..=59 + o].fill(0.05);
        b[60 + o] = 0.05;
        b[61 + o..=63 + o].copy_from_slice(&antes_do_pico);
        b[64 + o..=67 + o].copy_from_slice(&[0.6, 0.9, 1.0, 0.8]);
        b
    }

    #[test]
    fn busca_acha_o_vale_antes_da_subida() {
        // Pico b[66]; primeira queda r = 4 → pos = 68, pv = b[67] = 0,8.
        // Vale em b[60] (j = 61): desl = c − j = 64 − 61 = 3, limitado a u.
        let b = envelope_de_busca(4, [0.2, 0.25, 0.3]);
        assert_eq!(deslocamento_de_alinhamento(&b, 5, 16.0, 4, false), Some(3));
        let b = envelope_de_busca(0, [0.2, 0.25, 0.3]);
        assert_eq!(deslocamento_de_alinhamento(&b, 1, 16.0, 4, false), Some(1));
    }

    #[test]
    fn busca_sem_contraste_nao_desloca() {
        // Vale 0,4 (abaixo do limiar 0,55): 0,8/0,4 = 2 não passa de 2,11.
        let b = envelope_de_busca(4, [0.4, 0.4, 0.4]);
        assert_eq!(deslocamento_de_alinhamento(&b, 5, 16.0, 4, false), Some(0));
        // Sinal plano: nunca há queda, pv = 0.
        assert_eq!(
            deslocamento_de_alinhamento(&[0.1; 320], 5, 16.0, 4, false),
            Some(0)
        );
        // C = 3: c = 64 não fica antes de len(seg) − barI = 64.
        let b = envelope_de_busca(4, [0.2, 0.25, 0.3]);
        assert_eq!(deslocamento_de_alinhamento(&b, 5, 16.0, 3, false), Some(0));
    }

    #[test]
    fn busca_da_gemea_aceita_descida() {
        // Descida b[63] > b[64] > b[65] > b[66] no início do 2º compasso; o
        // máximo do seg (1,0) fica em b[100]. Pico para trás: pos = 63,
        // pv = b[62] = 0,9; vale em b[58] (j = 59): desl = 64 − 59 = 5.
        let o = 7;
        let mut b = vec![0.1; 320];
        b[50 + o..=58 + o].fill(0.05);
        b[59 + o..=66 + o].copy_from_slice(&[0.3, 0.7, 0.8, 0.9, 0.85, 0.6, 0.4, 0.2]);
        b[100 + o] = 1.0;
        assert_eq!(deslocamento_de_alinhamento(&b, 8, 16.0, 4, true), Some(5));
        assert_eq!(deslocamento_de_alinhamento(&b, 8, 16.0, 4, false), Some(0));
    }

    /// Rajadas de 150 Hz e 50 ms no início de cada batida, a 120 BPM e
    /// taxa/8 de 44,1 kHz; grade como a da passada B com `ms = 0`.
    fn rajadas(batidas: usize) -> (Vec<f64>, Vec<i32>, f64, f64) {
        let fs = 44100.0 / 8.0;
        let bpm = 120.0;
        let passo = (60.0 / bpm) * fs;
        let grade: Vec<i32> = (0..=batidas)
            .map(|i| (i as f64 * passo + 1.5) as i32)
            .collect();
        let len = (batidas as f64 * passo) as usize + 2000;
        let mut x = vec![0.0; len];
        for &g in &grade[..batidas] {
            let i0 = (g - 1) as usize;
            for (k, v) in x[i0..i0 + 275].iter_mut().enumerate() {
                *v = (std::f64::consts::TAU * 150.0 * k as f64 / fs).sin();
            }
        }
        (x, grade, bpm, fs)
    }

    #[test]
    fn marca_o_primeiro_avo_de_cada_batida() {
        let (x, grade, bpm, fs) = rajadas(9);
        let mut m = MarcasPorDezesseisAvos::novo(fs).unwrap();
        let marcas = m.processar(&x, bpm, 0, 1, fs, 1, &grade, 9).unwrap();
        // C = (9 − 1 + 1)/4 = 2 compassos.
        let batida = [1, 0, 0, 0];
        assert_eq!(marcas, batida.repeat(8));
        assert!(m.referencia > 1e-4);
        assert!((0..=1).contains(&m.deslocamento));
    }

    #[test]
    fn gemea_marca_o_primeiro_avo_de_cada_batida() {
        let (x, grade, bpm, fs) = rajadas(9);
        let mut m = MarcasPorDezesseisAvosGraves::novo(fs).unwrap();
        let marcas = m.processar(&x, bpm, 0, 1, fs, 1, &grade, 9).unwrap();
        assert_eq!(marcas, [1, 0, 0, 0].repeat(8));
        assert!(m.referencia > 1e-4);
        // Lote curto: um zero, deslocamento mantido (não é zerado na entrada).
        m.deslocamento = 3;
        let curto = m.processar(&x, bpm, 0, 1, fs, 2, &grade, 3).unwrap();
        assert_eq!((curto, m.deslocamento), (vec![0], 3));
    }

    #[test]
    fn lote_curto_devolve_um_zero_e_guarda_o_envelope() {
        let (x, grade, bpm, fs) = rajadas(9);
        let mut m = MarcasPorDezesseisAvos::novo(fs).unwrap();
        m.deslocamento = 7;
        // n − s + 1 = 3 batidas: nenhum compasso.
        let marcas = m.processar(&x, bpm, 0, 1, fs, 2, &grade, 3).unwrap();
        assert_eq!(marcas, vec![0]);
        assert_eq!(m.deslocamento, 7);
        assert_eq!(m.referencia, 1e-4);
        assert!(m.anterior.is_finite());
        // Taxa sem ordem de 200 Hz na tabela.
        let mut m = MarcasPorDezesseisAvos::novo(12000.0).unwrap();
        assert!(m.processar(&x, bpm, 0, 1, 12000.0, 1, &grade, 9).is_none());
    }

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
