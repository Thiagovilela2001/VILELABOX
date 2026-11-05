//! Estágios do fechamento da passada A do detector legado de BPM.
//!
//! Evidência: `spec/rekordbox/evidence/p4-1-analise-batida.md`, seção
//! "Estágios da passada A". Ordem das operações em ponto flutuante conferida
//! no assembly, para reproduzir o arredondamento.

use crate::filtro::Biquad;

/// Soma dos quadrados (`0x1411671d0`): blocos de 8 com FMA em dois
/// acumuladores de 4 posições, redução `(s2 + s3) + (s0 + s1)` com
/// `s = acc2 + acc1`, e cauda escalar `acc + x·x` sem fusão.
pub fn energia(x: &[f64]) -> f64 {
    let blocos = x.len() / 8 * 8;
    let mut acc = 0.0;
    if blocos > 0 {
        let (mut a1, mut a2) = ([0.0f64; 4], [0.0f64; 4]);
        for b in x[..blocos].chunks_exact(8) {
            for l in 0..4 {
                a1[l] = b[l].mul_add(b[l], a1[l]);
                a2[l] = b[l + 4].mul_add(b[l + 4], a2[l]);
            }
        }
        let s: Vec<f64> = (0..4).map(|l| a2[l] + a1[l]).collect();
        acc = (s[2] + s[3]) + (s[0] + s[1]);
    }
    for &v in &x[blocos..] {
        acc += v * v;
    }
    acc
}

/// Filtro de 1ª ordem em forma direta II transposta (`0x141166430`),
/// projetado por bilinear de `(b1·s + b0) / (a1·s + a0)` (`0x141165e30`).
#[derive(Debug, Clone, PartialEq)]
pub struct PrimeiraOrdem {
    pub c: [f64; 3],
    s: f64,
}

impl PrimeiraOrdem {
    pub fn bilinear(b1: f64, b0: f64, a1: f64, a0: f64, fs: f64) -> Self {
        let t = 1.0 / fs;
        let d = 1.0 / (t * a0 + a1 + a1);
        Self {
            c: [
                (b0 * t + b1 + b1) * d,
                (b0 * t - (b1 + b1)) * d,
                (t * a0 - (a1 + a1)) * d,
            ],
            s: 0.0,
        }
    }

    pub fn processar_no_lugar(&mut self, y: &mut [f64]) {
        let [c0, c1, c3] = self.c;
        for v in y.iter_mut() {
            let x = *v;
            let out = c0 * x + self.s;
            self.s = c1 * x - out * c3;
            *v = out;
        }
    }
}

/// `tan(ω/2fs)·2fs` (`0x141165dc0`).
fn pre_distorcao(w: f64, fs: f64) -> f64 {
    let dois_fs = fs + fs;
    (w / dois_fs).tan() * dois_fs
}

/// Energia por quarto de batida e por compasso (`0x1411885f0`).
///
/// `sinal` = sinal da passada A; `lp100` = passa-baixa de 100 Hz do motor
/// (`+0x3a0`, com estado); `grade` = índices 1-based de `0x141173280`.
/// Para cada batida `i`: três janelas de `q = (int)(fs/bpm·15 + 0,5)` amostras
/// a partir de `grade[i] − 1` e a quarta até `grade[i+1]` (ou até
/// `grade[i] + (int)(fs/bpm·60 + 0,5) − 1` nas duas últimas batidas). Fora da
/// taxa 5512,5 ± 0,05 os valores são multiplicados por 4. Os compassos somam
/// 16 valores na árvore do assembly.
///
/// ADAPTAÇÃO: janela fora do sinal devolve `None`; o original lê além do fim.
pub fn energia_por_quarto(
    sinal: &[f64],
    lp100: &mut Biquad,
    bpm: f64,
    fs: f64,
    grade: &[i32],
) -> Option<(Vec<f64>, Vec<f64>)> {
    let mut y = vec![0.0; sinal.len()];
    lp100.processar(sinal, &mut y);
    let batida = fs * (1.0 / bpm) * 60.0 + 0.5;
    let q = (fs * (1.0 / bpm) * 15.0 + 0.5) as i32;
    let janela = |ini: i32, len: i32| -> Option<f64> {
        if len < 0 || ini < 1 {
            return None;
        }
        let a = (ini - 1) as usize;
        y.get(a..a + len as usize).map(energia)
    };
    let n = grade.len();
    let mut quartos = Vec::with_capacity(n * 4);
    for i in 0..n {
        let mut pos = grade[i];
        let fim = if i + 2 < n {
            grade[i + 1]
        } else {
            batida as i32 - 1 + grade[i]
        };
        for _ in 0..3 {
            quartos.push(janela(pos, q)?);
            pos += q;
        }
        quartos.push(janela(pos, fim - pos)?);
    }
    if fs <= 5512.45 || 5512.55 <= fs {
        for v in quartos.iter_mut() {
            *v *= 4.0;
        }
    }
    let compassos = quartos
        .chunks_exact(16)
        .take(n / 4)
        .map(|p| {
            let x4 = (p[2] + p[1]) + (0.0 + p[0]);
            let x3 = x4 + ((p[4] + p[3]) + p[5]);
            let x4 = ((x3 + p[6]) + p[7]) + p[8];
            let x5 = x4 + (p[10] + p[9]);
            let x2 = ((p[12] + p[11]) + p[13]) + (p[15] + p[14]);
            x5 + x2
        })
        .collect();
    Some((quartos, compassos))
}

/// `0x1434e5600` (2π·300) e `0x1435380c8` (2π·5).
const W_300HZ: f64 = 1_884.955_592_153_875_8;
const W_5HZ: f64 = 31.415_926_535_897_93;

/// Onset de agudos (`0x141178330`): passa-alta Butterworth de 300 Hz
/// (`0x1434e5600` = 2π·300) em quatro seções (duas, duplicadas), valor
/// absoluto, passa-baixa de 1ª ordem de 5 Hz (`0x1435380c8` = 2π·5) e
/// diferença com `d[0] = 0`. Filtros novos a cada chamada (locais no
/// original). Taxa 0 vira 5512,5.
pub fn onset_agudos(sinal: &[f64], fs: f64) -> Vec<f64> {
    let fs = if fs == 0.0 { 5512.5 } else { fs };
    let o = pre_distorcao(W_300HZ, fs);
    let secao = |i: i32| {
        let k = (i as f64 * std::f64::consts::FRAC_PI_4 + std::f64::consts::FRAC_PI_8).cos();
        Biquad::bilinear(1.0, 0.0, 0.0, 1.0, (k + k) * o, o * o, fs)
    };
    let mut secoes = [secao(0), secao(1), secao(0), secao(1)];
    let o5 = pre_distorcao(W_5HZ, fs);
    let mut lp5 = PrimeiraOrdem::bilinear(0.0, o5, 1.0, o5, fs);
    let mut y = vec![0.0; sinal.len()];
    secoes[0].processar(sinal, &mut y);
    for s in &mut secoes[1..] {
        s.processar_no_lugar(&mut y);
    }
    for v in y.iter_mut() {
        *v = v.abs();
    }
    lp5.processar_no_lugar(&mut y);
    for i in (1..y.len()).rev() {
        y[i] -= y[i - 1];
    }
    if let Some(v) = y.first_mut() {
        *v = 0.0;
    }
    y
}

/// Soma (`0x14116ef60`, via `0x141167150`): blocos de 8 em dois acumuladores
/// de 4 posições, `s = a1 + a0`, redução `(s2 + s3) + (s0 + s1)` somada a 0
/// e cauda escalar.
pub fn soma(x: &[f64]) -> f64 {
    let blocos = x.len() / 8 * 8;
    let mut acc = 0.0;
    if blocos > 0 {
        let (mut a0, mut a1) = ([0.0f64; 4], [0.0f64; 4]);
        for b in x[..blocos].chunks_exact(8) {
            for l in 0..4 {
                a0[l] += b[l];
                a1[l] += b[l + 4];
            }
        }
        let s: Vec<f64> = (0..4).map(|l| a1[l] + a0[l]).collect();
        acc += (s[2] + s[3]) + (s[0] + s[1]);
    }
    for &v in &x[blocos..] {
        acc += v;
    }
    acc
}

/// Moldes de 16 posições (quatro batidas × quatro quartos) de
/// `0x141176680`.
const MOLDES: [[u8; 16]; 5] = [
    [1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0],
    [0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0],
    [1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0],
    [1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
    [1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0],
];

/// Sinalizadores de padrão rítmico de `0x141176680`, na ordem dos ponteiros
/// de saída (5º a 8º argumentos). Nomes descrevem os moldes, não o uso
/// (DESCONHECIDO).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Padrao {
    /// Molde 1000×4 ou 0010×4 em pelo menos 2 blocos de 8 compassos.
    pub seminimas: bool,
    pub molde_1000_0010_1000_0000: bool,
    pub molde_1000_0000_1000_0000: bool,
    pub molde_1000_0000_1010_0000: bool,
}

/// Classificação do padrão de picos (`0x141176680`).
///
/// `marcas` = 16 bytes por compasso, saída de `0x14118b160`; `compassos` =
/// energia por compasso (`motor+0x3f8`); `bpm` = `motor+0xb0`.
/// O compasso inicial é o primeiro, a partir do 17º (ou do 1º, com menos de
/// 17), cuja energia supera a média, até o antepenúltimo. Em até 8 blocos de 8
/// compassos, só as posições 2, 3, 6 e 7 de cada bloco contam, e um compasso
/// conta para um molde quando as 16 posições batem. Com
/// `t = blocos_disponíveis > 63`: dois ou mais blocos com 1000×4/0010×4 dão
/// `seminimas`; senão o molde 1000 0000 1010 0000 vence se aparecer em mais
/// de `t` blocos; senão 1000 0010 1000 0000, se aparecer em mais de `t`
/// blocos e o BPM for menor que 165 (`0x143538040`); senão
/// 1000 0000 1000 0000, se aparecer em mais de `t` blocos.
///
/// ADAPTAÇÃO: compasso fora de `marcas` devolve `None`; o original lê além.
pub fn classificar_padrao(marcas: &[u8], compassos: &[f64], bpm: f64) -> Option<Padrao> {
    let n = compassos.len() as i32;
    let mut inicio = 1i32;
    if n != 0 {
        let base = if n < 17 { 1 } else { 17 };
        inicio = base;
        let media = soma(compassos) / n as f64;
        let mut u = base;
        while u <= n - 3 {
            if media < compassos[(u - 1) as usize] {
                inicio = u;
                break;
            }
            u += 1;
        }
    }
    let disponiveis = -((marcas.len() as f64 * -0.063) as i32) - inicio;
    let mut flags = [[0i32; 8]; 4];
    if disponiveis > 0 {
        let blocos = if disponiveis > 63 { 8 } else { disponiveis / 8 };
        let mut l = (inicio - 1) as i64;
        for o in 0..blocos as usize {
            let mut cont = [0i32; 4];
            for k in 1..=8 {
                l += 1;
                if !(matches!(k, 2 | 3 | 6 | 7) && l > 1) {
                    continue;
                }
                let g = (l - 1) as usize * 16;
                let b = marcas.get(g..g + 16)?;
                let bate = |m: &[u8; 16]| m.iter().zip(b).all(|(x, y)| x == y);
                if bate(&MOLDES[0]) || bate(&MOLDES[1]) {
                    cont[0] += 1;
                }
                for (c, m) in cont[1..].iter_mut().zip(&MOLDES[2..]) {
                    if bate(m) {
                        *c += 1;
                    }
                }
            }
            for (f, c) in flags.iter_mut().zip(cont) {
                f[o] = (c > 0) as i32;
            }
        }
    }
    let [ab, c, d, e] = flags.map(|f| f.iter().sum::<i32>());
    let t = (disponiveis > 63) as i32;
    let mut p = Padrao::default();
    if ab >= 2 {
        p.seminimas = true;
    } else if t < e {
        p.molde_1000_0000_1010_0000 = true;
    } else if c <= t || bpm >= 165.0 {
        p.molde_1000_0000_1000_0000 = t < d;
    } else {
        p.molde_1000_0010_1000_0000 = true;
    }
    Some(p)
}

/// Ordenação decrescente e estável de `(valor, índice 1-based)`
/// (`0x1411678b0`; inserção em `0x14116e4c0` move enquanto `valor >
/// anterior`). Acima de 32 elementos o original usa merge sort estável com o
/// mesmo comparador, que dá o mesmo resultado.
fn argsort_decrescente(x: &[f64]) -> (Vec<f64>, Vec<i32>) {
    let mut pares: Vec<(f64, i32)> = x.iter().copied().zip(1..).collect();
    pares.sort_by(|a, b| {
        if a.0 > b.0 {
            std::cmp::Ordering::Less
        } else if b.0 > a.0 {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });
    pares.into_iter().unzip()
}

/// Picos de agudos por compasso (`0x1411786a0`).
///
/// Para cada um dos `grade.len() / 8` primeiros compassos (passo de quatro
/// batidas), ordena o onset de agudos do compasso em ordem decrescente e
/// aceita o candidato `k` quando o seguinte cai abaixo de `0,9886 ×` ele
/// (`0x1435380b8`) e ele não está a `±((int)(fs·15/bpm + 0,5) − 20)` amostras
/// dos primeiros `aceitos` índices da lista ordenada. Para no primeiro
/// candidato sem queda quando também não há queda logo depois. Capacidade de
/// 8 picos por compasso no total.
///
/// Depois, em ordem crescente, um pico sem outro a duas batidas
/// (`(int)(60/bpm·2fs + 0,5)`) com tolerância `(int)(fs·1,875/bpm + 0,5)`
/// (`0x1435380c0`) é zerado, em sequência. Por fim subtrai
/// `(int)(fs·ms·0,001 + 0,5)` dos picos válidos. BPM 0 vira 120 e taxa 0 vira
/// 5512,5.
///
/// ADAPTAÇÃO: compasso fora do onset dá `None`.
pub fn picos_de_agudos(
    onset: &[f64],
    fs: f64,
    bpm: f64,
    deslocamento_ms: i32,
    grade: &[i32],
) -> Option<PicosAgudos> {
    let bpm = if bpm == 0.0 { 120.0 } else { bpm };
    let fs = if fs == 0.0 { 5512.5 } else { fs };
    let compassos = (grade.len() as i32 / 8) as usize;
    let cap = compassos * 8;
    let mut pos = vec![0i32; cap];
    let raio = (fs * (15.0 / bpm) + 0.5) as i32 - 20;
    let mut total = 0usize;
    let mut ordenados = Vec::with_capacity(compassos);
    for b in 0..compassos {
        let ini = grade[4 * b];
        let len = grade[4 * b + 4] - ini;
        if ini < 1 || len < 0 {
            return None;
        }
        let a = (ini - 1) as usize;
        let (v, idx) = argsort_decrescente(onset.get(a..a + len as usize)?);
        let n = v.len();
        let mut aceitos = 0usize;
        let mut k = 0usize;
        while k + 1 < n {
            let perto = idx[..aceitos]
                .iter()
                .any(|&j| j - raio <= idx[k] && idx[k] <= j + raio);
            if !perto {
                let cheio = total >= cap;
                if 0.9886 * v[k] <= v[k + 1] || cheio {
                    if n < k + 3 || cheio || v[k + 1] * 0.9886 <= v[k + 2] {
                        break;
                    }
                } else {
                    pos[total] = ini - 1 + idx[k];
                    total += 1;
                    aceitos += 1;
                }
            }
            k += 1;
        }
        ordenados.push((v, idx));
    }
    pos.sort();
    let duas = ((60.0 / bpm) * (fs + fs) + 0.5) as i32;
    let tol = (fs * (1.875 / bpm) + 0.5) as i32;
    for i in 0..cap {
        let p = pos[i];
        if p == 0 {
            continue;
        }
        let (antes, depois) = (p - duas, p + duas);
        let tem_par = pos.iter().any(|&q| {
            q != 0
                && ((q - tol <= antes && antes <= q + tol)
                    || (q - tol <= depois && depois <= q + tol))
        });
        if !tem_par {
            pos[i] = 0;
        }
    }
    pos.sort();
    let zeros = pos.iter().take_while(|&&p| p == 0).count();
    let validos = cap - zeros;
    let vetorial = if validos > 15 {
        validos - validos % 16
    } else {
        0
    };
    for (i, p) in pos[zeros..].iter_mut().enumerate() {
        let d = if i < vetorial {
            (deslocamento_ms as f64 * (fs * 0.001) + 0.5) as i32
        } else {
            (fs * deslocamento_ms as f64 * 0.001 + 0.5) as i32
        };
        *p -= d;
    }
    Some(PicosAgudos {
        posicoes: pos,
        validos,
        primeiro: zeros + 1,
        ordenados,
    })
}

/// Saída de `picos_de_agudos`.
#[derive(Debug, Clone, PartialEq)]
pub struct PicosAgudos {
    /// Ordem crescente, zeros à frente.
    pub posicoes: Vec<i32>,
    pub validos: usize,
    /// Primeiro válido, 1-based.
    pub primeiro: usize,
    /// Por compasso: onset ordenado em ordem decrescente e índices 1-based
    /// no compasso (guardados nos vetores de `param_13`).
    pub ordenados: Vec<(Vec<f64>, Vec<i32>)>,
}

/// Saída de `confirmar_picos`: a lista `param_12` (posições) e `param_13`
/// (valores), em ordem crescente com zeros à frente.
#[derive(Debug, Clone, PartialEq)]
pub struct PicosConfirmados {
    pub posicoes: Vec<i32>,
    pub valores: Vec<f64>,
    pub validos: usize,
    pub primeiro: usize,
}

/// Permuta `v` pela ordenação crescente estável de `p` (`0x141168940`).
fn ordenar_junto(p: &mut [i32], v: &mut [f64]) {
    let mut pares: Vec<(i32, f64)> = p.iter().copied().zip(v.iter().copied()).collect();
    pares.sort_by_key(|a| a.0);
    for (i, (a, b)) in pares.into_iter().enumerate() {
        p[i] = a;
        v[i] = b;
    }
}

/// Segunda rodada de picos (`0x141179530`).
///
/// Seleção nas listas ordenadas de `picos_de_agudos`, como a primeira, mas a
/// vizinhança `±((int)(batida·0,25 + 0,5) − 20)` é medida contra as posições
/// já aceitas em todos os compassos, o teste usa `índice + início` e não há
/// teste de capacidade no corte. Depois, em sequência e em ordem crescente:
/// um pico sem par a duas batidas (`(int)(2·batida + 0,5)`, tolerância
/// `(int)(batida·0,125 + 0,5)`, `0x145415198`) é zerado; com par, é zerado se
/// as marcas de pico em volta do seu quarto de batida não tiverem um dos
/// padrões `(−4, +2)`, `(−6, +5)` ou `(−4, +4)` ou se houver marca no próprio
/// quarto; a cópia de saída só zera os que também não têm par a quatro
/// batidas (tolerância `[−¼, +⅛]` de batida). Com `seminimas == false`, a
/// cópia zera os picos cujo quarto anterior está marcado. Saídas ordenadas e
/// deslocadas por `(int)((fs·ms)·0,001 + 0,5)`. BPM 0 vira 120 e taxa 0 vira
/// 5512,5.
///
/// ADAPTAÇÃO: posições fora dos vetores dão `None`.
pub fn confirmar_picos(
    picos: &PicosAgudos,
    marcas: &[u8],
    fs: f64,
    bpm: f64,
    deslocamento_ms: i32,
    grade: &[i32],
    seminimas: bool,
) -> Option<PicosConfirmados> {
    let bpm = if bpm == 0.0 { 120.0 } else { bpm };
    let fs = if fs == 0.0 { 5512.5 } else { fs };
    let compassos = (grade.len() as i32 / 8) as usize;
    let cap = compassos * 8;
    let batida = (60.0 / bpm) * fs;
    let quarto = (batida * 0.25 + 0.5) as i32;
    let mut pos = vec![0i32; cap];
    let mut val = vec![0.0f64; cap];
    let mut total = 0usize;
    for b in 0..compassos {
        let base = grade[4 * b];
        let (v, idx) = picos.ordenados.get(b)?;
        let n = v.len();
        let mut k = 0usize;
        while k + 1 < n {
            let candidato = idx[k] + base;
            let perto = pos[..total]
                .iter()
                .any(|&q| q - (quarto - 20) <= candidato && candidato <= q + quarto - 20);
            if !perto && total < cap {
                if 0.9886 * v[k] <= v[k + 1] {
                    if n < k + 3 || v[k + 1] * 0.9886 <= v[k + 2] {
                        break;
                    }
                } else {
                    pos[total] = idx[k] - 1 + base;
                    val[total] = v[k];
                    total += 1;
                }
            }
            k += 1;
        }
    }
    ordenar_junto(&mut pos, &mut val);
    let mut saida = pos.clone();
    let mut saida_val = val.clone();
    let duas = (batida + batida + 0.5) as i32;
    let quatro = (batida * 4.0 + 0.5) as i32;
    let oitavo = (batida * 0.125 + 0.5) as i32;
    let desloc = (fs * deslocamento_ms as f64 * 0.001 + 0.5) as i32;
    let marca = |i: i32| marcas.get(i as usize).copied();
    for i in 0..cap {
        let p = pos[i];
        if p == 0 {
            continue;
        }
        let par = |d: i32, inf: i32, sup: i32, pos: &[i32]| {
            pos.iter().any(|&q| {
                q != 0
                    && ((q - inf <= p - d && p - d <= q + sup)
                        || (q - inf <= p + d && p + d <= q + sup))
            })
        };
        let par2 = par(duas, oitavo, oitavo, &pos);
        let par4 = par(quatro, quarto, oitavo, &pos);
        let zerar = if par2 {
            let slot = (p - desloc) / quarto;
            if 5 < slot && slot + 5 < marcas.len() as i32 {
                let m = |o: i32| marca(slot + o).map(|x| x == 1);
                let c = m(-4)?;
                let sem_padrao = (!c || !m(2)?) && (!m(-6)? || !m(5)?) && (!c || !m(4)?);
                sem_padrao || m(0)?
            } else {
                false
            }
        } else {
            true
        };
        if zerar {
            pos[i] = 0;
            val[i] = 0.0;
        }
        if !par4 && !par2 {
            saida[i] = 0;
            saida_val[i] = 0.0;
        }
    }
    if !seminimas {
        for p in saida.iter_mut() {
            if *p != 0 {
                let slot = (!desloc + *p) / quarto;
                if 0 < slot + 1 && slot + 1 < marcas.len() as i32 && marca(slot)? == 1 {
                    *p = 0;
                }
            }
        }
    }
    ordenar_junto(&mut saida, &mut saida_val);
    let zeros = saida.iter().take_while(|&&p| p == 0).count();
    for p in saida[zeros..].iter_mut() {
        *p -= desloc;
    }
    Some(PicosConfirmados {
        posicoes: saida,
        valores: saida_val,
        validos: cap - zeros,
        primeiro: zeros + 1,
    })
}

/// Terceira rodada de picos (`0x141178df0`).
///
/// Mesma seleção de `confirmar_picos`, com capacidade de 16 por compasso e
/// razão de queda `0,998849369936505` (`0x1435380d8`). Um pico fica se tiver
/// par a duas batidas (tolerância de ⅛ de batida) e, com
/// `6 <= quarto < marcas − 5`, se as marcas formarem `(−4, +2)`, `(−6, +5)`
/// ou `(−4, +4)` sem marca no próprio quarto; senão é zerado, em sequência.
/// Os valores paralelos (`param_12`) são sempre zero no original e não são
/// devolvidos. Deslocamento final como em `picos_de_agudos`.
///
/// Devolve `(posições, válidos, primeiro válido 1-based)`.
pub fn picos_finos(
    picos: &PicosAgudos,
    marcas: &[u8],
    fs: f64,
    bpm: f64,
    deslocamento_ms: i32,
    grade: &[i32],
) -> Option<(Vec<i32>, usize, usize)> {
    let bpm = if bpm == 0.0 { 120.0 } else { bpm };
    let fs = if fs == 0.0 { 5512.5 } else { fs };
    let compassos = (grade.len() as i32 / 8) as usize;
    let cap = compassos * 16;
    let batida = (60.0 / bpm) * fs;
    let quarto = (batida * 0.25 + 0.5) as i32;
    let raio = quarto - 20;
    let razao = 0.998_849_369_936_505;
    let mut pos = vec![0i32; cap];
    let mut total = 0usize;
    for b in 0..compassos {
        let base = grade[4 * b];
        let (v, idx) = picos.ordenados.get(b)?;
        let n = v.len();
        let mut k = 0usize;
        while k + 1 < n {
            let candidato = idx[k] + base;
            let perto = pos[..total]
                .iter()
                .any(|&q| q - raio <= candidato && candidato <= q + raio);
            if !perto && total < cap {
                if razao * v[k] <= v[k + 1] {
                    if n < k + 3 || v[k + 1] * razao <= v[k + 2] {
                        break;
                    }
                } else {
                    pos[total] = idx[k] - 1 + base;
                    total += 1;
                }
            }
            k += 1;
        }
    }
    pos.sort();
    let duas = (batida + batida + 0.5) as i32;
    let oitavo = (batida * 0.125 + 0.5) as i32;
    let desloc = (fs * deslocamento_ms as f64 * 0.001 + 0.5) as i32;
    for i in 0..cap {
        let p = pos[i];
        if p == 0 {
            continue;
        }
        let (antes, depois) = (p - duas, p + duas);
        let par = pos.iter().any(|&q| {
            q != 0
                && ((q - oitavo <= antes && antes <= q + oitavo)
                    || (q - oitavo <= depois && depois <= q + oitavo))
        });
        let fica = par && {
            let slot = (p - desloc) / quarto;
            if slot < 6 || marcas.len() as i32 <= slot + 5 {
                true
            } else {
                let m = |o: i32| marcas[(slot + o) as usize] == 1;
                ((m(-4) && m(2)) || (m(-6) && m(5)) || (m(-4) && m(4))) && !m(0)
            }
        };
        if !fica {
            pos[i] = 0;
        }
    }
    pos.sort();
    let zeros = pos.iter().take_while(|&&p| p == 0).count();
    let validos = cap - zeros;
    let vetorial = if validos > 15 {
        validos - validos % 16
    } else {
        0
    };
    for (i, p) in pos[zeros..].iter_mut().enumerate() {
        let d = if i < vetorial {
            (deslocamento_ms as f64 * (fs * 0.001) + 0.5) as i32
        } else {
            desloc
        };
        *p -= d;
    }
    Some((pos, validos, zeros + 1))
}

/// Energia por quarto de batida numa grade fixa (`0x141189170`).
///
/// O sinal passa pelo filtro dado (`motor+0x438`, projetado por
/// `filtro::passa_baixa_hz`). A grade começa em `(int)((fs·ms)·0,001 + 0,5)`
/// (índice 0-based), com batida `(int)(fs·(1/bpm)·60 + 0,5)` e quarto
/// `(int)(fs·(1/bpm)·15 + 0,5)`; há `(n − início) / batida` batidas, cada uma
/// com três quartos e o resto. Todos os valores são multiplicados por 4.
pub fn energia_por_quarto_fixa(
    sinal: &[f64],
    filtro: &mut Biquad,
    bpm: f64,
    deslocamento_ms: i32,
    fs: f64,
) -> Option<Vec<f64>> {
    let mut y = vec![0.0; sinal.len()];
    filtro.processar(sinal, &mut y);
    let inicio = (fs * deslocamento_ms as f64 * 0.001 + 0.5) as i32;
    let batida = (fs * (1.0 / bpm) * 60.0 + 0.5) as i32;
    let quarto = (fs * (1.0 / bpm) * 15.0 + 0.5) as i32;
    if batida == 0 {
        return None;
    }
    let n = (y.len() as i32 - inicio) / batida;
    let janela = |a: i32, len: i32| -> Option<f64> {
        if a < 0 || len < 0 {
            return None;
        }
        y.get(a as usize..(a + len) as usize).map(energia)
    };
    let mut out = Vec::with_capacity(n.max(0) as usize * 4);
    for i in 0..n {
        let base = inicio + i * batida;
        for j in 0..3 {
            out.push(janela(base + j * quarto, quarto)?);
        }
        out.push(janela(base + 3 * quarto, batida - 3 * quarto)?);
    }
    for v in out.iter_mut() {
        *v *= 4.0;
    }
    Some(out)
}

/// Resultado de `fase_do_compasso` (`0x141172490`, saídas 13 a 16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fase {
    /// Novo deslocamento em ms (`p13`).
    pub ms: i32,
    /// Modo de correção (`p14`): 0 nenhum, 1 recuo, 2 ou 3 avanço por quartos
    /// (3 quando o deslocamento não mudou).
    pub modo: i32,
    /// Molde `1000001` + 9 zeros encontrado (`p15`).
    pub molde_a: bool,
    /// `p16`: deslocamento mantido com energia sustentada. `None` no retorno
    /// antecipado, em que o original não escreve.
    pub mantido: Option<bool>,
}

const MOLDE_A: [u8; 7] = [1, 0, 0, 0, 0, 0, 1];
const MOLDE_C: [u8; 24] = [
    1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0,
];

/// Fase do compasso (`0x141172490`), conferida no assembly.
///
/// `marcas` = 16 bytes por compasso de `0x14118b160`; `picos` = saída de
/// `picos_de_agudos` (`p10`–`p12`); `sinal` = sinal da passada A;
/// `n_grade` = quantidade de índices da grade.
///
/// ADAPTAÇÃO: moldes que passam do fim das marcas não casam, e energia vazia
/// não avança o deslocamento; o original lê além do fim nos dois casos.
pub fn fase_do_compasso(
    sinal: &[f64],
    marcas: &[u8],
    fs: f64,
    bpm: f64,
    ms: i32,
    n_grade: usize,
    picos: &PicosAgudos,
) -> Option<Fase> {
    let blocos = (n_grade as i32 / 8) as usize;
    if blocos < 2 || ((marcas.len() as f64 * 0.031_25) as i32) < 2 {
        return Some(Fase {
            ms,
            modo: 0,
            molde_a: false,
            mantido: None,
        });
    }
    let inv = 60.0 / bpm;
    let batida = inv * fs;
    let quarto = batida * 0.25;
    let casa = |i: usize, m: &[u8]| marcas.get(i..i + m.len()).is_some_and(|b| b == m);
    let um = |i: usize| marcas.get(i) == Some(&1);
    let (mut a, mut b, mut c, mut d) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for i in 0..blocos * 16 {
        if casa(i, &MOLDE_A) {
            if casa(i + 7, &[0; 9]) {
                a.push(i as i32);
            }
            if casa(i + 8, &[0; 2]) {
                b.push(i as i32);
            }
        }
        if casa(i, &MOLDE_C) {
            c.push(i as i32);
        }
        if um(i)
            && marcas.get(i + 1) == Some(&0)
            && (um(i + 3) || um(i + 7))
            && (um(i + 6) || um(i + 10))
        {
            d.push(i as i32);
        }
    }
    // Lista de referência e limiar (0x1454151e0, 0x145415250, 0x145415270).
    let mut lista: Option<&Vec<i32>> = None;
    let mut limiar = 0.5;
    let mut por_modulo = false;
    if c.len() > 4 {
        lista = Some(&c);
        limiar = 0.2;
    } else if a.len() > 4 {
        lista = Some(&a);
        limiar = 0.4;
    } else if d.len() > 16 {
        let razao = if d.len() <= 25 {
            0.65f32 as f64
        } else if d.len() <= 60 {
            0.41f32 as f64
        } else {
            0.3f32 as f64
        };
        let r: Vec<i32> = d.iter().map(|&x| (x + 1) % 4).collect();
        let mx = *r.iter().max().unwrap_or(&0);
        let mut iguais = 0.0;
        for &x in &r {
            if x == mx {
                iguais += 1.0;
            }
        }
        if mx == 3 && iguais / d.len() as f64 > razao {
            por_modulo = true;
        }
    }
    if b.len() > 4 && !por_modulo && lista.is_none() {
        lista = Some(&b);
        limiar = 0.45;
    }
    // Período de um quarto de batida em ms (0x1454156e8 = 250).
    let periodo = if bpm == 0.0 {
        125
    } else {
        (inv * 250.0) as i32
    };
    if periodo <= 0 {
        return None;
    }
    let mut r = ms;
    while r > periodo {
        r -= periodo;
    }
    let e1 = energia_por_quarto_fixa(
        sinal,
        &mut crate::filtro::passa_baixa_hz(400.0, fs),
        bpm,
        r,
        fs,
    )?;
    let e2 = energia_por_quarto_fixa(
        sinal,
        &mut crate::filtro::passa_baixa_hz(100.0, fs),
        bpm,
        ms,
        fs,
    )?;
    let maximo = |v: &[f64]| {
        v.iter().copied().fold(None, |m: Option<f64>, x| match m {
            Some(y) if y < x => Some(x),
            Some(y) => Some(y),
            None => Some(x),
        })
    };
    // Primeiro quarto forte em e1 (0,48 e 0,6000000000000001).
    let m1 = maximo(&e1).unwrap_or(0.0) * 0.48;
    let (mut i1, mut zeros) = (0usize, false);
    for i in 0..e1.len() {
        let x = e1[i];
        if x > m1 && (i < 1 || (e1[i - 1] - x).abs() > x * 0.600_000_000_000_000_1) {
            i1 = i;
            zeros = i > 0 && e1[..i].iter().all(|&v| v == 0.0);
            break;
        }
    }
    // Energia sustentada em e2 (0,5 e 0,605).
    let m2 = maximo(&e2).unwrap_or(0.0);
    let mut sustentada = false;
    let mut bvar49 = false;
    let primeiro = e2.iter().position(|&x| m2 * 0.5 < x).map(|k| k + 1);
    if let Some(r10) = primeiro {
        let lim = m2 * 0.605;
        let n = e2.len();
        let mut cont = 0;
        let mut j = r10;
        let mut disparou = false;
        while j <= n {
            if lim >= e2[j - 1] {
                cont = 0;
            } else {
                cont += 1;
                if cont >= 16 {
                    disparou = true;
                    break;
                }
            }
            j += 4;
        }
        if !disparou {
            let mut blocos_fortes = 0;
            let mut ini = r10;
            while ini <= n {
                let fim = (ini + 63).min(n);
                let mut k = ini;
                while k <= fim {
                    if lim >= e2[k - 1] {
                        cont = 0;
                    } else {
                        cont += 1;
                        if cont >= 12 {
                            blocos_fortes += 1;
                            break;
                        }
                    }
                    k += 4;
                }
                if blocos_fortes >= 3 {
                    disparou = true;
                    break;
                }
                if cont < 12 {
                    blocos_fortes = 0;
                }
                ini += 64;
                cont = 0;
            }
        }
        if disparou {
            sustentada = true;
            bvar49 = r10 % 4 == 3;
        }
    }
    let mut avancou = false;
    if (i1 < 4 || zeros) && e1.get(i1).is_some_and(|&x| 70.0 < x) {
        r += i1 as i32 * periodo;
        avancou = true;
    }
    // Fração de picos fora do tempo forte, pela lista de referência.
    let mut fora = false;
    if let Some(ref_lista) = lista {
        let mut unicos: Vec<i32> = Vec::new();
        let cap = blocos * 8;
        for &v in ref_lista.iter() {
            if picos.validos > 0 && picos.primeiro - 1 < cap {
                let lim = v as f64 * quarto;
                for &x in &picos.posicoes[picos.primeiro - 1..cap] {
                    if x as f64 > lim && !unicos.contains(&x) {
                        unicos.push(x);
                    }
                }
            }
        }
        if !unicos.is_empty() {
            let div = (batida + 0.5) as i32;
            let n_fora = unicos
                .iter()
                .filter(|&&u| {
                    let resto = (u % div) as f64;
                    resto > quarto && resto < batida * 0.75
                })
                .count();
            fora = n_fora as f64 / unicos.len() as f64 > limiar;
        }
    }
    let (novo, modo) = if avancou {
        (r, (r == ms) as i32 + 2)
    } else if !(fora || bvar49 || por_modulo) {
        (ms, 0)
    } else {
        let mut x = if ms - 250 >= 0 { ms - 250 } else { ms };
        if bpm != 0.0 {
            x = ms - inv.mul_add(500.0, 0.5) as i32;
            if x < 0 {
                x += inv.mul_add(1000.0, 0.5) as i32;
            }
        }
        (x, 1)
    };
    Some(Fase {
        ms: novo,
        modo,
        molde_a: !a.is_empty(),
        mantido: Some(ms == novo && sustentada),
    })
}

/// Estado do motor lido e gravado pela marcação de picos (`motor+0x268`,
/// `+0x270`, `+0x278`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EstadoMarcas {
    /// Última amostra do envelope antes da diferença (`+0x268`).
    pub anterior: f64,
    /// Deslocamento achado pelo alinhamento, em amostras (`+0x270`).
    pub alinhamento: i32,
    /// Máximo corrente dos grupos (`+0x278`).
    pub maximo: f64,
}

impl Default for EstadoMarcas {
    fn default() -> Self {
        Self {
            anterior: 0.0,
            alinhamento: 0,
            maximo: 0.0001,
        }
    }
}

/// Primeiro máximo e seu índice 1-based (`0x1411672c0` → `0x142604de0`).
// Negação de comparação reproduz `vcomisd` + `jbe` do original com NaN.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn max_com_indice(x: &[f64]) -> Option<(f64, i32)> {
    let mut melhor: Option<(f64, usize)> = None;
    for (i, &v) in x.iter().enumerate() {
        match melhor {
            Some((m, _)) if !(v > m) => {}
            _ => melhor = Some((v, i)),
        }
    }
    melhor.map(|(m, i)| (m, i as i32 + 1))
}

/// Primeiro mínimo (`0x141167340` → `0x142604f60`).
// Negação de comparação reproduz `vcomisd` + `jbe` do original com NaN.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn min_valor(x: &[f64]) -> Option<f64> {
    x.iter().copied().fold(None, |m, v| match m {
        Some(a) if !(v < a) => Some(a),
        _ => Some(v),
    })
}

/// Alinhamento inicial de `0x14118b160` (`0x14118b4a9`–`0x14118b834`),
/// lido no assembly. Devolve o deslocamento em amostras.
// Negação de comparação reproduz `vcomisd` + `jbe` do original com NaN.
#[allow(clippy::neg_cmp_op_on_partial_ord, clippy::too_many_arguments)]
fn alinhar(
    y: &[f64],
    r15: i32,
    l288: i32,
    p: i32,
    qd: f64,
    bar_d: f64,
    grupos: i32,
    descida: bool,
) -> Option<i32> {
    let bar = bar_d as i32;
    let q = (qd + 0.5) as i32;
    let n = y.len() as i32;
    let g = |i: i32| -> Option<f64> { y.get(usize::try_from(i).ok()?).copied() };
    let ini = usize::try_from(r15 + bar - 1).ok()?;
    let mut copia = y.get(ini..ini + ((grupos - 1) * bar) as usize)?.to_vec();
    let mut cand = [0i32; 8];
    // `xmm12` e `r13d` valem entre voltas (zerados em 0x14118b4a9).
    let mut x12 = 0.0f64;
    let mut r13 = 0i32;
    for k in 0..8 {
        if k > 0 && cand[k - 1] >= 1 && cand[k - 1] as usize <= copia.len() {
            copia[(cand[k - 1] - 1) as usize] = 0.0;
        }
        let Some((m, idx)) = max_com_indice(&copia) else {
            continue;
        };
        cand[k] = idx;
        let e = if bar_d != 0.0 {
            ((cand[k] as f64 / bar_d) as i32 + 1) * bar
        } else {
            (0xffff + 1) * bar
        };
        let val = if e + r15 - 1 <= n {
            g(e + r15 - 2)?
        } else {
            g(n - 1)?
        };
        if !(val > m * 0.2 && e < copia.len() as i32 - bar) {
            continue;
        }
        let w0 = usize::try_from(l288 - 1 + e).ok()?;
        let janela = y.get(w0..w0 + (bar + 1) as usize)?;
        let (x1, x0, nx) = (g(e + r15 - 1)?, g(e + r15)?, g(e + r15 + 1)?);
        let subida = if val > x1 {
            if descida && x1 > x0 && x0 > nx {
                false
            } else {
                continue;
            }
        } else if x1 > x0 || x0 > nx {
            continue;
        } else {
            true
        };
        let b = e - 2 + r15;
        for t in 1..=q {
            let d = if subida {
                g(b + t + 1)? - g(b + t)?
            } else {
                g(b - t - 1)? - g(b - t)?
            };
            if d < 0.0 {
                let r = if subida { t + e } else { e - t };
                x12 = g(r15 + r - 2)?;
                r13 = r;
                break;
            }
        }
        if !(x12 > m * 0.595_662) {
            continue;
        }
        let limiar = (m + min_valor(janela)?) * 0.5;
        let mut vale = m;
        let (mut achou, mut quietos, mut pos) = (false, 0, 0);
        let lim = p * 5 / 4;
        let mut resultado = None;
        for t in 1..=lim {
            let x = g(r13 - t + r15 - 2)?;
            if !achou {
                if x > g(r15 - 3 + r13 - t)? || x >= limiar {
                    continue;
                }
                achou = true;
                pos = r13 - t;
                vale = x;
            } else if x > vale * 1.12 {
                achou = false;
                quietos = 0;
            } else {
                quietos += 1;
                if quietos >= (qd * 0.25) as i32 {
                    resultado = Some(());
                    break;
                }
            }
        }
        if resultado.is_some() {
            if vale == 0.0 {
                return Some(0);
            }
            if x12 > vale * 2.11 {
                let r = ((e as f64 - pos as f64) + 0.5) as i32;
                return Some(if r < r15 { r } else { r15 });
            }
        }
    }
    Some(0)
}

/// Marcação de picos (`0x14118b160`), lida no assembly.
///
/// `y` = envelope antes da diferença (`FiltrosEnvelope::envelope`);
/// `inicio_batida` = 5º argumento (1 no fechamento da passada A); `chamada`
/// = 7º argumento. Batida `p = (int)fma(60/bpm, fs, 0,5)`, início
/// `(int)fma(ms·0,001, fs, 0,5)`, quarto `(int)(p·0,25 + 0,5)`. Com dois ou
/// mais grupos de quatro batidas, o alinhamento move o sinal; depois, em cada
/// grupo, 16 janelas de um quarto (base `fma(g, 4p, 1)`) dão o máximo, e até
/// seis vezes o maior é marcado se passar 0,0001, `0,596 ×` o primeiro máximo
/// do grupo e `0,178 ×` o máximo corrente.
///
/// Devolve as marcas (16 por grupo; um byte zero quando não há grupo).
/// ADAPTAÇÃO: leituras fora do sinal devolvem `None`.
pub fn marcar_picos(
    y: &[f64],
    estado: &mut EstadoMarcas,
    fs: f64,
    bpm: f64,
    ms: i32,
    inicio_batida: i32,
    chamada: i32,
) -> Option<Vec<u8>> {
    marcar_interno(
        y,
        y,
        estado,
        fs,
        bpm,
        ms,
        inicio_batida,
        chamada,
        6,
        true,
        false,
    )
}

/// Marcação da banda de 110 a 200 Hz (`0x14118c1b0`), lida no assembly e
/// comparada instrução a instrução com `0x14118b160`. Diferenças: o
/// alinhamento usa `envelope_200` e só o caso de subida (o de descida foi
/// removido pelo compilador como inalcançável); a grade usa `envelope_banda`;
/// até 12 rodadas por grupo; `estado.anterior` (`motor+0x4e0`) recebe sempre a
/// última amostra de `envelope_banda` antes da diferença.
#[allow(clippy::too_many_arguments)]
pub fn marcar_picos_banda(
    envelope_200: &[f64],
    envelope_banda: &[f64],
    estado: &mut EstadoMarcas,
    fs: f64,
    bpm: f64,
    ms: i32,
    inicio_batida: i32,
    chamada: i32,
) -> Option<Vec<u8>> {
    marcar_interno(
        envelope_200,
        envelope_banda,
        estado,
        fs,
        bpm,
        ms,
        inicio_batida,
        chamada,
        12,
        false,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn marcar_interno(
    y: &[f64],
    y_marcas: &[f64],
    estado: &mut EstadoMarcas,
    fs: f64,
    bpm: f64,
    ms: i32,
    inicio_batida: i32,
    chamada: i32,
    rodadas: usize,
    descida: bool,
    anterior_sempre: bool,
) -> Option<Vec<u8>> {
    let n = y.len() as i32;
    let inicio = (ms as f64 * 0.001).mul_add(fs, 0.5) as i32;
    let p = (60.0 / bpm).mul_add(fs, 0.5) as i32;
    if p == 0 {
        return None;
    }
    let qd = p as f64 * 0.25;
    let q = (qd + 0.5) as i32;
    let bar_d = p as f64 * 4.0;
    let l288 = (inicio_batida - 1) * p + inicio;
    let r15 = l288 + 1;
    let nb = (n - r15 + 1) / p;
    let mut grupos = nb / 4;
    let fim = (r15 - 1) + nb * p;
    let bar = bar_d as i32;
    if bar * grupos - 1 + r15 > n {
        grupos -= 1;
    }
    let maximo_ini = if chamada > 1 { estado.maximo } else { 0.0001 };
    if grupos < 1 {
        estado.anterior = *y.last()?;
        estado.maximo = maximo_ini;
        return Some(vec![0]);
    }
    let alinh = if grupos > 1 {
        alinhar(y, r15, l288, p, qd, bar_d, grupos, descida)?
    } else {
        0
    };
    let antigo = estado.alinhamento;
    estado.alinhamento = alinh;
    let desloc = if chamada > 1 { antigo } else { alinh };
    // Diferença de primeira ordem, de trás para frente.
    let mut sig = y_marcas.to_vec();
    let anterior = estado.anterior;
    if anterior_sempre {
        estado.anterior = *sig.last()?;
    }
    for i in (1..sig.len()).rev() {
        sig[i] -= sig[i - 1];
    }
    if let Some(v) = sig.first_mut() {
        *v -= anterior;
    }
    let l294 = fim - r15;
    if r15 > desloc && fim - desloc <= n {
        let a = (r15 - desloc - 1) as usize;
        sig = sig.get(a..a + (l294 + 1) as usize)?.to_vec();
    } else if fim - desloc - 1 <= n && r15 < desloc {
        let c = fim - desloc - 1;
        if c < 0 {
            return None;
        }
        let tmp = sig.get(..c as usize)?.to_vec();
        if n < l294 {
            sig.resize(l294 as usize, 0.0);
        }
        let z = (desloc - r15 + 1) as usize;
        for v in sig.iter_mut().take(z) {
            *v = 0.0;
        }
        sig.get_mut(z..z + tmp.len())?.copy_from_slice(&tmp);
    }
    let mut marcas = vec![0u8; grupos as usize * 16];
    let mut corrente = 0.0001f64;
    let p_d = p as f64;
    for gr in 0..grupos {
        let base = (gr as f64).mul_add(bar_d, 1.0);
        let mut v = [0.0f64; 16];
        for j in 0..4 {
            let x6 = j as f64 * p_d + base;
            for k in 0..4 {
                let ini = (k as f64 * qd + x6) as i32;
                let a = usize::try_from(ini - 1).ok()?;
                let janela = sig.get(a..a + q as usize)?;
                v[j * 4 + k] = max_com_indice(janela).map_or(0.0, |(m, _)| m);
            }
        }
        let mut primeiro = 0.0;
        for rodada in 0..rodadas {
            let (mut mx, mut im) = (v[0], 0usize);
            for (i, &x) in v.iter().enumerate().skip(1) {
                if x > mx {
                    im = i;
                }
                if mx < x {
                    mx = x;
                }
            }
            if rodada == 0 {
                primeiro = mx;
            }
            if mx > 0.0001 && mx > primeiro * 0.596 && mx > corrente * 0.178 {
                marcas[gr as usize * 16 + im] = 1;
                v[im] = 0.0;
            }
        }
        corrente = if primeiro > corrente {
            primeiro
        } else {
            corrente
        };
    }
    estado.maximo = corrente;
    Some(marcas)
}

/// Padrão de dois compassos entre as marcas da banda larga (`a`, de
/// `marcar_picos`) e da banda de 110 a 200 Hz (`b`, de `marcar_picos_banda`)
/// (`0x141177b30`).
///
/// Em cada posição `i` de `0` a `len(a) − 33`: ambas marcadas em `i`; ambas
/// vazias em `i+2..=i+6`; ao menos uma posição de `i+7..=i+9` só em `b`; ambas
/// vazias em `i+10..=i+14`; ao menos uma posição de `i+15..=i+17` em ambas.
/// Leituras além de `len(a)` falham (as vazias) ou não contam (as contagens).
/// Uma ocorrência em `i` conta se a anterior não contada estava em
/// `i − 17`, `i − 16` ou `i − 15` (o original guarda `anterior + 17` e compara
/// com `i`, `i + 1` e `i + 2`); depois de contar, a referência volta a 0.
/// Como ela começa em 0, uma ocorrência em `i <= 2` já conta. Devolve
/// verdadeiro com mais de uma contada.
///
/// ADAPTAÇÃO: posição fora de `b` não casa (o original lê além do fim).
pub fn padrao_duas_bandas(a: &[u8], b: &[u8]) -> bool {
    let n = a.len() as i64;
    let par = |j: i64| -> Option<(u8, u8)> {
        if j >= n {
            return None;
        }
        Some((*a.get(j as usize)?, *b.get(j as usize)?))
    };
    let vazias = |de: i64, ate: i64| (de..=ate).all(|j| par(j) == Some((0, 0)));
    let (mut ultimo, mut contagem) = (0i64, 0);
    let mut i = 0i64;
    while i < n - 32 {
        let ok = par(i) == Some((1, 1))
            && vazias(i + 2, i + 6)
            && (i + 7..=i + 9).any(|j| par(j) == Some((0, 1)))
            && vazias(i + 10, i + 14)
            && (i + 15..=i + 17).any(|j| par(j) == Some((1, 1)));
        if ok {
            if ultimo == i || ultimo == i + 1 || ultimo == i + 2 {
                contagem += 1;
                ultimo = 0;
            } else {
                ultimo = i + 17;
            }
        }
        i += 1;
    }
    contagem > 1
}

/// Batida (1-based) em que o áudio começa (`0x141171760`), com FMAs
/// conferidos no assembly.
///
/// `sinal` = primeiros `min(n, fs/8·30)` valores do sinal da passada A. Com
/// `início = (int)fma(ms·0,001, fs, 0,5)` e
/// `batida = (int)fma(60/bpm, fs, 0,5)` (0,5 no lugar de `60/bpm` com BPM
/// nulo), procura o primeiro `k >= 1` com `|sinal[início + k − 1]| > 0,0005`
/// (`0x143537eb0`; posição fora vale 0) e devolve
/// `(int)(k / batida + 0,5) + 1`, ou `(int)(k · 0,000362811… + 0,5) + 1`
/// com batida 0 (`0x143537ea8` = 1/2756,25). Sem amostra audível, `k = 0`.
pub fn primeira_batida_audivel(sinal: &[f64], bpm: f64, ms: i32, fs: f64) -> i32 {
    let inicio = (ms as f64 * 0.001).mul_add(fs, 0.5) as i32;
    let fator = if bpm != 0.0 { 60.0 / bpm } else { 0.5 };
    let batida = fator.mul_add(fs, 0.5) as i32;
    let n = sinal.len() as i32;
    let mut k = 0;
    for j in 1..n {
        let idx = inicio - 1 + j;
        let v = if idx < n {
            usize::try_from(idx)
                .ok()
                .and_then(|i| sinal.get(i))
                .map_or(0.0, |x| x.abs())
        } else {
            0.0
        };
        if 0.0005 < v {
            k = j;
            break;
        }
    }
    let d = if batida == 0 {
        k as f64 * 0.000_362_811_791_383_219_96
    } else {
        k as f64 / batida as f64
    };
    (d + 0.5) as i32 + 1
}

/// Resultado de `numero_da_batida` (`0x1411718e0`, saídas 17 a 24).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumeroBatida {
    /// Número da batida inicial (`p17`, gravado em `motor+0x20c`).
    pub numero: i32,
    /// Paridade escolhida (`p18`, 0 ou 1).
    pub paridade: i32,
    /// Picos no tempo forte do ciclo de duas batidas (`p20`).
    pub fortes: i32,
    /// Picos no tempo forte ou no meio do ciclo (`p21`).
    pub fortes_ou_meio: i32,
    /// `fortes / fortes_ou_meio`, 1 sem picos (`p22`).
    pub razao: f64,
    /// Energia de quarto forte nos quartos 5 a 8 de 16 (`p24`).
    pub segunda_metade: bool,
}

/// Energia por quarto de `0x1411889e0`: como `energia_por_quarto`, mas a
/// quarta janela só usa o fim calculado na última batida e o fator 4 vale
/// sempre.
fn energia_por_quarto_numero(
    sinal: &[f64],
    lp100: &mut Biquad,
    bpm: f64,
    fs: f64,
    grade: &[i32],
) -> Option<Vec<f64>> {
    let mut y = vec![0.0; sinal.len()];
    lp100.processar(sinal, &mut y);
    let batida = fs * (1.0 / bpm) * 60.0 + 0.5;
    let q = (fs * (1.0 / bpm) * 15.0 + 0.5) as i32;
    let janela = |ini: i32, len: i32| -> Option<f64> {
        if len < 0 || ini < 1 {
            return None;
        }
        let a = (ini - 1) as usize;
        y.get(a..a + len as usize).map(energia)
    };
    let n = grade.len();
    let mut out = Vec::with_capacity(n * 4);
    for i in 0..n {
        let mut pos = grade[i];
        let fim = if i + 1 < n {
            grade[i + 1]
        } else {
            batida as i32 - 1 + grade[i]
        };
        for _ in 0..3 {
            out.push(janela(pos, q)?);
            pos += q;
        }
        out.push(janela(pos, fim - pos)?);
    }
    for v in out.iter_mut() {
        *v *= 4.0;
    }
    Some(out)
}

/// Número da batida inicial (`0x1411718e0`), conferido no assembly.
///
/// `posicoes`/`validos`/`primeiro` = saída de `picos_finos` (`p10`, `p12`,
/// `p13`; os valores `p11` são sempre zero no original); `ms_original` e
/// `ms_novo` = deslocamento antes e depois de `fase_do_compasso` (`p5`,
/// `p6`); `base` = 7º argumento (1); `campo_bc` = `motor+0xbc` (`p14`);
/// `modo` = modo de `fase_do_compasso` (`p16`).
///
/// 1. Os picos válidos andam `ms_original − ms_novo` ms: nos blocos de 16 do
///    laço vetorial por `d · (fs·0,001)` somado; no resto por
///    `fma(d·0,001, fs, pos)`.
/// 2. Com `batida = (int)fma(60/bpm, fs, 0,5)` (BPM 0 vira 0,1) e
///    `q = (int)fma(batida, 0,25, 0,5)`, cada pico tem resto
///    `(int)(pos + 0,5) % 2·batida`: perto de 0 (até `q`) é forte; a até `q`
///    de `batida` é meio; o resto não conta.
/// 3. Paridade: com até 6 picos fortes ou do meio, `(campo_bc − 1) mod 2`;
///    senão 1 se a razão passar 0,886 (`0x143537eb8`), senão 0 (a regra de
///    0,15 compara somas de valores, que são zero).
/// 4. Energia por quarto com passa-baixa de 100 Hz (`motor+0x730`); limiar
///    `fma(média, 0,6, máximo·0,4)`; a partir do quarto `4·paridade`, o
///    primeiro valor acima do limiar na posição `k mod 16` igual a 0, 1, 2,
///    13, 14 ou 15 encerra; em 5 a 8 liga `segunda_metade`.
/// 5. `numero = base + 2·segunda_metade + paridade`, salvo quando
///    `modo != 1` e `(campo_bc + 1) mod 2 == paridade`: então `numero` é
///    `base` com modo 2 ou 3 e `campo_bc` nos demais.
#[allow(clippy::too_many_arguments)]
pub fn numero_da_batida(
    sinal: &[f64],
    fs: f64,
    bpm: f64,
    ms_original: i32,
    ms_novo: i32,
    base: i32,
    grade: &[i32],
    posicoes: &[i32],
    validos: usize,
    primeiro: usize,
    campo_bc: i32,
    modo: i32,
) -> Option<NumeroBatida> {
    let mut pos: Vec<f64> = posicoes.iter().map(|&p| p as f64).collect();
    let ini = primeiro.checked_sub(1)?;
    let trecho = pos.get_mut(ini..ini + validos)?;
    let d = (ms_original - ms_novo) as f64;
    let vetorial = if validos > 15 {
        validos - validos % 16
    } else {
        0
    };
    for (i, v) in trecho.iter_mut().enumerate() {
        *v = if i < vetorial {
            d * (fs * 0.001) + *v
        } else {
            (d * 0.001).mul_add(fs, *v)
        };
    }
    let bpm_b = if bpm == 0.0 { 0.1 } else { bpm };
    let batida = (60.0 / bpm_b).mul_add(fs, 0.5) as i32;
    let q = (batida as f64).mul_add(0.25, 0.5) as i32;
    if batida == 0 {
        return None;
    }
    let (mut fortes, mut ambos) = (0, 0);
    for &x in &pos[ini..ini + validos] {
        let r = (0.5 + x) as i32 % (batida * 2);
        let classe = if q < r && r <= batida * 2 - q {
            if r < batida - q || q + batida < r {
                -1
            } else {
                0
            }
        } else {
            1
        };
        if classe == 1 {
            fortes += 1;
        }
        if classe == 0 || classe == 1 {
            ambos += 1;
        }
    }
    let razao = if ambos > 0 {
        fortes as f64 / ambos as f64
    } else {
        1.0
    };
    let paridade = if ambos <= 6 {
        (campo_bc - 1) % 2
    } else if razao > 0.886 {
        1
    } else {
        // 0,15 < razão: (soma dos valores fortes > soma dos do meio), ambas 0.
        0
    };
    let e = energia_por_quarto_numero(
        sinal,
        &mut crate::filtro::passa_baixa_hz(100.0, fs),
        bpm_b,
        fs,
        grade,
    )?;
    let maximo = e.iter().copied().fold(None, |m: Option<f64>, x| match m {
        Some(y) if y < x => Some(x),
        Some(y) => Some(y),
        None => Some(x),
    });
    let media = if e.is_empty() {
        0.0
    } else {
        soma(&e) / e.len() as f64
    };
    let limiar = media.mul_add(0.6, maximo.unwrap_or(0.0) * 0.4);
    let mut segunda_metade = false;
    let n = e.len() as i32;
    for k in 0..(n - 4).max(0) {
        let x = *e.get((paridade * 4 + k) as usize)?;
        if limiar >= x {
            continue;
        }
        let r = k % 16;
        if r == 0 || (1..=2).contains(&r) || (13..=15).contains(&r) {
            break;
        }
        if (5..=8).contains(&r) {
            segunda_metade = true;
            break;
        }
    }
    let soma_numero = if segunda_metade { 2 } else { 0 } + i32::from(paridade == 1);
    let numero = if modo != 1 && (campo_bc + 1) % 2 == paridade {
        if (modo - 2) as u32 > 1 {
            campo_bc
        } else {
            base
        }
    } else {
        soma_numero + base
    };
    Some(NumeroBatida {
        numero,
        paridade,
        fortes,
        fortes_ou_meio: ambos,
        razao,
        segunda_metade,
    })
}

/// Grade de índices 1-based (`0x141173280`), conferida no assembly:
/// passo `(60/bpm)·fs`, início `(int)fma(ms·0,001, fs, 0,5)`, quantidade
/// `(int)((n − início) / passo)` e índice `(int)((i·passo + início) + 1,5)`;
/// o último é limitado a `n`.
///
/// ADAPTAÇÃO: quantidade negativa dá `None` (o original falha na alocação) e
/// grade vazia não lê o índice −1.
pub fn grade_de_indices(n: usize, fs: f64, bpm: f64, ms: i32) -> Option<Vec<i32>> {
    let passo = (60.0 / bpm) * fs;
    let inicio = (ms as f64 * 0.001).mul_add(fs, 0.5) as i32;
    let n = n as i32;
    let qtd = ((n - inicio) as f64 / passo) as i32;
    if qtd < 0 {
        return None;
    }
    let mut g: Vec<i32> = (0..qtd)
        .map(|i| ((i as f64 * passo + inicio as f64) + 1.5) as i32)
        .collect();
    if let Some(u) = g.last_mut() {
        if n < *u {
            *u = n;
        }
    }
    Some(g)
}

/// Grades das três bandas da passada B (`0x141173370`, chamada por
/// `0x141195ed0`), conferidas no assembly. Por banda: passo `(60/bpm)·fs`,
/// início `(int)fma(ms·0,001, fs, 0,5)` e quantidade
/// `(int)((n − início) / passo)`, com `n = frames / M` em `f64`. A quantidade
/// comum `q` é o mínimo das três (gravado em `+0x1180`); cada grade tem
/// `q + 1` índices `(int)((i·passo + início) + 1,5)` (`+0x1120`, `+0x1140`,
/// `+0x1160`) e o índice `q − 1` (o penúltimo) é limitado a `(int)n`.
///
/// ADAPTAÇÃO: `q < 0` dá `None` (o original falha na alocação ou, com
/// `q = −1`, lê o índice −2) e `q = 0` não lê o índice −1.
pub fn grades_passada_b(n: [f64; 3], fs: [f64; 3], bpm: f64, ms: i32) -> Option<[Vec<i32>; 3]> {
    let passo = fs.map(|f| (60.0 / bpm) * f);
    let inicio = fs.map(|f| (ms as f64 * 0.001).mul_add(f, 0.5) as i32 as f64);
    let q = (0..3)
        .map(|b| ((n[b] - inicio[b]) / passo[b]) as i32)
        .min()?;
    if q < 0 {
        return None;
    }
    Some(std::array::from_fn(|b| {
        let mut g: Vec<i32> = (0..=q)
            .map(|i| ((i as f64 * passo[b] + inicio[b]) + 1.5) as i32)
            .collect();
        if q > 0 {
            let u = &mut g[q as usize - 1];
            *u = (*u).min(n[b] as i32);
        }
        g
    }))
}

/// Parâmetros fixos do analisador gravados no init (`0x141170020`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfigPassadaA {
    /// Taxa decimada (`motor+0x98` = taxa/8).
    pub fs: f64,
    /// BPM de partida (`motor+0xb0` = BPM × 100 · 0,01 do init).
    pub bpm: f64,
    /// Deslocamento de partida em ms (`motor+0xac`).
    pub ms: i32,
    /// `motor+0xbc` (6º argumento do init).
    pub campo_bc: i32,
    /// `motor+0xca`: mantém o deslocamento de partida.
    pub manter_ms: bool,
    /// `motor+0xcb`: não recalcula o número da batida.
    pub manter_numero: bool,
}

/// Resultado do fechamento da passada A (`motor+0x208`–`+0x260`).
#[derive(Debug, Clone, PartialEq)]
pub struct ResultadoPassadaA {
    /// Deslocamento final em ms (`+0x208`).
    pub ms: i32,
    /// Número da batida inicial (`+0x20c`).
    pub numero: i32,
    /// Molde A visto nas marcas (`+0x210`).
    pub molde_a: bool,
    /// Picos confirmados (`+0x218`, `+0x238`, `+0x258`, `+0x25c`).
    pub picos: PicosConfirmados,
    /// Batida em que o áudio começa (`+0x260`).
    pub primeira_audivel: i32,
}

/// Fechamento da passada A (`0x141195480`): encadeia os estágios portados na
/// ordem do original. `sinal` = sinal da passada A (mono, passa-baixa de
/// `fs/8 · 0,4` e decimação por 8), até `fs · 210` amostras.
///
/// Primeira volta com o deslocamento de partida: grade, marcas, energia por
/// quarto, padrão rítmico, onset de agudos, três rodadas de picos e fase do
/// compasso. Segunda volta com o deslocamento corrigido: nova grade, marcas
/// das duas bandas, padrão entre bandas, início audível (30 s) e número da
/// batida.
pub fn fechamento_passada_a(sinal: &[f64], c: &ConfigPassadaA) -> Option<ResultadoPassadaA> {
    use crate::filtro::{passa_baixa_hz, FiltrosBanda, FiltrosEnvelope};
    let (fs, bpm, ms) = (c.fs, c.bpm, c.ms);
    let lim = ((fs * 210.0) as usize).min(sinal.len());
    let sinal = &sinal[..lim];
    // Primeira volta.
    let grade = grade_de_indices(sinal.len(), fs, bpm, ms)?;
    let mut env = FiltrosEnvelope::novo(fs)?;
    let mut est = EstadoMarcas::default();
    let y = env.envelope(sinal)?;
    let marcas = marcar_picos(&y, &mut est, fs, bpm, ms, 1, 1)?;
    let mut lp100 = passa_baixa_hz(100.0, fs);
    let (_, compassos) = energia_por_quarto(sinal, &mut lp100, bpm, fs, &grade)?;
    let padrao = classificar_padrao(&marcas, &compassos, bpm)?;
    let onset = onset_agudos(sinal, fs);
    let agudos = picos_de_agudos(&onset, fs, bpm, ms, &grade)?;
    let confirmados = confirmar_picos(&agudos, &marcas, fs, bpm, ms, &grade, padrao.seminimas)?;
    let (finos, validos_finos, primeiro_finos) =
        picos_finos(&agudos, &marcas, fs, bpm, ms, &grade)?;
    let fase = fase_do_compasso(sinal, &marcas, fs, bpm, ms, grade.len(), &agudos)?;
    let (mut ms_novo, mut modo) = (fase.ms, fase.modo);
    if c.manter_ms {
        ms_novo = ms;
        modo = 0;
    }
    // Segunda volta.
    let grade2 = grade_de_indices(sinal.len(), fs, bpm, ms_novo)?;
    let mut env2 = FiltrosEnvelope::novo(fs)?;
    let mut est2 = EstadoMarcas::default();
    let y2 = env2.envelope(sinal)?;
    let marcas2 = marcar_picos(&y2, &mut est2, fs, bpm, ms_novo, 1, 1)?;
    let mut banda = FiltrosBanda::novo(fs)?;
    let mut est3 = EstadoMarcas::default();
    let (e200, ebanda) = banda.envelopes(sinal)?;
    let marcas_banda = marcar_picos_banda(&e200, &ebanda, &mut est3, fs, bpm, ms_novo, 1, 1)?;
    let _duas_bandas = padrao_duas_bandas(&marcas2, &marcas_banda);
    let trinta = ((fs * 30.0) as usize).min(sinal.len());
    let primeira_audivel = primeira_batida_audivel(&sinal[..trinta], bpm, ms_novo, fs);
    let numero = if validos_finos < 1 || c.manter_numero || fase.mantido == Some(true) {
        c.campo_bc
    } else {
        numero_da_batida(
            sinal,
            fs,
            bpm,
            ms,
            ms_novo,
            1,
            &grade2,
            &finos,
            validos_finos,
            primeiro_finos,
            c.campo_bc,
            modo,
        )?
        .numero
    };
    Some(ResultadoPassadaA {
        ms: ms_novo,
        numero,
        molde_a: fase.molde_a,
        picos: confirmados,
        primeira_audivel,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constantes_de_frequencia() {
        assert_eq!(W_300HZ.to_bits(), 0x409d73d286bfe4d8);
        assert_eq!(W_5HZ.to_bits(), 0x403f6a7a2955385e);
    }

    #[test]
    fn energia_soma_quadrados() {
        assert_eq!(energia(&[]), 0.0);
        assert_eq!(energia(&[3.0, 4.0]), 25.0);
        let x: Vec<f64> = (1..=11).map(|v| v as f64).collect();
        assert_eq!(energia(&x), 506.0);
    }

    #[test]
    fn primeira_ordem_tem_ganho_dc_1() {
        let w = std::f64::consts::PI * 10.0;
        let o = pre_distorcao(w, 5512.5);
        let mut f = PrimeiraOrdem::bilinear(0.0, o, 1.0, o, 5512.5);
        let mut y = vec![1.0; 20000];
        f.processar_no_lugar(&mut y);
        assert!((y[19999] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn energia_por_quarto_cobre_as_janelas() {
        // fs 400, 60 BPM: batida = 400 amostras, quarto = 100.
        let sinal = vec![1.0; 4000];
        let mut lp = Biquad::bilinear(0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 400.0);
        // Coeficiente identidade: b0 = 1, resto 0.
        lp.c = [1.0, 0.0, 0.0, 0.0, 0.0];
        let grade: Vec<i32> = (0..8).map(|i| 1 + i * 400).collect();
        let (q, c) = energia_por_quarto(&sinal, &mut lp, 60.0, 400.0, &grade).unwrap();
        assert_eq!(q.len(), 32);
        // Fora de 5512,5: ×4. Quarta janela: 100 amostras até a próxima
        // batida, mas 99 nas duas últimas (fim = início + 400 − 1).
        assert!(q[..24].iter().all(|&v| v == 400.0));
        assert_eq!(
            &q[24..],
            &[400.0, 400.0, 400.0, 396.0, 400.0, 400.0, 400.0, 396.0]
        );
        assert_eq!(c, vec![6400.0, 6392.0]);
        // Grade além do sinal.
        assert!(energia_por_quarto(&sinal, &mut lp, 60.0, 400.0, &[3990]).is_none());
    }

    #[test]
    fn onset_agudos_zera_o_inicio_e_rejeita_dc() {
        let y = onset_agudos(&vec![1.0; 5000], 5512.5);
        assert_eq!(y[0], 0.0);
        assert!(y[4999].abs() < 1e-9);
    }

    #[test]
    fn soma_vetorial() {
        let x: Vec<f64> = (1..=19).map(|v| v as f64).collect();
        assert_eq!(soma(&x), 190.0);
        assert_eq!(soma(&[]), 0.0);
    }

    #[test]
    fn padrao_de_seminimas() {
        // 80 compassos com pico em cada batida; energia uniforme.
        let marcas: Vec<u8> = MOLDES[0].iter().copied().cycle().take(16 * 80).collect();
        let compassos = vec![1.0; 20];
        let p = classificar_padrao(&marcas, &compassos, 128.0).unwrap();
        assert!(p.seminimas);
        // Sem picos: nada marcado.
        let p = classificar_padrao(&vec![0u8; 16 * 80], &compassos, 128.0).unwrap();
        assert_eq!(p, Padrao::default());
        // Molde 1000 0010 1000 0000 só vale abaixo de 165 BPM.
        let m: Vec<u8> = MOLDES[2].iter().copied().cycle().take(16 * 80).collect();
        assert!(
            classificar_padrao(&m, &compassos, 128.0)
                .unwrap()
                .molde_1000_0010_1000_0000
        );
        assert!(
            !classificar_padrao(&m, &compassos, 170.0)
                .unwrap()
                .molde_1000_0010_1000_0000
        );
    }

    #[test]
    fn argsort_e_decrescente_e_estavel() {
        let (v, i) = argsort_decrescente(&[1.0, 3.0, 2.0, 3.0]);
        assert_eq!(v, vec![3.0, 3.0, 2.0, 1.0]);
        assert_eq!(i, vec![2, 4, 3, 1]);
    }

    #[test]
    fn picos_de_agudos_acha_o_pico_e_exige_par() {
        // fs 400, 120 BPM: batida = 200 amostras; compasso = 800.
        let fs = 400.0;
        let grade: Vec<i32> = (0..17).map(|i| 1 + i * 200).collect();
        let mut onset = vec![0.0; 3400];
        // Um pico forte por compasso, sempre 10 amostras após o início.
        for c in 0..2 {
            onset[c * 800 + 10] = 1.0;
        }
        let r = picos_de_agudos(&onset, fs, 120.0, 0, &grade).unwrap();
        // Picos a 800 amostras: sem par a duas batidas (400) -> zerados.
        assert_eq!((r.validos, r.primeiro), (0, r.posicoes.len() + 1));
        assert_eq!(r.ordenados.len(), 2);
        // Com pico também a 400 amostras, o par existe.
        for c in 0..2 {
            onset[c * 800 + 410] = 0.5;
        }
        let r = picos_de_agudos(&onset, fs, 120.0, 0, &grade).unwrap();
        assert_eq!(r.validos, 4);
        assert_eq!(&r.posicoes[r.posicoes.len() - 4..], &[11, 411, 811, 1211]);
        // Deslocamento de 10 ms a 400 Hz = 4 amostras.
        let d = picos_de_agudos(&onset, fs, 120.0, 10, &grade).unwrap();
        assert_eq!(d.posicoes[d.posicoes.len() - 1], 1207);
        // Segunda rodada, sem marcas: 411 e 811 têm par mas saem da lista
        // interna (sem padrão de marcas); 1211 então fica sem par a duas e a
        // quatro batidas e sai também da saída. 11 fica (quarto 0 < 6).
        let marcas = vec![0u8; 64];
        let c = confirmar_picos(&r, &marcas, fs, 120.0, 0, &grade, true).unwrap();
        assert_eq!(c.validos, 3);
        assert_eq!(&c.posicoes[c.posicoes.len() - 3..], &[11, 411, 811]);
        assert_eq!(&c.valores[c.valores.len() - 3..], &[1.0, 0.5, 1.0]);
        // Terceira rodada, sem marcas: só o pico no quarto 0 (fora da faixa
        // de marcas) fica; 411 e 811 caem por falta de padrão e 1211 fica sem
        // par depois disso.
        let (f, validos, _) = picos_finos(&r, &marcas, fs, 120.0, 0, &grade).unwrap();
        assert_eq!(f.len(), 32);
        assert_eq!((validos, f[31]), (1, 11));
    }

    #[test]
    fn energia_por_quarto_fixa_cobre_a_grade() {
        // fs 400, 60 BPM: batida 400, quarto 100; início 10 ms = 4 amostras.
        let sinal = vec![1.0; 1000];
        let mut f = Biquad::bilinear(0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 400.0);
        f.c = [1.0, 0.0, 0.0, 0.0, 0.0];
        let e = energia_por_quarto_fixa(&sinal, &mut f, 60.0, 10, 400.0).unwrap();
        // (1000 − 4) / 400 = 2 batidas.
        assert_eq!(e, vec![400.0; 8]);
        assert!(crate::filtro::passa_baixa_hz(400.0, 5512.5).c[0] > 0.0);
    }

    #[test]
    fn constantes_da_fase() {
        // 0x143537f70, 0x143537f80, 0x143537f68, 0x143537f60.
        assert_eq!(0.600_000_000_000_000_1f64.to_bits(), 0x3fe3333333333334);
        assert_eq!((0.65f32 as f64).to_bits(), 0x3fe4ccccc0000000);
        assert_eq!((0.41f32 as f64).to_bits(), 0x3fda3d70a0000000);
        assert_eq!((0.3f32 as f64).to_bits(), 0x3fd3333340000000);
    }

    #[test]
    fn fase_retorna_cedo_e_detecta_molde() {
        let vazio = PicosAgudos {
            posicoes: vec![0; 16],
            validos: 0,
            primeiro: 17,
            ordenados: vec![],
        };
        // Grade curta: retorno antecipado sem tocar o deslocamento.
        let f = fase_do_compasso(&[], &[0; 64], 5512.5, 120.0, 37, 8, &vazio).unwrap();
        assert_eq!((f.ms, f.modo, f.mantido), (37, 0, None));
        // Sinal mudo e marcas com o molde A uma vez: nada a corrigir.
        let mut marcas = vec![0u8; 64];
        marcas[..7].copy_from_slice(&MOLDE_A);
        let sinal = vec![0.0; 22050];
        let f = fase_do_compasso(&sinal, &marcas, 5512.5, 120.0, 37, 16, &vazio).unwrap();
        assert_eq!(
            (f.ms, f.modo, f.molde_a, f.mantido),
            (37, 0, true, Some(false))
        );
    }

    #[test]
    fn marcar_picos_marca_o_ataque_de_cada_batida() {
        // fs 400, 120 BPM: p = 200, quarto 50; envelope com subida no início
        // de cada batida.
        let fs = 400.0;
        let y: Vec<f64> = (0..2000)
            .map(|i| {
                if i % 200 < 10 {
                    1.0 + i as f64 * 1e-6
                } else {
                    0.0
                }
            })
            .collect();
        let mut e = EstadoMarcas::default();
        let m = marcar_picos(&y, &mut e, fs, 120.0, 0, 1, 1).unwrap();
        // (2000 − 1 + 1) / 200 = 10 batidas -> 2 grupos.
        assert_eq!(m.len(), 32);
        // Ataque no início de cada batida: molde 1000 em todos os quartos.
        let esperado: Vec<u8> = MOLDES[0].iter().copied().cycle().take(32).collect();
        assert_eq!(m, esperado);
        assert_eq!(e.alinhamento, 0);
        // Sinal curto: um byte zero e a última amostra guardada.
        let mut e2 = EstadoMarcas::default();
        let m2 = marcar_picos(&[0.0, 0.5], &mut e2, fs, 120.0, 0, 1, 1).unwrap();
        assert_eq!((m2, e2.anterior), (vec![0], 0.5));
    }

    #[test]
    fn marcar_banda_usa_doze_rodadas_e_guarda_anterior() {
        let fs = 400.0;
        let y: Vec<f64> = (0..2000)
            .map(|i| {
                if i % 50 < 5 {
                    1.0 + i as f64 * 1e-6
                } else {
                    0.0
                }
            })
            .collect();
        let mut e6 = EstadoMarcas::default();
        let mut e12 = EstadoMarcas::default();
        let m6 = marcar_picos(&y, &mut e6, fs, 120.0, 0, 1, 1).unwrap();
        let m12 = marcar_picos_banda(&y, &y, &mut e12, fs, 120.0, 0, 1, 1).unwrap();
        // Ataque em todo quarto: 6 rodadas marcam 6 de 16, 12 marcam 12.
        assert_eq!(m6[..16].iter().filter(|&&b| b == 1).count(), 6);
        assert_eq!(m12[..16].iter().filter(|&&b| b == 1).count(), 12);
        assert_eq!(e12.anterior, y[1999]);
        assert_eq!(e6.anterior, 0.0);
        assert!(crate::filtro::FiltrosBanda::novo(5512.5).is_some());
    }

    #[test]
    fn padrao_de_duas_bandas_encadeado() {
        // Um ciclo de 17 posições: 1 em 0 nas duas, 0 em 2..6, só b em 7,
        // 0 em 10..14, 1 nas duas em 15.
        let mut a = vec![0u8; 120];
        let mut b = vec![0u8; 120];
        for c in [0usize, 17, 34] {
            a[c] = 1;
            b[c] = 1;
            b[c + 7] = 1;
            a[c + 15] = 1;
            b[c + 15] = 1;
        }
        a[51] = 1;
        b[51] = 1;
        // 0 conta (referência inicial 0); 17 não conta e guarda 34; 34 conta.
        assert!(padrao_duas_bandas(&a, &b));
        // Sem o pico só em b, nada casa.
        let b2: Vec<u8> = a.clone();
        assert!(!padrao_duas_bandas(&a, &b2));
    }

    #[test]
    fn primeira_batida_audivel_conta_batidas_de_silencio() {
        // fs 400, 120 BPM: batida 200. Silêncio até a amostra 650.
        let mut x = vec![0.0; 2000];
        x[650] = -0.01;
        // k = 651 (índice 650 = início − 1 + k com início 0) -> 651/200 = 3,26.
        assert_eq!(primeira_batida_audivel(&x, 120.0, 0, 400.0), 4);
        assert_eq!(primeira_batida_audivel(&[0.0; 10], 120.0, 0, 400.0), 1);
    }

    #[test]
    fn numero_da_batida_conta_picos_no_ciclo_de_duas() {
        // fs 400, 120 BPM: batida 200, q 50, ciclo 400.
        let grade: Vec<i32> = (0..8).map(|i| 1 + i * 200).collect();
        // 8 picos no tempo forte (resto 0) e nenhum no meio: razão 1 > 0,886.
        let posicoes: Vec<i32> = (0..8).map(|i| i * 400).collect();
        let sinal = vec![0.0; 2000];
        let r =
            numero_da_batida(&sinal, 400.0, 120.0, 0, 0, 1, &grade, &posicoes, 8, 1, 1, 0).unwrap();
        assert_eq!((r.fortes, r.fortes_ou_meio, r.paridade), (8, 8, 1));
        // modo 0 e (1 + 1) % 2 = 0 != 1: número = 1 + 0 + 1.
        assert_eq!(r.numero, 2);
        // Com modo 2 e campo par: (2 + 1) % 2 = 1 == paridade, número = base.
        let r =
            numero_da_batida(&sinal, 400.0, 120.0, 0, 0, 1, &grade, &posicoes, 8, 1, 2, 2).unwrap();
        assert_eq!(r.numero, 1);
    }

    #[test]
    fn grade_de_indices_segue_o_assembly() {
        // fs 400, 120 BPM: passo 200; início 10 ms = (int)(4,5) = 4.
        let g = grade_de_indices(1000, 400.0, 120.0, 10).unwrap();
        // (1000 − 4) / 200 = 4,98 -> 4 índices: 5,5 -> 5, 205, 405, 605.
        assert_eq!(g, vec![5, 205, 405, 605]);
        assert!(grade_de_indices(10, 400.0, 120.0, 1000).is_none());
    }

    #[test]
    fn grades_passada_b_seguem_o_assembly() {
        // 120 BPM, 10 ms; passos 200, 50 e 12,5; inícios 4, 1 e 0.
        // Quantidades 3 (646/200), 4 (249/50) e 5 (62,5/12,5): q = 3.
        let [a, b, c] =
            grades_passada_b([650.0, 250.0, 62.5], [400.0, 100.0, 25.0], 120.0, 10).unwrap();
        assert_eq!(a, vec![5, 205, 405, 605]);
        assert_eq!(b, vec![2, 52, 102, 152]);
        assert_eq!(c, vec![1, 14, 26, 39]);
        assert!(grades_passada_b([10.0; 3], [400.0; 3], 120.0, 1000).is_none());
    }

    #[test]
    fn fechamento_passada_a_roda_de_ponta_a_ponta() {
        // Clique a cada batida (120 BPM) a 5512,5 Hz por 40 s.
        let fs = 5512.5;
        let batida = (60.0 / 120.0 * fs) as usize;
        let sinal: Vec<f64> = (0..(fs as usize * 40))
            .map(|i| {
                let k = i % batida;
                if k < 40 {
                    (k as f64 * 0.7).sin() * (1.0 - k as f64 / 40.0)
                } else {
                    0.0
                }
            })
            .collect();
        let c = ConfigPassadaA {
            fs,
            bpm: 120.0,
            ms: 0,
            campo_bc: 1,
            manter_ms: false,
            manter_numero: false,
        };
        let r = fechamento_passada_a(&sinal, &c).unwrap();
        // Regressão do porte (não comparado com o programa de referência):
        // cliques no tempo mantêm o deslocamento e a batida 1; sem padrões de
        // marcas, nenhum pico é confirmado.
        assert_eq!((r.ms, r.numero, r.primeira_audivel), (0, 1, 1));
        assert_eq!((r.molde_a, r.picos.validos), (false, 0));
    }
}
