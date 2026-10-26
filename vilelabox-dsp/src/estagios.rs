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
}
