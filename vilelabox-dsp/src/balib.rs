//! Estimador de BPM `BeatAnalyzer_1_0` (motor 1 do detector), em porte.
//!
//! Evidência: seção "Estimador de BPM" de
//! `spec/rekordbox/evidence/p4-1-analise-batida.md`. Endereços do 7.2.19.
//!
//! Etapas portadas aqui:
//! - alimentação (`0x142606e40`): mistura estéreo → mono e blocos;
//! - banco de 6 bandas do sensor de batidas (init `0x142612650`, filtros
//!   `0x14260d2d0`/`0x14260c520`, média móvel de `0x1426080d0`);
//! - sensor de batidas (`0x1426080d0`): seguidores de pico, anel e
//!   agrupamento de intervalos (`0x142611f20`, `0x1426120b0`), escolha da
//!   banda pelo histograma de BPM (`0x142608b20`) e lista de batidas
//!   (`0x142607b60`);
//! - contagem de BPM e geração da grade nos modos 0 e 1 (`0x142613100`,
//!   regrade uniforme `0x1426182f0` e ajuste local do modo 1 `0x142617e60`);
//! - refinador de envelope e fase de compasso (`UnitEnvelopeMaker`: init
//!   `0x142619aa0`, alimentação `0x142619ea0`, filtro passa-alta 1D
//!   `0x14260dd20`/`0x14260dd80`/`0x14260de00`, processamento `0x14261a080`);
//! - ajuste fino de BPM e fase no modo 0 (`UnitBeatAdjust`: init
//!   `0x142609070`, alimentação `0x142609260`, ajuste `0x142609560`) e
//!   orquestrador completo (`AnalisadorDeBatidas`, `0x142606e40`/`0x142607110`).
//!
//! ADAPTAÇÕES:
//! - `sin`, `cos` e `sinh` vêm da biblioteca padrão do Rust, não da CRT
//!   (`0x14280e240`, `0x1429b5400`). Os coeficientes podem diferir no
//!   último bit.
//! - O original liga flush-to-zero no MXCSR (`0x142606880`). O porte mantém
//!   subnormais, o que só muda resultados abaixo de ~1e−308.

/// 2π como gravado em `0x143ea0760`, truncado (não é `TAU`).
#[allow(clippy::approx_constant)] // o valor truncado é o do binário
const DOIS_PI: f64 = 6.283_185_307_179_58;
/// `0x1457ff3f0`.
const Q_BUTTERWORTH: f32 = 0.707_106_77;
/// `0x1457fed94` = 0,5/32 768.
const ESCALA_MONO: f32 = 1.525_878_9e-5;

/// Seção biquad em forma direta I (`0x14260c520`), coeficientes
/// normalizados por `a0` e estado `(x[n−2], x[n−1], y[n−2], y[n−1])` em
/// `+0x38`…`+0x50`.
#[derive(Debug, Clone, PartialEq)]
pub struct SecaoRbj {
    /// `[b0, b1, b2, a1, a2]`.
    pub c: [f64; 5],
    x2: f64,
    x1: f64,
    y2: f64,
    y1: f64,
}

/// `(sen, cos)` de `w = (f·2π)/fs`, com `f` e `fs` em `f32` como no original.
fn angulo(f: f32, fs: f32) -> (f64, f64) {
    ((f64::from(f) * DOIS_PI) / f64::from(fs)).sin_cos()
}

impl SecaoRbj {
    fn nova(c: [f64; 5]) -> Self {
        Self {
            c,
            x2: 0.0,
            x1: 0.0,
            y2: 0.0,
            y1: 0.0,
        }
    }

    /// Passa-baixa (`0x14260c280`), conferido no assembly.
    pub fn passa_baixa(f: f32, fs: f32, q: f32) -> Self {
        let (s, c) = angulo(f, fs);
        let d = 1.0 - c;
        let k = 0.5 / f64::from(q);
        let a0 = k.mul_add(s, 1.0);
        let inv = 1.0 / a0;
        let b0 = (d * 0.5) * inv;
        Self::nova([b0, inv * d, b0, (c * -2.0) / a0, (-k).mul_add(s, 1.0) / a0])
    }

    /// Passa-alta (`0x14260c8b0` e metade de `0x14260cb40`), conferido no
    /// assembly.
    pub fn passa_alta(f: f32, fs: f32, q: f32) -> Self {
        let (s, c) = angulo(f, fs);
        let k = 0.5 / f64::from(q);
        let h = 0.5f64.mul_add(c, 0.5);
        let a0 = k.mul_add(s, 1.0);
        let inv = 1.0 / a0;
        Self::nova([
            h / a0,
            (-1.0 - c) * inv,
            h * inv,
            (c * -2.0) * inv,
            inv * (-k).mul_add(s, 1.0),
        ])
    }

    /// `LowPathFilter2D` (`0x14260d630`), conferido no assembly: mesma
    /// receita de `passa_baixa`, com `f` e `fs` em `f64` e `k` fixo em
    /// `0x1457ff8c8`.
    pub fn passa_baixa_2d(f: f64, fs: f64) -> Self {
        let (s, c) = ((f * DOIS_PI) / fs).sin_cos();
        let k = std::f64::consts::FRAC_1_SQRT_2;
        let d = 1.0 - c;
        let a0 = s.mul_add(k, 1.0);
        let inv = 1.0 / a0;
        let b0 = (d * 0.5) * inv;
        Self::nova([b0, inv * d, b0, (c * -2.0) / a0, (-k).mul_add(s, 1.0) / a0])
    }

    /// `HighPathFilter2D` (`0x14260d9e0`), conferido no assembly. Difere de
    /// `passa_alta` em `b0 = h·(1/a0)`, `a1 = (c·−2)/a0` e `a2 = (…)/a0`.
    pub fn passa_alta_2d(f: f64, fs: f64) -> Self {
        let (s, c) = ((f * DOIS_PI) / fs).sin_cos();
        let k = std::f64::consts::FRAC_1_SQRT_2;
        let a0 = s.mul_add(k, 1.0);
        let h = c.mul_add(0.5, 0.5);
        let inv = 1.0 / a0;
        let b0 = h * inv;
        Self::nova([
            b0,
            (-1.0 - c) * inv,
            b0,
            (c * -2.0) / a0,
            (-k).mul_add(s, 1.0) / a0,
        ])
    }

    /// Passa-faixa de ganho 0 dB no centro, largura em oitavas
    /// (`0x14260cfa0`; `0x143ea0758` = ln 2 / 2), sem FMA.
    pub fn passa_faixa(f: f32, fs: f32, oitavas: f32) -> Self {
        let w = (f64::from(f) * DOIS_PI) / f64::from(fs);
        let (s, c) = w.sin_cos();
        let alfa = (((f64::from(oitavas) * 0.346_573_590_279_972_64) * w) / s).sinh() * s;
        let a0 = alfa + 1.0;
        let inv = 1.0 / a0;
        Self::nova([
            alfa / a0,
            0.0,
            -(alfa * inv),
            (c * -2.0) * inv,
            (1.0 - alfa) * inv,
        ])
    }

    /// Filtra um bloco, mantendo o estado. A ordem de acumulação segue o
    /// laço desenrolado do assembly: em geral `b0·x[n]` primeiro e depois
    /// `b1`, `b2`, `−a1`, `−a2` com FMA. A amostra 1 e a terceira de cada
    /// grupo de 4 do laço desenrolado (`n = 4, 8, …`, enquanto ele roda)
    /// começam por `b1·x[n−1]`. Por isso o resultado depende do tamanho do
    /// bloco, como no original.
    pub fn processar(&mut self, x: &[f64], y: &mut [f64]) {
        let n = x.len().min(y.len());
        if n == 0 {
            return;
        }
        let [b0, b1, b2, a1, a2] = self.c;
        let fim_desenrolado = if n >= 6 { 2 + 4 * ((n - 6) / 4 + 1) } else { 2 };
        for i in 0..n {
            let x0 = x[i];
            let xa = if i >= 1 { x[i - 1] } else { self.x1 };
            let xb = match i {
                0 => self.x2,
                1 => self.x1,
                _ => x[i - 2],
            };
            let ya = if i >= 1 { y[i - 1] } else { self.y1 };
            let yb = match i {
                0 => self.y2,
                1 => self.y1,
                _ => y[i - 2],
            };
            let b1_primeiro = i == 1 || (i >= 2 && i < fim_desenrolado && (i - 2) % 4 == 2);
            let acc = if b1_primeiro {
                b0.mul_add(x0, b1 * xa)
            } else {
                b1.mul_add(xa, b0 * x0)
            };
            let acc = b2.mul_add(xb, acc);
            let acc = (-a1).mul_add(ya, acc);
            y[i] = (-a2).mul_add(yb, acc);
        }
        if n == 1 {
            self.x2 = self.x1;
            self.y2 = self.y1;
        } else {
            self.x2 = x[n - 2];
            self.y2 = y[n - 2];
        }
        self.x1 = x[n - 1];
        self.y1 = y[n - 1];
    }
}

/// Seções em cascata em `f64`, com entrada e saída `f32` (`0x14260d2d0`).
/// As seções alternam dois buffers; a saída é convertida com arredondamento
/// para o mais próximo (`vcvtpd2ps`).
#[derive(Debug, Clone, PartialEq)]
pub struct Cascata {
    pub secoes: Vec<SecaoRbj>,
}

impl Cascata {
    pub fn processar(&mut self, x: &[f32]) -> Vec<f32> {
        let mut a: Vec<f64> = x.iter().map(|&v| f64::from(v)).collect();
        let mut b = vec![0.0; x.len()];
        for s in &mut self.secoes {
            s.processar(&a, &mut b);
            std::mem::swap(&mut a, &mut b);
        }
        a.iter().map(|&v| v as f32).collect()
    }
}

/// Média móvel subtraída e retificada de uma banda (`0x142618b80` e laço de
/// `0x1426080d0`): `soma = (x − janela[i]) + soma`, `janela[i] = x` e
/// `y = max(x − soma/N, 0)`, tudo em `f32`.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaMovel {
    janela: Vec<f32>,
    pos: usize,
    soma: f32,
}

impl MediaMovel {
    /// `N = (int)(taxa·(ms·0,001) + 0,5)`, no mínimo 2.
    pub fn nova(taxa: u32, ms: f32) -> Self {
        let n = ((taxa as f32) * (ms * 0.001) + 0.5) as i32;
        Self {
            janela: vec![0.0; n.max(2) as usize],
            pos: 0,
            soma: 0.0,
        }
    }

    pub fn len(&self) -> usize {
        self.janela.len()
    }

    pub fn is_empty(&self) -> bool {
        self.janela.is_empty()
    }

    pub fn processar_no_lugar(&mut self, y: &mut [f32]) {
        let n = self.janela.len() as f32;
        for v in y.iter_mut() {
            let x = *v;
            self.soma += x - self.janela[self.pos];
            self.janela[self.pos] = x;
            self.pos = (self.pos + 1) % self.janela.len();
            let d = x - self.soma / n;
            *v = if d > 0.0 { d } else { 0.0 };
        }
    }
}

/// Uma banda do sensor: filtra o bloco e retifica, ou copia a saída de
/// outra banda (`+0x54 >= 0`). Opcionalmente aplica a média móvel.
#[derive(Debug, Clone, PartialEq)]
pub struct Banda {
    pub filtro: Cascata,
    pub fonte: Option<usize>,
    pub media: Option<MediaMovel>,
}

/// Banco de bandas do sensor de batidas, na configuração de 6 bandas que
/// `0x142608000` fixa (`[rsp+0x28] = 6`):
///
/// | Banda | Conteúdo |
/// | --- | --- |
/// | 0 | 2 × passa-baixa de 283 Hz (`0x143ea07b0`), Q 0,7071 |
/// | 1 | 2 × passa-faixa de 150 Hz (`0x1458001a8`), 1,5 oitava (`0x1457ff750`) |
/// | 2 | passa-alta de 283 Hz + passa-baixa de 7 900 Hz (`0x143ea0c6c`), duas vezes |
/// | 3 | 2 × passa-alta de 7 900 Hz |
/// | 4 | cópia da banda 0 com média móvel de 1 ms (`0x1457ff570`) |
/// | 5 | cópia da banda 3 com média móvel de 0,2 ms (`0x1457ff168`) |
#[derive(Debug, Clone, PartialEq)]
pub struct BancoDeBandas {
    pub bandas: Vec<Banda>,
    /// Frames por bloco interno: `taxa/75` (`+0x15`).
    pub bloco: usize,
}

impl BancoDeBandas {
    /// Só 44 100 e 48 000 Hz são aceitos pelo analisador (`0x142606880`).
    pub fn novo(taxa: u32) -> Option<Self> {
        if taxa != 44_100 && taxa != 48_000 {
            return None;
        }
        let fs = taxa as f32;
        let q = Q_BUTTERWORTH;
        let cascata = |secoes: Vec<SecaoRbj>| Cascata { secoes };
        let lp283 = SecaoRbj::passa_baixa(283.0, fs, q);
        let bp150 = SecaoRbj::passa_faixa(150.0, fs, 1.5);
        let hp283 = SecaoRbj::passa_alta(283.0, fs, q);
        let lp7900 = SecaoRbj::passa_baixa(7900.0, fs, q);
        let hp7900 = SecaoRbj::passa_alta(7900.0, fs, q);
        let filtrada = |secoes| Banda {
            filtro: cascata(secoes),
            fonte: None,
            media: None,
        };
        let copia = |fonte, ms| Banda {
            filtro: cascata(Vec::new()),
            fonte: Some(fonte),
            media: Some(MediaMovel::nova(taxa, ms)),
        };
        Some(Self {
            bandas: vec![
                filtrada(vec![lp283.clone(), lp283]),
                filtrada(vec![bp150.clone(), bp150]),
                filtrada(vec![hp283.clone(), lp7900.clone(), hp283, lp7900]),
                filtrada(vec![hp7900.clone(), hp7900]),
                copia(0, 1.0),
                copia(3, 0.2),
            ],
            bloco: (taxa / 75) as usize,
        })
    }

    /// Um bloco de até `bloco` amostras mono: saída de cada banda.
    pub fn processar_bloco(&mut self, x: &[f32]) -> Vec<Vec<f32>> {
        let mut saidas: Vec<Vec<f32>> = Vec::with_capacity(self.bandas.len());
        for banda in &mut self.bandas {
            let mut y = match banda.fonte {
                Some(f) => saidas[f].clone(),
                None => banda
                    .filtro
                    .processar(x)
                    .into_iter()
                    .map(f32::abs)
                    .collect(),
            };
            if let Some(m) = &mut banda.media {
                m.processar_no_lugar(&mut y);
            }
            saidas.push(y);
        }
        saidas
    }

    /// Um sinal inteiro, cortado em blocos de `bloco` amostras como no
    /// sensor; as saídas são concatenadas por banda.
    pub fn processar(&mut self, x: &[f32]) -> Vec<Vec<f32>> {
        let mut todas = vec![Vec::with_capacity(x.len()); self.bandas.len()];
        for pedaco in x.chunks(self.bloco.max(1)) {
            for (t, y) in todas.iter_mut().zip(self.processar_bloco(pedaco)) {
                t.extend(y);
            }
        }
        todas
    }
}

/// Alimentação do `Analyzer` (`0x142606e40`), conferida no decompilado.
/// Junta os frames em blocos de `taxa/75` (`+0x18`) e entrega cada bloco
/// cheio. O bloco não é cortado no total de frames (`+0x1c`): o que passa
/// do total entra no último bloco, que sai marcado como `ultimo`.
#[derive(Debug, Clone)]
pub struct Alimentador {
    bloco: usize,
    total: usize,
    processados: usize,
    mono: Vec<f32>,
    estereo: Vec<[i16; 2]>,
}

/// Bloco entregue pela alimentação: mono `(L + R)·0,5/32 768` em `f32`
/// (`+0x30`) e estéreo intercalado `(i16)(i32)` (`+0x38`).
pub struct BlocoAlimentado<'a> {
    pub mono: &'a [f32],
    pub estereo: &'a [[i16; 2]],
    /// Verdadeiro quando o total de frames foi atingido: o original chama o
    /// fechamento `0x142607110` em seguida.
    pub ultimo: bool,
}

impl Alimentador {
    pub fn novo(taxa: u32, total_frames: usize) -> Self {
        let bloco = (taxa / 75) as usize;
        Self {
            bloco,
            total: total_frames,
            processados: 0,
            mono: Vec::with_capacity(bloco),
            estereo: Vec::with_capacity(bloco),
        }
    }

    /// Consome `l`/`r` (mesmo comprimento) e chama `ao_bloco` a cada bloco.
    /// Devolve verdadeiro quando o total foi atingido; o resto da entrada é
    /// ignorado, como no original.
    pub fn alimentar(
        &mut self,
        l: &[f32],
        r: &[f32],
        mut ao_bloco: impl FnMut(BlocoAlimentado<'_>),
    ) -> bool {
        let n = l.len().min(r.len());
        let mut lido = 0;
        while lido < n {
            let cabe = (self.bloco - self.mono.len()).min(n - lido);
            for (&a, &b) in l[lido..lido + cabe].iter().zip(&r[lido..lido + cabe]) {
                self.mono.push((a + b) * ESCALA_MONO);
                self.estereo.push([(a as i32) as i16, (b as i32) as i16]);
            }
            lido += cabe;
            self.processados += cabe;
            let ultimo = self.total <= self.processados;
            if self.mono.len() == self.bloco || ultimo {
                ao_bloco(BlocoAlimentado {
                    mono: &self.mono,
                    estereo: &self.estereo,
                    ultimo,
                });
                self.mono.clear();
                self.estereo.clear();
            }
            if ultimo {
                return true;
            }
        }
        false
    }
}

/// Trincas `(mín ms, máx ms, razão em batidas)` de `0x143ea0b40`.
const FAIXAS_X: [(f32, f32, f32); 9] = [
    (500.0, 1276.0, 1.5),
    (468.0, 1071.0, 1.25),
    (923.0, 2142.0, 2.5),
    (290.0, 638.0, 0.75),
    (320.0, 860.0, 1.0),
    (290.0, 428.0, 0.5),
    (666.0, 1714.0, 2.0),
    (1500.0, 3333.0, 4.0),
    (1125.0, 2580.0, 3.0),
];

/// As mesmas trincas de `0x143ea0bb0`, em outra ordem.
const FAIXAS_HIST: [(f32, f32, f32); 9] = [
    (1500.0, 3333.0, 4.0),
    (923.0, 2142.0, 2.5),
    (666.0, 1714.0, 2.0),
    (500.0, 1276.0, 1.5),
    (468.0, 1071.0, 1.25),
    (320.0, 860.0, 1.0),
    (290.0, 638.0, 0.75),
    (290.0, 428.0, 0.5),
    (1125.0, 2580.0, 3.0),
];

/// Tolerância `min(u·0,04, 20)` calculada em `f64` (`0x1438264d0`,
/// `0x1457ffb70`) e levada a `f32`.
fn tolerancia(u: f32) -> f32 {
    let t = f64::from(u) * 0.04;
    (if 20.0 < t { 20.0 } else { t }) as f32
}

/// Uma entrada do anel de intervalos: `(posição do pico, pico, intervalo em ms)`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Intervalo {
    pub pos: i32,
    pub pico: f32,
    pub ms: f32,
}

/// Resultado de um casamento de `0x1426120b0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Casamento {
    /// Linha de `FAIXAS_X` a partir da qual a próxima busca recomeça.
    pub proxima: usize,
    /// Intervalo médio por batida, em ms.
    pub media: f32,
    pub erro: f32,
    /// Intervalos do anel usados.
    pub contagem: i32,
}

/// Registro de intervalos de uma banda (0x40 bytes; construtor
/// `0x1426188a0`): anel de 32 entradas e o melhor agrupamento.
#[derive(Debug, Clone, PartialEq)]
pub struct RegistroDeIntervalos {
    pub anel: [Intervalo; 32],
    /// Próxima posição de escrita (`+0x10`).
    pub escrita: i32,
    /// Entradas válidas (`+0x14`).
    pub quantidade: i32,
    /// Posição do último pico (`+0x20`).
    pub ultimo_pico: i32,
    /// Intervalo do último pico ou média do agrupamento (`+4`).
    pub intervalo: f32,
    /// Erro (`+0x30`) e contagem (`+0x34`) do melhor agrupamento.
    pub erro: f32,
    pub contagem: i32,
    /// Agrupamento válido neste passo (`+0x2d`).
    pub valido: bool,
    /// Pico (`+0x38`) e posição (`+0x3c`) do último agrupamento válido.
    pub pico: f32,
    pub pos_pico: i32,
}

impl Default for RegistroDeIntervalos {
    fn default() -> Self {
        Self {
            anel: [Intervalo::default(); 32],
            escrita: 0,
            quantidade: 0,
            ultimo_pico: 0,
            intervalo: 0.0,
            erro: f32::MAX,
            contagem: 0,
            valido: false,
            pico: 0.0,
            pos_pico: 0,
        }
    }
}

impl RegistroDeIntervalos {
    const CAP: i32 = 32;

    /// Entrada `j` do anel, da mais recente (0) para trás.
    fn recente(&self, j: i32) -> Intervalo {
        self.anel[((self.escrita - j - 1 + Self::CAP) % Self::CAP) as usize]
    }

    /// Casamento de `x` (ms) com o anel a partir da linha `inicio`
    /// (`0x1426120b0`, conferido no assembly):
    ///
    /// 1. Linha `k >= inicio` de `FAIXAS_X` com `mín < x < máx`; razão `r`,
    ///    unidade `u = x/r`, tolerância `min(u·0,04, 20)` e alvo `T = 2f` se
    ///    `f <= r`, senão `f` (`f = 4`).
    /// 2. Do mais recente para trás: o intervalo `y` precisa casar com a
    ///    primeira linha de `FAIXAS_HIST` com `mín < y < máx` e
    ///    `|y/r₂ − u| < tolerância`. Soma `r₂` às batidas e `y` ao total.
    ///    Batidas iguais a `T` encerram. Se passarem de `T`, `T` dobra, desde
    ///    que `f < T < 4f` e batidas `< 4f`. Para antes de usar a entrada `j`
    ///    se `j + 1 >= quantidade` ou `j + 2 >= 32`.
    /// 3. Falhou: tenta a próxima linha de `FAIXAS_X`.
    /// 4. Casou: `média = total/T`, `erro = |média − u|`. Estende para trás
    ///    enquanto `|y/q − média| < min(média·0,04, 20)`, com
    ///    `q = (int)fma(y/média, 4, 0,5)/4`, arredondado ao inteiro acima de
    ///    2,5 e limitado a 4.
    #[allow(clippy::neg_cmp_op_on_partial_ord)] // segue os saltos do `comiss` com NaN
    pub fn casar(&self, x: f32, inicio: usize) -> Option<Casamento> {
        let f = 4.0f32;
        let mut linha = inicio;
        'linhas: while linha < 9 {
            let (lo, hi, r) = FAIXAS_X[linha];
            linha += 1;
            if !(x > lo && x < hi) {
                continue;
            }
            let u = x / r;
            let tol = tolerancia(u);
            let mut alvo = if f <= r { f + f } else { f };
            let (mut batidas, mut total) = (r, x);
            let mut j = 0i32;
            loop {
                let y = self.recente(j).ms;
                let mut casou = None;
                for &(lo2, hi2, r2) in &FAIXAS_HIST {
                    if y > lo2 && y < hi2 && ((y / r2) - u).abs() < tol {
                        casou = Some(r2);
                        break;
                    }
                }
                let Some(r2) = casou else { continue 'linhas };
                batidas += r2;
                total += y;
                if batidas == alvo {
                    break;
                }
                if batidas > alvo {
                    let quatro_f = f64::from(f) * 4.0;
                    if !(alvo > f)
                        || !(f64::from(alvo) < quatro_f)
                        || !(f64::from(batidas) < quatro_f)
                    {
                        continue 'linhas;
                    }
                    alvo += alvo;
                } else if !(batidas < alvo) {
                    continue 'linhas;
                }
                j += 1;
                if j + 1 >= self.quantidade || j + 2 >= Self::CAP {
                    continue 'linhas;
                }
            }
            let media = total / alvo;
            let erro = (media - u).abs();
            let tol2 = tolerancia(media);
            let inv = 1.0 / media;
            let mut c = j + 1;
            loop {
                let antes = c + 2;
                let y = self.recente(c).ms;
                let mut q = ((inv * y).mul_add(4.0, 0.5) as i32) as f32 * 0.25;
                if q > 2.5 {
                    q = ((q + 0.5) as i32) as f32;
                }
                let q = if q < 4.0 { q } else { 4.0 };
                if !(((y / q) - media).abs() < tol2) {
                    break;
                }
                c += 1;
                if antes >= self.quantidade || antes + 1 >= Self::CAP {
                    break;
                }
            }
            return Some(Casamento {
                proxima: linha,
                media,
                erro,
                contagem: c,
            });
        }
        None
    }

    /// Agrupamento de `0x142611f20`: casa `x` a partir de cada linha e guarda
    /// o de maior contagem cuja média seja `>= 333 ms` (`0x143ea0c68`).
    pub fn agrupar(&mut self, x: f32) -> bool {
        let mut achou = false;
        let mut melhor = 0;
        let mut inicio = 0;
        while let Some(c) = self.casar(x, inicio) {
            if c.media >= 333.0 && melhor < c.contagem {
                self.intervalo = c.media;
                self.erro = c.erro;
                self.contagem = c.contagem;
                melhor = c.contagem;
                achou = true;
            }
            inicio = c.proxima;
        }
        achou
    }
}

/// Seguidor de pico de uma banda (0x78 bytes; construtor `0x142618950`,
/// init em `0x142612650`), conferido no assembly do laço de `0x1426080d0`.
/// Roda a cada `passo` amostras sobre a média da banda.
#[derive(Debug, Clone, PartialEq)]
pub struct SeguidorDePico {
    aberto: bool,
    /// Tempo mínimo sem renovar para fechar o pico: 0,29 s (`+4`).
    fecha: u32,
    limiar: f32,
    inclinacao: f32,
    queda: f32,
    piso: f32,
    fase: i32,
    espera: i32,
    fator_limiar: f32,
    fator_pico: f32,
    fator_queda: f32,
    /// 1,7 s (`+0x2c`).
    espera_pico: i32,
    /// 1,3 s (`+0x30`, `+0x34`).
    duracoes: [u32; 2],
    marco: i32,
    pub pos: i32,
    passo: i32,
    pico: f32,
    pos_pico: i32,
    pos_renovado: i32,
}

impl SeguidorDePico {
    pub fn novo(taxa: u32, passo: i32, inicio: i32) -> Self {
        let fs = taxa as f32;
        let d = (fs * 1.3) as i32 as u32;
        Self {
            aberto: false,
            fecha: (fs * 0.29) as i32 as u32,
            limiar: 0.001,
            inclinacao: 0.0,
            queda: 0.0,
            piso: 0.001,
            fase: 0,
            espera: 0,
            fator_limiar: 0.75,
            fator_pico: 0.5,
            fator_queda: 0.5,
            espera_pico: (fs * 1.7) as i32,
            duracoes: [d, d],
            marco: 0,
            pos: inicio,
            passo,
            pico: 0.0,
            pos_pico: 0,
            pos_renovado: 0,
        }
    }

    fn abrir(&mut self, v: f32, pos: i32) {
        self.limiar = v * self.fator_limiar;
        self.pico = v;
        self.pos_pico = pos;
        self.pos_renovado = pos;
        self.espera = self.espera_pico;
        self.fase = 0;
        self.inclinacao = 0.0;
        self.queda = 0.0;
        self.marco = pos;
    }

    /// Um passo com a média `v`. Devolve `(posição, pico)` quando um pico
    /// fecha.
    ///
    /// - O limiar cai `queda` por passo. Fechado: o piso é 0,001. Passadas
    ///   `espera` amostras desde o marco, a queda é recalculada: primeiro
    ///   rumo a `0,5·pico` em 1,3 s, depois rumo a `0,5·limiar` em 1,3 s,
    ///   com `limiar − fator·x` fundido. `v >= limiar` abre um pico.
    /// - Aberto: um valor maior renova o pico. O pico fecha após 0,29 s sem
    ///   renovar (comparação sem sinal).
    pub fn passo(&mut self, v: f32) -> Option<(i32, f32)> {
        let pos = self.pos + self.passo;
        self.pos = pos;
        let mut limiar = self.limiar - self.queda;
        self.limiar = limiar;
        if !self.aberto {
            if limiar < self.piso {
                self.limiar = self.piso;
                limiar = self.piso;
            }
            if pos - self.marco > self.espera {
                let alvo = match self.fase {
                    0 => {
                        self.fase = 1;
                        Some((
                            (-self.fator_pico).mul_add(self.pico, limiar),
                            self.duracoes[0],
                        ))
                    }
                    1 => Some((
                        (-limiar).mul_add(self.fator_queda, limiar),
                        self.duracoes[1],
                    )),
                    _ => None,
                };
                if let Some((delta, dur)) = alvo {
                    let inclinacao = delta / dur as f32;
                    self.queda = self.passo as f32 * inclinacao;
                    self.inclinacao = inclinacao;
                    self.marco = pos;
                    self.espera = dur as i32;
                }
            }
            if v >= limiar {
                self.abrir(v, pos);
                self.aberto = true;
            }
            return None;
        }
        if v > self.pico {
            self.abrir(v, pos);
        }
        if (pos - self.pos_renovado) as u32 >= self.fecha {
            self.aberto = false;
            return Some((self.pos_pico, self.pico));
        }
        None
    }
}

/// Sensor de batidas do motor 1 (`0x1426080d0`), etapa 2: banco de bandas,
/// média de 8 amostras por banda, seguidor de pico e registro de intervalos.
/// A seleção da banda (`0x142608b20`) e a lista de batidas (`0x142607b60`)
/// ainda não estão portadas; o gancho `ao_passo` marca onde entram.
#[derive(Debug, Clone)]
pub struct SensorDeBatidas {
    pub banco: BancoDeBandas,
    pub seguidores: Vec<SeguidorDePico>,
    pub registros: Vec<RegistroDeIntervalos>,
    pub histograma: HistogramaBpm,
    pub lista: ListaDeBatidas,
    /// Última banda escolhida: `(intervalo, posição, pico)` em
    /// `[0x10]`–`[0x12]`.
    pub selecao: Option<(f32, i32, f32)>,
    /// Amostras desde a última escolha (`[0x13]`).
    pub desde_selecao: i32,
    acumulado: Vec<f32>,
    na_media: i32,
    passo: i32,
    amostras: i32,
    fim: i32,
    taxa: u32,
}

impl SensorDeBatidas {
    /// `total_frames` é o fim da janela de contagem (`[0x18]` = `+0x1c` do
    /// analisador); o início é 0 e a média tem 8 amostras.
    pub fn novo(taxa: u32, total_frames: i32) -> Option<Self> {
        let banco = BancoDeBandas::novo(taxa)?;
        let n = banco.bandas.len();
        Some(Self {
            banco,
            seguidores: vec![SeguidorDePico::novo(taxa, 8, 0); n],
            registros: vec![RegistroDeIntervalos::default(); n],
            histograma: HistogramaBpm::default(),
            lista: ListaDeBatidas::default(),
            selecao: None,
            desde_selecao: 0,
            acumulado: vec![0.0; n],
            na_media: 0,
            passo: 8,
            amostras: 0,
            fim: total_frames,
            taxa,
        })
    }

    /// Consome um bloco mono, cortado em pedaços de `taxa/75`. A cada passo
    /// de 8 amostras roda os seguidores, escolhe a banda
    /// (`HistogramaBpm::escolher`) e grava as batidas das bandas válidas
    /// (`ListaDeBatidas::registrar`). Devolve verdadeiro se houve escolha.
    pub fn alimentar(&mut self, x: &[f32]) -> bool {
        let mut escolheu = false;
        self.desde_selecao += x.len() as i32;
        for pedaco in x.chunks(self.banco.bloco.max(1)) {
            if self.fim <= self.amostras {
                return escolheu;
            }
            let bandas = self.banco.processar_bloco(pedaco);
            for k in 0..pedaco.len() {
                self.amostras += 1;
                if self.fim <= self.amostras {
                    break;
                }
                for (a, b) in self.acumulado.iter_mut().zip(&bandas) {
                    *a += b[k];
                }
                self.na_media += 1;
                if self.na_media < self.passo {
                    continue;
                }
                self.na_media = 0;
                for b in 0..self.acumulado.len() {
                    let v = self.acumulado[b] / self.passo as f32;
                    self.acumulado[b] = 0.0;
                    let reg = &mut self.registros[b];
                    reg.valido = false;
                    let Some((pos, pico)) = self.seguidores[b].passo(v) else {
                        continue;
                    };
                    let anterior = reg.ultimo_pico;
                    reg.ultimo_pico = pos;
                    let ms =
                        (1000.0 / self.taxa as f32) * (pos.wrapping_sub(anterior) as u32 as f32);
                    let ms = if anterior == 0 {
                        0.0
                    } else {
                        reg.intervalo = ms;
                        reg.valido = reg.agrupar(ms);
                        ms
                    };
                    if reg.quantidade < RegistroDeIntervalos::CAP {
                        reg.quantidade += 1;
                    }
                    reg.anel[reg.escrita as usize] = Intervalo { pos, pico, ms };
                    reg.escrita = (reg.escrita + 1) % RegistroDeIntervalos::CAP;
                    if reg.valido {
                        reg.pico = pico;
                        reg.pos_pico = pos;
                    }
                }
                let Some(melhor) = self.histograma.escolher(&mut self.registros) else {
                    continue;
                };
                let r = &self.registros[melhor];
                self.selecao = Some((r.intervalo, r.pos_pico, r.pico));
                self.desde_selecao = 0;
                for (banda, r) in self.registros.iter().enumerate() {
                    if r.valido {
                        self.lista.registrar(r.intervalo, r, banda as i32);
                    }
                }
                escolheu = true;
            }
        }
        escolheu
    }
}

/// `(a, 2a, 0,5a)` perto de `b` dentro de `tol`. A terceira conta é fundida
/// (`vfmsub231ss`) onde `fundida` é verdadeiro.
fn perto_em_oitava(a: f32, b: f32, tol: f32, fundida: bool) -> bool {
    let meio = if fundida {
        a.mul_add(0.5, -b)
    } else {
        a * 0.5 - b
    };
    (a - b).abs() <= tol || ((a + a) - b).abs() <= tol || meio.abs() <= tol
}

/// Histograma de BPM do sensor (`+0x64`, 256 caixas) e escolha da banda
/// (`0x142608b20`), conferidos no assembly.
#[derive(Debug, Clone, PartialEq)]
pub struct HistogramaBpm {
    pub caixas: Vec<i32>,
    /// Menor (`+0x464`, início 200) e maior (`+0x468`, início 40) caixa usada.
    pub minimo: i32,
    pub maximo: i32,
}

impl Default for HistogramaBpm {
    fn default() -> Self {
        Self {
            caixas: vec![0; 256],
            minimo: 200,
            maximo: 40,
        }
    }
}

impl HistogramaBpm {
    /// Primeira caixa com mais votos em `minimo..=maximo`, ou −1.
    fn moda(&self) -> i32 {
        let (mut m, mut votos) = (-1, 0);
        for b in self.minimo..=self.maximo {
            let c = self.caixas.get(b as usize).copied().unwrap_or(0);
            if c > votos {
                m = b;
                votos = c;
            }
        }
        m
    }

    fn votos(&self, b: i32) -> i32 {
        usize::try_from(b)
            .ok()
            .and_then(|i| self.caixas.get(i))
            .copied()
            .unwrap_or(0)
    }

    /// Escolhe a banda do passo.
    ///
    /// 1. Cada banda válida vota em `(int)(60000/intervalo + 0,5)` com peso
    ///    `max((int)(4/(1 + erro)), 1)`. A melhor é a de menor erro.
    /// 2. Moda `m`. Se o intervalo da melhor não estiver perto de
    ///    `60000/m`, do dobro ou da metade (`min(intervalo·0,04, 20)`), e a
    ///    caixa da melhor tiver ao menos 3 votos, zera as caixas
    ///    incompatíveis com a melhor e recalcula a moda. Com menos de 3
    ///    votos, não há escolha.
    /// 3. Exige moda com ao menos 3 votos. Invalida as bandas incompatíveis
    ///    com a moda e devolve a melhor.
    pub fn escolher(&mut self, registros: &mut [RegistroDeIntervalos]) -> Option<usize> {
        let mut melhor: Option<(usize, f32, i32)> = None;
        let mut menor_erro = f32::MAX;
        for (i, r) in registros.iter().enumerate() {
            if !r.valido {
                continue;
            }
            let caixa = (60000.0 / r.intervalo + 0.5) as i32;
            let peso = (4.0 / (1.0 + r.erro)) as i32;
            let slot = self.caixas.get_mut(usize::try_from(caixa).ok()?)?;
            *slot += if peso > 0 { peso } else { 1 };
            self.minimo = self.minimo.min(caixa);
            self.maximo = self.maximo.max(caixa);
            if r.erro < menor_erro {
                menor_erro = r.erro;
                melhor = Some((i, r.intervalo, caixa));
            }
        }
        let mut m = self.moda();
        let mut modo = 60000.0 / m as f32;
        let (i, iv, caixa) = melhor?;
        let tol = tolerancia(iv);
        let mut escolhida = Some(i);
        while m > 0 {
            if (modo - iv).abs() <= tol {
                break;
            }
            if ((modo + modo) - iv).abs() <= tol || modo.mul_add(0.5, -iv).abs() <= tol {
                break;
            }
            if self.votos(caixa) < 3 {
                escolhida = None;
                break;
            }
            let (lo, hi) = (self.minimo, self.maximo);
            self.minimo = 200;
            self.maximo = 40;
            for b in lo..=hi {
                let mb = 60000.0 / b as f32;
                let Some(c) = usize::try_from(b).ok().and_then(|k| self.caixas.get_mut(k)) else {
                    continue;
                };
                if !perto_em_oitava(mb, iv, tol, false) {
                    *c = 0;
                } else if *c != 0 {
                    self.minimo = self.minimo.min(b);
                    self.maximo = self.maximo.max(b);
                }
            }
            m = self.moda();
            modo = 60000.0 / m as f32;
        }
        if m < 0 || self.votos(m) < 3 {
            return None;
        }
        let i = escolhida?;
        for r in registros.iter_mut().filter(|r| r.valido) {
            if !perto_em_oitava(modo, r.intervalo, tolerancia(r.intervalo), true) {
                r.valido = false;
            }
        }
        Some(i)
    }
}

/// Batida candidata da lista do sensor (24 bytes em `+0xb0`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BatidaSensor {
    pub pos: i32,
    /// Intervalo da entrada em quartos de batida (`+4`).
    pub batidas: f32,
    /// Intervalo médio do agrupamento, em ms (`+8`).
    pub intervalo: f32,
    /// Etiqueta `geração·256 + força` (`+0xc`).
    pub etiqueta: u32,
    pub banda: i32,
}

/// Lista de batidas do sensor, gravada por `BeatListSignal::[1]`
/// (`0x142607b60`), conferida no assembly. Ordenada por posição.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListaDeBatidas {
    pub batidas: Vec<BatidaSensor>,
    /// `+0x1c` do objeto: byte baixo = força do grupo atual, resto = geração.
    pub etiqueta: u32,
}

impl ListaDeBatidas {
    /// Grava as `contagem + 2` entradas do anel do registro, da mais antiga
    /// para a mais recente.
    ///
    /// - `batidas = (int)fma(ms/intervalo, 4, 0,5)/4`.
    /// - Inserção após as de posição menor ou igual. Para trás, entre as de
    ///   mesma posição: se o BPM×10 `(int)(600000/iv + 0,5)` coincidir (ou
    ///   dobrar ou cair à metade) com `(int)fma(600000, 1/intervalo, 0,5)`,
    ///   ou se os intervalos forem incompatíveis (`min(intervalo·0,04, 20)`
    ///   em `iv`, `2iv` e `0,5iv`), a mais forte fica: a antiga sai se a
    ///   força atual for maior, senão a nova não entra. Nos outros casos, a
    ///   busca continua.
    /// - Ao fim, a geração avança e a força volta a 0.
    pub fn registrar(&mut self, intervalo: f32, reg: &RegistroDeIntervalos, banda: i32) {
        let n = reg.contagem + 1;
        self.etiqueta = self.etiqueta.wrapping_add((n + 1) as u32);
        let forca = self.etiqueta as u8;
        let inv = 1.0 / intervalo;
        let tol = tolerancia(intervalo);
        let bpm10 = 600_000.0f32.mul_add(inv, 0.5) as i32;
        for antes in (0..=n).rev() {
            let e = reg.recente(antes);
            let nova = BatidaSensor {
                pos: e.pos,
                batidas: ((inv * e.ms).mul_add(4.0, 0.5) as i32) as f32 * 0.25,
                intervalo,
                etiqueta: self.etiqueta,
                banda,
            };
            let mut idx = self.batidas.len();
            while idx > 0 && self.batidas[idx - 1].pos > e.pos {
                idx -= 1;
            }
            let mut entra = true;
            let mut t = idx;
            while t > 0 && self.batidas[t - 1].pos == e.pos {
                let velha = self.batidas[t - 1];
                let b1 = (600_000.0 / velha.intervalo + 0.5) as i32;
                let mesmo_tempo = b1 == bpm10 || b1 * 2 == bpm10 || b1 == bpm10 * 2;
                if mesmo_tempo || !perto_em_oitava(velha.intervalo, intervalo, tol, true) {
                    if forca > velha.etiqueta as u8 {
                        self.batidas.remove(t - 1);
                        idx -= 1;
                    } else {
                        entra = false;
                    }
                    break;
                }
                t -= 1;
            }
            if entra {
                self.batidas.insert(idx, nova);
            }
        }
        self.etiqueta = (self.etiqueta & 0xffff_ff00).wrapping_add(0x100);
    }
}

/// `peakss::BpmCountTbl`: histograma de BPM com `decimais` casas
/// (construída em `0x142613100`; contagem `0x142612e10`, refino
/// `0x142612fb0`), conferida no assembly.
#[derive(Debug, Clone, PartialEq)]
pub struct TabelaBpm {
    pub caixas: Vec<i32>,
    pub decimais: i32,
    /// `10^decimais` (`pow`, `0x14299fd40`).
    pub escala: f64,
}

impl TabelaBpm {
    pub fn nova(n: usize, decimais: i32) -> Self {
        Self {
            caixas: vec![0; n],
            decimais,
            escala: 10f64.powi(decimais),
        }
    }

    /// Caixa de um intervalo: `(int)fma(600000/iv, escala, 5)/10`.
    fn caixa(&self, iv: f32) -> i32 {
        (600_000.0 / f64::from(iv)).mul_add(self.escala, 5.0) as i32 / 10
    }

    fn somar(&mut self, iv: f32) {
        let b = self.caixa(iv);
        if let Some(c) = usize::try_from(b).ok().and_then(|b| self.caixas.get_mut(b)) {
            *c += 1;
        }
    }

    /// Zera e conta a partir de `inicio` as batidas a até `janela` amostras
    /// da primeira. Devolve o índice da primeira que ficou de fora.
    ///
    /// ADAPTAÇÃO: a guarda da primeira batida no original é
    /// `(b >= 0) || (b < n)`, sempre verdadeira, e pode escrever fora do
    /// vetor. Aqui caixas fora de `0..n` são ignoradas, como nas demais.
    pub fn contar(&mut self, lista: &[BatidaSensor], inicio: usize, janela: i32) -> usize {
        self.caixas.fill(0);
        let Some(base) = lista.get(inicio) else {
            return inicio + 1;
        };
        self.somar(base.intervalo);
        let mut i = inicio + 1;
        while i < lista.len() {
            if lista[i].pos - base.pos > janela {
                break;
            }
            self.somar(lista[i].intervalo);
            i += 1;
        }
        i
    }

    /// Primeiro máximo estrito de `lo..hi`, ou −1 se tudo for 0.
    fn primeiro_maximo(&self, lo: i32, hi: i32) -> i32 {
        let (mut m, mut votos) = (-1, 0);
        for b in lo.max(0)..hi.min(self.caixas.len() as i32) {
            let c = self.caixas[b as usize];
            if c > votos {
                m = b;
                votos = c;
            }
        }
        m
    }

    /// Refino de uma caixa vinda de uma tabela com `decimais` casas: busca
    /// em `[(int)((s·b − s)·10^−d), (int)((s·b + s)·10^−d))`, com FMA, ou na
    /// tabela inteira se `decimais >= self.decimais`.
    pub fn refinar(&self, caixa: i32, decimais: i32) -> i32 {
        if decimais >= self.decimais {
            return self.primeiro_maximo(0, self.caixas.len() as i32);
        }
        let s = self.escala;
        let b = f64::from(caixa);
        let f = 10f64.powf(-f64::from(decimais));
        let lo = (s.mul_add(b, -s) * f) as i32;
        let lo = if lo <= 0 { 0 } else { lo };
        let hi = (s.mul_add(b, s) * f) as i32;
        let hi = hi.min(self.caixas.len() as i32);
        self.primeiro_maximo(lo, hi)
    }
}

/// `x` dentro de `(c − t, c + t)` como intervalo, dobro ou metade, com
/// comparações estritas e a metade sem FMA (laço de poda de `0x142613100`).
fn na_janela(x: f32, c: f32, t: f32) -> bool {
    let (lo, hi) = (c - t, t + c);
    let dentro = |v: f32| lo < v && v < hi;
    dentro(x) || dentro(x + x) || dentro(x * 0.5)
}

/// Primeira estimativa do modo 0 (flag `+0x2c` = 0) de `0x142613100`,
/// conferida no assembly:
///
/// 1. `TabelaBpm` de 400 caixas (BPM inteiro) sobre as batidas a até
///    `janela` amostras da primeira; primeiro máximo.
/// 2. `TabelaBpm` de 4 000 caixas (0,1 BPM), refinada a ±1 BPM em torno do
///    primeiro máximo; `iv0 = (f32)(600000/caixa)`.
/// 3. Poda: a batida fica se o intervalo dela (ou o dobro, ou a metade)
///    estiver a menos de `min(iv0·0,04, 20)` de `iv0`, ou a menos da mesma
///    tolerância do intervalo da batida anterior. Na primeira batida, o
///    "anterior" é a própria tolerância, como no original.
/// 4. Devolve `iv0` e a primeira batida restante com o intervalo trocado
///    por `iv0`: o primeiro segmento da etapa seguinte.
pub fn estimar_modo0(lista: &mut Vec<BatidaSensor>, janela: i32) -> Option<(f32, BatidaSensor)> {
    if lista.is_empty() {
        return None;
    }
    let mut grossa = TabelaBpm::nova(400, 0);
    grossa.contar(lista, 0, janela);
    let g = grossa.primeiro_maximo(0, 400);
    if g < 0 {
        return None;
    }
    let mut fina = TabelaBpm::nova(4000, 1);
    fina.contar(lista, 0, janela);
    let f = fina.refinar(g, 0);
    if f < 0 {
        return None;
    }
    let iv0 = (600_000.0 / f64::from(f)) as f32;
    let tol0 = tolerancia(iv0);
    let mut anterior = tol0;
    let mut i = 0;
    while i < lista.len() {
        let iv = lista[i].intervalo;
        let fica = na_janela(iv, iv0, tol0) || na_janela(iv, anterior, tolerancia(anterior));
        anterior = iv;
        if fica {
            i += 1;
        } else {
            lista.remove(i);
        }
    }
    let primeira = *lista.first()?;
    Some((
        iv0,
        BatidaSensor {
            intervalo: iv0,
            ..primeira
        },
    ))
}

/// Segmento de tempo de `0x142613100` (registro de 24 bytes em
/// `local_210`). Nasce de uma `BatidaSensor`; o refino troca `+4` por 0 e
/// `+0xc` por 4. Os campos crus são mantidos até o significado de `+4`,
/// `+0xc` e `+0x10` ser lido nas etapas seguintes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segmento {
    pub pos: i32,
    pub campo4: u32,
    pub intervalo: f32,
    pub campo_c: u32,
    pub campo10: i32,
}

impl From<BatidaSensor> for Segmento {
    fn from(b: BatidaSensor) -> Self {
        Self {
            pos: b.pos,
            campo4: b.batidas.to_bits(),
            intervalo: b.intervalo,
            campo_c: b.etiqueta,
            campo10: b.banda,
        }
    }
}

/// Refino por segmento do trecho comum de `0x142613100` (após
/// `LAB_14261448b`), conferido no assembly. Para cada segmento, conta as
/// batidas da lista a partir da primeira na posição do segmento até o
/// próximo segmento (o último vai até `total`), com as mesmas tabelas de 400
/// e 4 000 caixas da estimativa. Achando a caixa fina, grava
/// `intervalo = (f32)(600000/caixa)`, `campo4 = 0` e `campo_c = 4`. Se o
/// novo intervalo estiver na janela do segmento anterior (`na_janela`), o
/// segmento é removido e o anterior é recontado com a janela estendida.
///
/// Segmento sem batida na mesma posição: o original lê `lista[−1]`; aqui o
/// segmento fica como está.
pub fn refinar_segmentos(lista: &[BatidaSensor], segmentos: &mut Vec<Segmento>, total: i32) {
    let mut grossa = TabelaBpm::nova(400, 0);
    let mut fina = TabelaBpm::nova(4000, 1);
    let mut i = 0;
    while i < segmentos.len() {
        let janela = if i + 1 == segmentos.len() {
            total
        } else {
            segmentos[i + 1].pos - segmentos[i].pos
        };
        let Some(inicio) = lista.iter().position(|b| b.pos == segmentos[i].pos) else {
            i += 1;
            continue;
        };
        grossa.contar(lista, inicio, janela);
        let g = grossa.primeiro_maximo(0, 400);
        if g < 0 {
            i += 1;
            continue;
        }
        fina.contar(lista, inicio, janela);
        let f = fina.refinar(g, 0);
        if f < 0 {
            i += 1;
            continue;
        }
        let s = &mut segmentos[i];
        s.intervalo = (600_000.0 / f64::from(f)) as f32;
        s.campo4 = 0;
        s.campo_c = 4;
        if i >= 1 {
            let anterior = segmentos[i - 1].intervalo;
            if na_janela(segmentos[i].intervalo, anterior, tolerancia(anterior)) {
                segmentos.remove(i);
                i -= 1;
                continue;
            }
        }
        i += 1;
    }
}

/// Valores iniciais dos 6 doubles por segmento (linhas 953–994):
/// `[0x143ea0ca0, ca8, cb0, cb8, c80, c88]`, lidos do executável.
pub const PONTUACAO_INICIAL: [f64; 6] = [-30.0, -20.0, -3.0, -3.0, -20.0, -3.0];

/// Deduplicação da lista por força (linhas 996–1104 de `0x142613100`),
/// conferida no assembly.
///
/// 1. `força[geração]` (geração = `etiqueta >> 8`, com sinal): o byte baixo
///    da primeira etiqueta encontrada daquela geração; uma força 0 é
///    sobrescrita pela próxima.
/// 2. Uma passada só pela lista, a partir da segunda batida, com a primeira
///    como "anterior". Para cada segmento, a distância mínima é
///    `(int)(((f32)taxa · iv) · 1,25e−4)` (`0x143ea0c30`) e o fim é o
///    próximo segmento (o último vai até `total`).
/// 3. Duas batidas mais perto que a distância: sai a anterior se a força
///    dela for menor que a da atual (a atual vira a anterior); senão sai a
///    atual.
pub fn deduplicar(lista: &mut Vec<BatidaSensor>, segmentos: &[Segmento], taxa: i32, total: i32) {
    let geracao = |b: &BatidaSensor| (b.etiqueta as i32) >> 8;
    let mut forca: Vec<i32> = Vec::new();
    for b in lista.iter() {
        let Ok(g) = usize::try_from(geracao(b)) else {
            continue;
        };
        if forca.len() <= g {
            forca.resize(g + 1, 0);
        }
        if forca[g] == 0 {
            forca[g] = i32::from(b.etiqueta as u8);
        }
    }
    let f = |g: i32| {
        usize::try_from(g)
            .ok()
            .and_then(|g| forca.get(g))
            .copied()
            .unwrap_or(0)
    };
    let Some(&primeira) = lista.first() else {
        return;
    };
    let mut anterior = primeira;
    let mut i = 1;
    for (j, s) in segmentos.iter().enumerate() {
        let distancia = ((taxa as f32 * s.intervalo) * 1.25e-4) as i32;
        let fim = segmentos.get(j + 1).map_or(total, |p| p.pos);
        while i < lista.len() {
            let b = lista[i];
            if b.pos >= fim {
                break;
            }
            if (anterior.pos - b.pos).abs() < distancia {
                if f(geracao(&anterior)) < f(geracao(&b)) {
                    lista.remove(i - 1);
                    anterior = b;
                } else {
                    lista.remove(i);
                }
            } else {
                anterior = b;
                i += 1;
            }
        }
    }
}

/// BPM do segmento mais longo (linhas 1124–1205): `60000/intervalo` do
/// primeiro segmento de maior extensão. O primeiro conta de 0 e o último
/// vai até `total`. Sem segmentos, 0.
pub fn bpm_do_mais_longo(segmentos: &[Segmento], total: i32) -> f32 {
    let (mut maior, mut bpm) = (0, 0.0f32);
    for (j, s) in segmentos.iter().enumerate() {
        let inicio = if j == 0 { 0 } else { s.pos };
        let fim = segmentos.get(j + 1).map_or(total, |p| p.pos);
        if maior < fim - inicio {
            bpm = 60000.0 / s.intervalo;
            maior = fim - inicio;
        }
    }
    bpm
}

/// Último índice com `pos <= v` (busca binária com meio truncado), ou −1.
fn ultimo_ate(lista: &[BatidaSensor], v: i32) -> i32 {
    let (mut lo, mut hi) = (0i32, lista.len() as i32 - 1);
    while lo <= hi {
        let d = hi - lo;
        let meio = (if d < 0 { d + 1 } else { d } >> 1) + lo;
        if lista[meio as usize].pos <= v {
            lo = meio + 1;
        } else {
            hi = meio - 1;
        }
    }
    hi
}

/// Batida mais próxima de `x` a até `w` amostras (`0x142616fc0`; a mesma
/// busca dentro de `0x142617c00`), conferida no assembly. Varre de
/// `max(último <= x, 0)` até `último <= x + w` (−1 vira 0), fica com a
/// primeira de menor distância e para na primeira posterior a `x` dentro da
/// janela.
pub fn mais_proxima(lista: &[BatidaSensor], x: i32, w: i32) -> Option<usize> {
    if lista.is_empty() {
        return None;
    }
    let a = ultimo_ate(lista, x).max(0);
    let b = match ultimo_ate(lista, x.wrapping_add(w)) {
        -1 => 0,
        h => h,
    };
    let (mut melhor, mut menor) = (None, i32::MAX);
    for i in a..=b {
        let p = lista[i as usize].pos;
        let d = (x - p).abs();
        if d <= w {
            if d < menor {
                melhor = Some(i as usize);
                menor = d;
            }
            if x < p {
                break;
            }
        }
    }
    melhor
}

/// Pontuação de uma grade candidata (`0x142617c00`), conferida no assembly:
/// para `t` de `inicio` até `fim` (exclusivo) em passos de `passo`, a
/// batida mais próxima de `(int)t` a até `w` soma `w − |pos − (int)t|`.
/// Devolve a soma e a primeira posição casada (−1 sem casamento).
pub fn pontuar_grade(
    lista: &[BatidaSensor],
    inicio: f64,
    fim: f64,
    passo: f64,
    w: i32,
) -> (f64, f64) {
    let (mut soma, mut primeira) = (0.0f64, -1.0f64);
    let mut t = inicio;
    while t < fim {
        let ti = t as i32;
        if let Some(i) = mais_proxima(lista, ti, w) {
            let p = lista[i].pos;
            if p >= 0 {
                soma += f64::from(w - (p - ti).abs());
                if primeira < 0.0 {
                    primeira = f64::from(p);
                }
            }
        }
        t += passo;
    }
    (soma, primeira)
}

/// Busca de fase e intervalo por segmento (`0x142617530`), conferida no
/// assembly. Para cada segmento, com `ms = taxa·0,001`:
///
/// 1. Deslocamentos do intervalo de `0, +1, −1, +2, −2` ms; passo
///    `ivs = (iv + desl)·ms` e `w0 = (int)(ivs·0,125)`.
/// 2. Janela: o início é `min(max(pos_anterior, s), pos)`, e o fim é
///    `min(pos_seguinte, max(e, pos))`, os dois com piso 0. `s` e `e` saem
///    da batida da lista logo antes da posição, deslocada `+0,125·ivs`, ou
///    da primeira batida deslocada `−0,125·ivs`. O primeiro segmento começa
///    em 0 e o último termina em `total`, sem piso.
/// 3. Subdivisões `k = 4, 2, 1`: fases `t` a cada 96 amostras em
///    `[início, min(início + k·ivs, fim))`, pontuadas por `pontuar_grade`
///    com passo `k·ivs` e `w = max(taxa/20, w0)`. A melhor pontuação (por
///    segmento) é reescalada ×2 a cada meia subdivisão. Uma pontuação maior
///    fixa a fase: a primeira batida casada (ou `t`) recuada em
///    `(int)((x − início)/ivs)` passos; com casamento, o recuo usa
///    `fnmadd`.
/// 4. No deslocamento 0, para cedo se a fase for válida e
///    `k·melhor > ((fim − início)/ivs)·(w/2)`.
/// 5. Com fase `>= 0`, o segmento recebe `pos = (int)fase`,
///    `intervalo = (f32)(iv + desl)`, `campo_c = (int)(4k)` e
///    `campo10 = recuo`.
pub fn ajustar_fase(lista: &[BatidaSensor], segmentos: &mut [Segmento], taxa: i32, total: i32) {
    let ms = f64::from(taxa) * 0.001;
    let n = segmentos.len();
    for j in 0..n {
        let mut fase = -1.0f64;
        let mut recuo = 0.0f64;
        let mut melhor = 0.0f64;
        let (mut iv_melhor, mut k_melhor) = (0.0f64, 0.0f64);
        let mut primeira_passada = true;
        let mut desl = 0.0f64;
        let vizinha = |pos: i32, ivs: f64| -> Option<i32> {
            let k = match ultimo_ate(lista, pos) {
                -1 => 0,
                k => k,
            };
            let b = lista.get(k as usize)?;
            Some(if k < 1 {
                b.pos - (ivs * 0.125) as i32
            } else {
                lista[k as usize - 1].pos - (ivs * -0.125) as i32
            })
        };
        'deslocamentos: loop {
            let iv = f64::from(segmentos[j].intervalo) + desl;
            let ivs = iv * ms;
            let w0 = (ivs * 0.125) as i32;
            let pos = f64::from(segmentos[j].pos);
            let fim = if j + 1 == n {
                f64::from(total)
            } else {
                let Some(e) = vizinha(segmentos[j + 1].pos, ivs) else {
                    return;
                };
                let m = f64::from(e).max(pos).min(f64::from(segmentos[j + 1].pos));
                if m < 0.0 {
                    0.0
                } else {
                    m
                }
            };
            let inicio = if j == 0 {
                0.0
            } else {
                let Some(s) = vizinha(segmentos[j].pos, ivs) else {
                    return;
                };
                let m = f64::from(segmentos[j - 1].pos).max(f64::from(s)).min(pos);
                if m < 0.0 {
                    0.0
                } else {
                    m
                }
            };
            let w = (taxa / 20).max(w0);
            let mut k = 4.0f64;
            loop {
                let passo = k * ivs;
                let parada = (passo + inicio).min(fim);
                let mut t = inicio;
                while t < parada {
                    let (soma, primeira) = pontuar_grade(lista, t, fim, passo, w);
                    if soma > melhor {
                        k_melhor = k * 4.0;
                        let inv = 1.0 / ivs;
                        if primeira < 0.0 {
                            recuo = f64::from(((t - inicio) * inv) as i32);
                            fase = t - recuo * ivs;
                        } else {
                            recuo = f64::from(((primeira - inicio) * inv) as i32);
                            fase = (-ivs).mul_add(recuo, primeira);
                        }
                        iv_melhor = iv;
                        melhor = soma;
                    }
                    t += 96.0;
                }
                if primeira_passada
                    && fase >= 0.0
                    && k * melhor > ((fim - inicio) / ivs) * (f64::from(w) * 0.5)
                {
                    break 'deslocamentos;
                }
                k *= 0.5;
                melhor += melhor;
                if k < 1.0 {
                    break;
                }
            }
            primeira_passada = false;
            while k < 4.0 {
                k += k;
                melhor *= 0.5;
            }
            if desl != 0.0 {
                desl = -desl;
            }
            if 0.0 <= desl {
                desl += 1.0;
            }
            if desl >= 3.0 {
                break;
            }
        }
        if fase >= 0.0 {
            let s = &mut segmentos[j];
            s.pos = fase as i32;
            s.intervalo = iv_melhor as f32;
            s.campo_c = k_melhor as i32 as u32;
            s.campo10 = recuo as i32;
        }
    }
}

/// Batida da grade de saída (registro de 24 bytes de `local_230`, copiado
/// para `motor+0x50`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BatidaGrade {
    pub pos: i32,
    /// `+4`: 0,0 na geração; numerado depois.
    pub campo4: f32,
    /// Intervalo em ms do passo vigente (`+8`).
    pub intervalo: f32,
    /// `+0xc`: 0 na geração; recebe o número da batida no compasso depois.
    pub numero: i32,
    /// Banda casada no tempo forte (`+0x10`), ou −1.
    pub marca: i32,
    /// Âncora da reestimativa (`+0x14`).
    pub ancora: bool,
}

/// Resultado da geração da grade.
#[derive(Debug, Clone, PartialEq)]
pub struct Grade {
    pub batidas: Vec<BatidaGrade>,
    /// Índice do primeiro (`local_1c8`) e do último (`local_160`) tempo forte
    /// marcado com banda válida, ou −1.
    pub primeiro_forte: i32,
    pub segundo_forte: i32,
    /// Posição da primeira âncora do primeiro segmento (`local_1bc`).
    pub primeira_pos: i32,
}

/// Parâmetros de frase do analisador: `(4, 16, 12)` com `+0x28 = 32`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frase {
    pub depois: i32,
    pub adiante: i32,
    pub resto: i32,
}

impl Frase {
    pub const PADRAO: Self = Self {
        depois: 4,
        adiante: 16,
        resto: 12,
    };

    fn batidas(self) -> i32 {
        self.adiante + self.depois + self.resto
    }
}

/// Geração da grade (linhas 1254–1888 de `0x142613100`), conferida no
/// assembly do 7.2.19, depois de `ajustar_fase`. Por segmento, com
/// `ms = taxa·0,001`, `ivs = iv·ms` e `w = max(taxa/20, (int)(ivs·0,125))`:
///
/// 1. Avança `recuo` passos (`p` e `q` juntos) e casa `p` com a batida mais
///    próxima. Com casamento, `p = q = pos` e a compensação é a da banda;
///    sem, o máximo das 6. A âncora é `q = fma(ms, compensação, q)`.
/// 2. Avança `p = fma(ivs, 0,25, p)` por quarto. A cada `campo_c` quartos,
///    casa `p` com a batida mais próxima e marca a banda.
/// 3. A cada `4·32` quartos: o passo vira `fma(ms·pont[b], 4, (p − q)·4)·
///    (0,25/32)`, ou `((p − q) + ms·máx)·4·(0,25/32)` sem banda válida.
///    Uma vez por segmento, preenche `recuo + 1` batidas para trás (sem
///    passar da última da grade), grava a âncora e mais 16 adiante. Sempre
///    acrescenta 4 batidas a partir da última e volta `q` 16 batidas; o
///    contador perde 16 quartos e as marcas deslizam.
/// 4. No fim do segmento, faz o mesmo com `passo = X/quartos` (pulado se os
///    quartos forem múltiplos de 128 ou menos de 4) e completa a grade até o
///    fim com passo constante.
/// 5. O último intervalo da grade recebe `passo/ms`.
///
/// `campo_c` 0 dividiria por zero no original; aqui o casamento por
/// `campo_c` é pulado.
pub fn gerar_grade(
    lista: &[BatidaSensor],
    segmentos: &[Segmento],
    pontuacoes: &[[f64; 6]],
    taxa: i32,
    total: i32,
    frase: Frase,
) -> Grade {
    let ms = f64::from(taxa) * 0.001;
    let frase_q = frase.batidas() * 4;
    let fator = 0.25 / f64::from(frase.batidas());
    let mut g = Grade {
        batidas: Vec::new(),
        primeiro_forte: -1,
        segundo_forte: -1,
        primeira_pos: 0,
    };
    let mut passo = 0.0f64;
    let n = segmentos.len();
    for j in 0..n {
        let seg = segmentos[j];
        let pont = pontuacoes.get(j).copied().unwrap_or(PONTUACAO_INICIAL);
        let maxp = pont
            .iter()
            .copied()
            .fold(f64::MIN, |a, b| if b > a { b } else { a });
        let fim = f64::from(segmentos.get(j + 1).map_or(total, |s| s.pos));
        let ult = g.batidas.last().map_or(0.0, |b| f64::from(b.pos));
        let mut p = f64::from(seg.pos);
        let mut q = p;
        let ivs = f64::from(seg.intervalo) * ms;
        passo = ivs;
        let mut iv_reg = seg.intervalo;
        let sub = seg.campo_c as i32;
        let recuo = seg.campo10;
        for _ in 0..recuo.max(0) {
            p += ivs;
            q += ivs;
            if p >= fim {
                break;
            }
        }
        let mut marcas = vec![-1i32; (frase_q + 1) as usize];
        let w0 = (ivs * 0.125) as i32;
        let w = (taxa / 20).max(w0);
        let banda_de = |i: usize| lista[i].banda;
        let compensa = match mais_proxima(lista, p as i32, w) {
            Some(i) => {
                p = f64::from(lista[i].pos);
                marcas[0] = banda_de(i);
                q = p;
                pont.get(usize::try_from(marcas[0]).unwrap_or(6))
                    .copied()
                    .unwrap_or(maxp)
            }
            None => maxp,
        };
        q = ms.mul_add(compensa, q);
        let ms_max = ms * maxp;
        if j == 0 {
            g.primeira_pos = q as i32;
        }
        let mut pular = g.batidas.len() as i32;
        let nova = |pos: f64, iv: f32, marca: i32, ancora: bool| BatidaGrade {
            pos: pos as i32,
            campo4: 0.0,
            intervalo: iv,
            numero: 0,
            marca,
            ancora,
        };
        let forte = |g: &mut Grade, marca: i32, segundo: bool| {
            if (marca as u32) < 6 {
                let k = g.batidas.len() as i32 - 1;
                if g.primeiro_forte < 0 {
                    g.primeiro_forte = k;
                } else if segundo {
                    g.segundo_forte = k;
                }
            }
        };
        let reestimar = |marcas: &[i32], idx: usize, p: f64, q: f64| -> f64 {
            let b = marcas[idx];
            if (0..6).contains(&b) {
                (ms * pont[b as usize]).mul_add(4.0, (p - q) * 4.0)
            } else {
                ((p - q) + ms_max) * 4.0
            }
        };
        let para_tras = |g: &mut Grade, q: f64, passo: f64, iv: f32, pular: &mut i32| {
            for k in -1..recuo {
                let pos = q - f64::from(recuo - k) * passo;
                if (pos as i32) < ult as i32 {
                    if k >= 0 {
                        *pular -= 1;
                    }
                } else {
                    g.batidas.push(nova(pos, iv, -1, false));
                    if k < 0 {
                        *pular += 1;
                    }
                }
            }
        };
        let adiante =
            |g: &mut Grade, q: &mut f64, passo: f64, iv: f32, marcas: &[i32], base: i32, n: i32| {
                for i in 0..n {
                    let quarto = base + 4 * (i + 1);
                    *q += passo;
                    if *q >= fim {
                        break;
                    }
                    let marca = if sub != 0 && quarto % sub == 0 {
                        marcas.get(quarto as usize).copied().unwrap_or(-1)
                    } else {
                        -1
                    };
                    g.batidas.push(nova(*q, iv, marca, false));
                    if sub != 0 && quarto % sub == 0 {
                        forte(g, marca, true);
                    }
                }
            };
        let mut c = 0i32;
        let mut idx = 0usize;
        if p < fim {
            loop {
                for s in 1..=4 {
                    p = ivs.mul_add(0.25, p);
                    marcas[idx + s] = -1;
                    let cc = c + s as i32;
                    if sub != 0 && cc % sub == 0 {
                        if let Some(i) = mais_proxima(lista, p as i32, w) {
                            p = f64::from(lista[i].pos);
                            marcas[idx + s] = banda_de(i);
                        }
                    }
                }
                idx += 4;
                c += 4;
                if c % frase_q == 0 {
                    passo = reestimar(&marcas, idx, p, q) * fator;
                    iv_reg = (passo / ms) as f32;
                    if (g.batidas.len() as i32) < recuo + 1 + pular {
                        para_tras(&mut g, q, passo, iv_reg, &mut pular);
                        g.batidas.push(nova(q, iv_reg, marcas[0], true));
                        if g.primeiro_forte < 0 && (marcas[0] as u32) < 6 {
                            g.primeiro_forte = g.batidas.len() as i32 - 1;
                        }
                    }
                    if (g.batidas.len() as i32) < recuo + 1 + frase.adiante + pular {
                        adiante(&mut g, &mut q, passo, iv_reg, &marcas, 0, frase.adiante);
                    }
                    if let Some(u) = g.batidas.last_mut() {
                        u.intervalo = iv_reg;
                    }
                    q = g.batidas.last().map_or(q, |b| f64::from(b.pos));
                    adiante(
                        &mut g,
                        &mut q,
                        passo,
                        iv_reg,
                        &marcas,
                        frase.adiante * 4,
                        frase.depois,
                    );
                    let len = g.batidas.len() as i32;
                    q = g
                        .batidas
                        .get((len - frase.adiante - 1).max(0) as usize)
                        .map_or(q, |b| f64::from(b.pos));
                    let desliza = (frase.depois * 4) as usize;
                    c -= frase.depois * 4;
                    idx -= desliza;
                    let base = (frase.adiante * 4) as usize;
                    if base <= idx {
                        for t in base..=idx {
                            marcas[t] = marcas[t + desliza];
                        }
                    }
                }
                if p >= fim {
                    break;
                }
            }
            if c % frase_q != 0 && c / 4 != 0 {
                let mut iv_novo = iv_reg;
                if (g.batidas.len() as i32) < recuo + 1 + pular {
                    passo = reestimar(&marcas, idx, p, q) / f64::from(c);
                    iv_novo = (passo / ms) as f32;
                    para_tras(&mut g, q, passo, iv_novo, &mut pular);
                    g.batidas.push(nova(q, iv_novo, marcas[0], true));
                    if g.primeiro_forte < 0 && (marcas[0] as u32) < 6 {
                        g.primeiro_forte = g.batidas.len() as i32 - 1;
                    }
                }
                if (g.batidas.len() as i32) < recuo + 1 + frase.adiante + pular {
                    passo = reestimar(&marcas, idx, p, q) / f64::from(c);
                    iv_novo = (passo / ms) as f32;
                    adiante(&mut g, &mut q, passo, iv_novo, &marcas, 0, frase.adiante);
                }
                iv_reg = iv_novo;
                q = g.batidas.last().map_or(q, |b| f64::from(b.pos));
                adiante(
                    &mut g,
                    &mut q,
                    passo,
                    iv_reg,
                    &marcas,
                    frase.adiante * 4,
                    c / 4 - frase.adiante,
                );
            }
            // O preenchimento final fica dentro do bloco `p < fim` (linha 1862).
            if q < fim {
                loop {
                    q += passo;
                    if q >= fim {
                        break;
                    }
                    g.batidas.push(nova(q, iv_reg, -1, false));
                }
            }
        }
    }
    if let Some(u) = g.batidas.last_mut() {
        u.intervalo = (passo / ms) as f32;
    }
    g
}

/// Regrade uniforme do modo 0 (`0x1426182f0`), conferida no assembly.
/// Entre a batida `a = max(primeiro, 0)` e `b` (`segundo`, ou a última),
/// `passo = (pb − pa)/(b − a)` e `k = (int)(pa·(1/passo))`. A grade é refeita
/// do zero:
///
/// - `k + 1` batidas a partir de `fma(−passo, k, pa)`, com a de índice `k`
///   marcada como âncora e com a banda de `a`;
/// - `b − a` batidas, a última com a banda de `b`;
/// - `(int)((total − pb)·(1/passo))` batidas depois.
///
/// O intervalo de todas é `(f32)((1000/taxa)·passo)`. Grade vazia ou `b = a`
/// (divisão por zero no original): a grade fica como está.
pub fn regrade_uniforme(
    grade: &mut Vec<BatidaGrade>,
    total: i32,
    primeiro: i32,
    segundo: i32,
    taxa: i32,
) {
    if grade.is_empty() {
        return;
    }
    let a = primeiro.max(0) as usize;
    let b = if segundo < 0 {
        grade.len() - 1
    } else {
        segundo as usize
    };
    let (Some(ba), Some(bb)) = (grade.get(a).copied(), grade.get(b).copied()) else {
        return;
    };
    let n = b as i32 - a as i32;
    if n == 0 {
        return;
    }
    let pa = f64::from(ba.pos);
    let pb = f64::from(bb.pos);
    let passo = (pb - pa) / f64::from(n);
    let inv = 1.0 / passo;
    let k = (pa * inv) as i32;
    let mut x = (-passo).mul_add(f64::from(k), pa);
    let depois = ((f64::from(total) - pb) * inv) as i32;
    let iv = ((1000.0 / f64::from(taxa)) * passo) as f32;
    grade.clear();
    let mut emitir = |grade: &mut Vec<BatidaGrade>, n: i32| {
        for _ in 0..n.max(0) {
            grade.push(BatidaGrade {
                pos: x as i32,
                campo4: 0.0,
                intervalo: iv,
                numero: 0,
                marca: -1,
                ancora: false,
            });
            x += passo;
        }
    };
    emitir(grade, k + 1);
    if let Some(e) = usize::try_from(k).ok().and_then(|k| grade.get_mut(k)) {
        e.ancora = true;
        e.marca = ba.marca;
    }
    emitir(grade, n);
    if let Some(e) = usize::try_from(k + n).ok().and_then(|i| grade.get_mut(i)) {
        e.marca = bb.marca;
    }
    emitir(grade, depois);
}

/// Numeração no compasso e limpeza (linhas 1944–2002): com
/// `por = max(4, depois)`, a batida em `primeira_pos` (última com
/// `pos <= primeira_pos`, ou a 0) é a número 1. Depois saem as batidas com
/// `pos < 0` ou `pos > total`.
pub fn numerar(grade: &mut Vec<BatidaGrade>, primeira_pos: i32, total: i32, frase: Frase) {
    let por = frase.depois.max(4);
    let mut lo = 0i32;
    let mut hi = grade.len() as i32 - 1;
    while lo <= hi {
        let d = hi - lo;
        let meio = (if d < 0 { d + 1 } else { d } >> 1) + lo;
        if grade[meio as usize].pos <= primeira_pos {
            lo = meio + 1;
        } else {
            hi = meio - 1;
        }
    }
    let i0 = if hi < 0 { 0 } else { hi };
    let base = por - i0 % por;
    for (i, b) in grade.iter_mut().enumerate() {
        b.numero = (base + i as i32) % por + 1;
    }
    grade.retain(|b| b.pos >= 0 && b.pos <= total);
}

/// Contagem de BPM do motor 1 no modo 0 (`0x142613100` com flag 0), ligando
/// as etapas portadas: estimativa, refino de segmentos, deduplicação, busca
/// de fase, geração da grade, regrade uniforme e numeração. Devolve o BPM
/// (`60000/intervalo` da primeira batida da regrade, em `f32`) e a grade.
/// Sem estimativa, devolve `(0, vazio)`, como o original, que sai com o
/// BPM zerado sem tocar na saída.
pub fn contar_bpm_modo0(
    lista: &mut Vec<BatidaSensor>,
    taxa: i32,
    total: i32,
    frase: Frase,
) -> (f32, Vec<BatidaGrade>) {
    let Some((_, semente)) = estimar_modo0(lista, total) else {
        return (0.0, Vec::new());
    };
    let mut segs = vec![Segmento::from(semente)];
    refinar_segmentos(lista, &mut segs, total);
    let pontuacoes = vec![PONTUACAO_INICIAL; segs.len()];
    deduplicar(lista, &segs, taxa, total);
    ajustar_fase(lista, &mut segs, taxa, total);
    let mut g = gerar_grade(lista, &segs, &pontuacoes, taxa, total, frase);
    let primeira_pos = if g.primeiro_forte < 0 {
        0
    } else {
        g.batidas
            .get(g.primeiro_forte as usize)
            .map_or(0, |b| b.pos)
    };
    regrade_uniforme(
        &mut g.batidas,
        total,
        g.primeiro_forte,
        g.segundo_forte,
        taxa,
    );
    let bpm = g.batidas.first().map_or(0.0, |b| 60000.0f32 / b.intervalo);
    numerar(&mut g.batidas, primeira_pos, total, frase);
    (bpm, g.batidas)
}

/// Primeira batida de `lista[inicio..]` cuja caixa na `tab` é `caixa`
/// (`0x142616e30`, rekordbox 7.2.19).
fn primeira_na_caixa(
    lista: &[BatidaSensor],
    inicio: usize,
    caixa: i32,
    tab: &TabelaBpm,
) -> Option<usize> {
    (inicio..lista.len()).find(|&k| tab.caixa(lista[k].intervalo) == caixa)
}

/// Poda e segmentação do modo 1 (`0x142613100` com `param_8 != 0`, linhas
/// 362–816 de `decomp/142613100.c` e `disasm/142613100.asm:458-1200`),
/// conferidas no assembly:
///
/// 1. Monta `ger_iv[geração]` (`etiqueta >> 8`) com o intervalo da primeira
///    batida de cada geração, ou, se ainda zerado, da próxima geração não nula
///    compatível em `na_janela`.
/// 2. Remove gerações iniciais zeradas e corridas curtas (`< 2`, ou `< 8`
///    quando a geração seguinte volta à geração anterior) de gerações
///    incompatíveis.
/// 3. Remove corridas curtas (`< 8`, ou `< 32` quando a batida seguinte volta
///    ao intervalo anterior) de batidas consecutivas incompatíveis (`0x142618a70`).
/// 4. Segmenta `lista` em janelas de `8 * taxa` amostras usando `TabelaBpm`
///    de 400 caixas (`0x142612d40`) e `primeira_na_caixa` (`0x142616e30`) nas
///    oitavas `0,5×`, `1×` e `2×`.
pub fn estimar_modo1(lista: &mut Vec<BatidaSensor>, taxa: i32) -> Vec<Segmento> {
    if lista.is_empty() {
        return Vec::new();
    }
    let geracao = |b: &BatidaSensor| (b.etiqueta as i32) >> 8;
    let mut ger_iv: Vec<f32> = Vec::new();
    for b in lista.iter() {
        let g = geracao(b);
        if g < 0 {
            continue;
        }
        let gu = g as usize;
        if gu >= ger_iv.len() {
            ger_iv.resize(gu + 1, 0.0);
            ger_iv[gu] = b.intervalo;
        } else if ger_iv[gu] == 0.0 {
            if let Some(&iv_prox) = ger_iv[gu + 1..].iter().find(|&&v| v != 0.0) {
                if na_janela(b.intervalo, iv_prox, tolerancia(iv_prox)) {
                    ger_iv[gu] = b.intervalo;
                }
            }
        }
    }
    if lista.is_empty() || ger_iv.is_empty() {
        return Vec::new();
    }

    let mut inicio_run = 0usize;
    let mut g_idx = 0usize;
    while g_idx < ger_iv.len() && ger_iv[g_idx] == 0.0 {
        let g_alvo = g_idx as i32;
        lista.retain(|b| geracao(b) != g_alvo);
        inicio_run += 1;
        g_idx += 1;
    }

    if g_idx < ger_iv.len() {
        let mut iv_atual = ger_iv[g_idx];
        let mut iv_ant = 0.0f32;
        let mut run_ant = 0i32;
        g_idx += 1;
        let mut run_atual = 0i32;
        while g_idx < ger_iv.len() {
            let iv_cand = ger_iv[g_idx];
            if iv_cand == 0.0 {
                let g_alvo = g_idx as i32;
                lista.retain(|b| geracao(b) != g_alvo);
                g_idx += 1;
                continue;
            }
            let iv_prev = iv_atual;
            iv_atual = iv_cand;
            if na_janela(iv_cand, iv_prev, tolerancia(iv_prev)) {
                g_idx += 1;
                run_atual += 1;
            } else {
                let descartar = run_atual < 2
                    || (run_atual < 8 && na_janela(iv_cand, iv_ant, tolerancia(iv_ant)));
                if descartar {
                    for g_rem in inicio_run..g_idx {
                        let g_alvo = g_rem as i32;
                        lista.retain(|b| geracao(b) != g_alvo);
                    }
                    inicio_run = g_idx;
                    run_atual = run_ant;
                    if run_ant != 0 {
                        iv_atual = iv_ant;
                    } else {
                        iv_atual = ger_iv[g_idx];
                    }
                } else {
                    run_ant = run_atual;
                    iv_ant = iv_prev;
                    run_atual = 0;
                    inicio_run = g_idx;
                    g_idx += 1;
                }
            }
        }
        if run_ant != 0 && run_atual < 1 && inicio_run < g_idx {
            for g_rem in inicio_run..g_idx {
                let g_alvo = g_rem as i32;
                lista.retain(|b| geracao(b) != g_alvo);
            }
        }
    }

    if lista.is_empty() {
        return Vec::new();
    }

    let mut run_ant = 0i32;
    let mut iv_ant = 0.0f32;
    let mut inicio_b = 0usize;
    let mut iv_atual = lista[0].intervalo;
    let mut idx = 1usize;
    let mut run_atual = 0i32;
    while idx < lista.len() {
        let iv_cand = lista[idx].intervalo;
        let iv_prev = iv_atual;
        iv_atual = iv_cand;
        if na_janela(iv_cand, iv_prev, tolerancia(iv_prev)) {
            idx += 1;
            run_atual += 1;
        } else {
            let descartar =
                run_atual < 8 || (run_atual < 32 && na_janela(iv_cand, iv_ant, tolerancia(iv_ant)));
            if descartar {
                lista.drain(inicio_b..idx);
                idx = inicio_b;
                run_atual = run_ant;
                if run_ant != 0 {
                    iv_atual = iv_ant;
                } else if inicio_b < lista.len() {
                    iv_atual = lista[inicio_b].intervalo;
                }
            } else {
                run_ant = run_atual;
                iv_ant = iv_prev;
                run_atual = 0;
                inicio_b = idx;
                idx += 1;
            }
        }
    }
    if run_ant != 0 && run_atual < 8 {
        lista.truncate(inicio_b);
    }

    if lista.is_empty() {
        return Vec::new();
    }

    let mut tab = TabelaBpm::nova(400, 0);
    let janela = (taxa << 3) as f32 as i32;
    let mut segmentos = Vec::new();
    let mut iv_seg_atual = 0.0f32;
    let mut i = 0usize;
    while i < lista.len() {
        tab.contar(lista, i, janela);
        let caixa_max = tab.primeiro_maximo(0, 400);
        let mut prox_i = i;
        if caixa_max > 0 {
            let iv_cand = 60_000.0f32 / (caixa_max as f32);
            let iv_ant_seg = iv_seg_atual;
            iv_seg_atual = iv_cand;
            if !na_janela(iv_cand, iv_ant_seg, tolerancia(iv_ant_seg)) {
                let tol_cand = tolerancia(iv_cand);
                let idx_base = primeira_na_caixa(lista, i, caixa_max, &tab).unwrap_or(i);
                let mut idx_min = idx_base;
                let mut mult = 0.5f32;
                while mult <= 2.0 {
                    let iv_m = mult * iv_cand;
                    let cx_lo = (60_000.0f32 / (iv_m + tol_cand + 0.5)) as i32;
                    let cx_hi = (60_000.0f32 / (iv_m - tol_cand + 0.5)) as i32;
                    for cx in cx_lo..cx_hi {
                        if let Some(k) = primeira_na_caixa(lista, i, cx, &tab) {
                            if k < idx_min {
                                idx_min = k;
                            }
                        }
                    }
                    mult += mult;
                }
                if let (Some(&b_min), Some(&b_base)) = (lista.get(idx_min), lista.get(idx_base)) {
                    let mut seg = Segmento::from(b_min);
                    seg.intervalo = b_base.intervalo;
                    segmentos.push(seg);
                    prox_i = idx_min;
                }
            }
        }
        i = prox_i + 1;
    }
    segmentos
}

/// Ajuste local de intervalos em janelas de 4 batidas do modo 1
/// (`0x142617e60`, conferido em `decomp/142617e60.c` e `disasm/142617e60.asm`).
pub fn ajustar_grade_modo1(grade: &mut Vec<BatidaGrade>) {
    let mut i = 0usize;
    while i + 3 < grade.len() {
        let p0 = grade[i].pos;
        let p1 = grade[i + 1].pos;
        let p2 = grade[i + 2].pos;
        let p3 = grade[i + 3].pos;
        let d0 = f64::from(p1 - p0);
        let d1 = f64::from(p2 - p1);
        let d2 = f64::from(p3 - p2);
        let d_max = d2.max(d0);
        let d_min = d2.min(d0);
        let inv_max = 1.0 / d_max;
        if d1 * inv_max > 1.1 {
            if d_max == d0 && (d1 - d_max) / (d_max - d_min) < 4.0 {
                grade[i + 2].pos = p1 + (d_max as i32);
            } else if d_min == d0 && d_max.mul_add(3.0, d1) > d_min * 5.0 {
                let passo = d_min as i32;
                let neg2 = (d_min * -2.0) as i32;
                grade[i + 2].pos = p1 + passo;
                if p1 + passo - neg2 < p3 {
                    let mut nova = grade[i + 2];
                    nova.pos = grade[i + 1].pos - neg2;
                    grade.insert(i + 2, nova);
                }
            } else {
                let span = f64::from(p3 - p0);
                if (d1 + d0 + d2) * inv_max < 3.300_000_000_000_000_3 {
                    let terco = span * (1.0 / 3.0);
                    grade[i + 1].pos = p0 + (terco as i32);
                    grade[i + 2].pos = grade[i].pos - ((terco * -2.0) as i32);
                } else {
                    let quarto = span * 0.25;
                    grade[i + 1].pos = p0 + (quarto as i32);
                    grade[i + 2].pos = grade[i].pos - ((quarto * -2.0) as i32);
                    let mut nova = grade[i + 2];
                    nova.pos = grade[i].pos - ((quarto * -3.0) as i32);
                    grade.insert(i + 2, nova);
                }
            }
        } else {
            let inv_min = 1.0 / d_min;
            if inv_min * d1 < 0.9 {
                if d_min == d0 && (d_min - d1) / (d_max - d_min) < 4.0 {
                    grade[i + 2].pos = p1 + (d_min as i32);
                } else if d_max == d0 && d_min.mul_add(4.0, d1) < d_max * 4.0 {
                    let novo_p2 = p1 + (d_max as i32);
                    if novo_p2 < p3 {
                        grade[i + 2].pos = novo_p2;
                    } else {
                        grade.remove(i + 2);
                        i = (i + 1).saturating_sub(2);
                        continue;
                    }
                } else if (d1 + d0 + d2) * inv_min > 2.7 {
                    let terco = f64::from(p3 - p0) * (1.0 / 3.0);
                    grade[i + 1].pos = p0 + (terco as i32);
                    grade[i + 2].pos = grade[i].pos - ((terco * -2.0) as i32);
                } else {
                    grade[i + 1].pos = (p0 + p3) / 2;
                    grade.remove(i + 2);
                    i = (i + 1).saturating_sub(2);
                    continue;
                }
            }
        }
        i += 1;
    }
}

/// Contagem de BPM e geração da grade no **modo 1** (`0x142613100` com
/// `param_8 != 0`, rekordbox 7.2.19): executa a poda/segmentação do modo 1
/// (`estimar_modo1`), refina os segmentos, deduplica, ajusta a fase por
/// segmento, gera a grade multi-segmento, aplica `ajustar_grade_modo1`
/// (`0x142617e60`), numera a partir de `primeira_pos = 0` e devolve o BPM do
/// segmento mais longo (`bpm_do_mais_longo`).
pub fn contar_bpm_modo1(
    lista: &mut Vec<BatidaSensor>,
    taxa: i32,
    total: i32,
    frase: Frase,
) -> (f32, Vec<BatidaGrade>) {
    let mut segs = estimar_modo1(lista, taxa);
    if segs.is_empty() {
        return (0.0, Vec::new());
    }
    refinar_segmentos(lista, &mut segs, total);
    let pontuacoes = vec![PONTUACAO_INICIAL; segs.len()];
    deduplicar(lista, &segs, taxa, total);
    let bpm = bpm_do_mais_longo(&segs, total);
    ajustar_fase(lista, &mut segs, taxa, total);
    let mut g = gerar_grade(lista, &segs, &pontuacoes, taxa, total, frase);
    ajustar_grade_modo1(&mut g.batidas);
    numerar(&mut g.batidas, 0, total, frase);
    (bpm, g.batidas)
}

/// `BeatAnalyzer_1_0::HighPathFilter1D` (`0x14260dd20` / `0x14260dd80` /
/// `0x14260de00`, rekordbox 7.2.19), conferido em `disasm/14260dd20.asm` e
/// `disasm/14260dd80.asm`: filtro passa-alta IIR de 1ª ordem usado em `+0x50`
/// de `UnitEnvelopeMaker` (`0x142619aa0`).
#[derive(Debug, Clone, PartialEq)]
pub struct PassaAlta1D {
    pub b0: f64,
    pub b1: f64,
    pub a0: f64,
    pub a1: f64,
    x1: f64,
    y1: f64,
}

impl PassaAlta1D {
    /// Inicializa o passa-alta 1D para frequência de corte `f` Hz e taxa `fs` Hz
    /// (`0x14260dd80`).
    pub fn novo(f: f64, fs: f64) -> Self {
        let dt = 1.0 / fs;
        let w = f * DOIS_PI;
        let inv = 1.0 / dt.mul_add(w, 2.0);
        let b0 = inv + inv;
        let b1 = -b0;
        let a1 = dt.mul_add(w, -2.0) * inv;
        Self {
            b0,
            b1,
            a0: 1.0,
            a1,
            x1: 0.0,
            y1: 0.0,
        }
    }

    /// Filtra uma amostra mantendo o estado `(x1, y1)` (`0x14260de00`).
    pub fn filtrar_amostra(&mut self, x: f64) -> f64 {
        let y = (-self.a1).mul_add(self.y1, self.b1.mul_add(self.x1, self.b0 * x));
        self.x1 = x;
        self.y1 = y;
        y
    }
}

/// Refinador de envelope e fase de compasso (`0x142619aa0` / `0x142619ea0` /
/// `0x14261a080`, rekordbox 7.2.19).
///
/// Decima o áudio mono `(L + R) * 0,5` em blocos de 128 frames, aplica dois
/// passa-baixas 2D a 150 Hz, retifica a primeira amostra do bloco, suaviza com
/// passa-baixas 2D a 5 Hz e passa-alta 1D a 0,5 Hz (`HighPathFilter1D` em
/// `+0x50`), e usa os picos desse envelope para corrigir inversões de
/// meio-tempo (180°) e alinhar o tempo forte (`numero` 1..4) da grade.
#[derive(Debug, Clone)]
pub struct RefinadorDeEnvelope {
    /// Taxa de amostragem em Hz (`+0x14`).
    pub taxa: i32,
    /// Total de frames da faixa (`+8`).
    pub total: i32,
    /// Tamanho do bloco de decimação (`+0x18`, sempre 128).
    pub bloco: usize,
    contados: i32,
    pos_bloco: usize,
    buf_mono: Vec<f64>,
    buf_lp1: Vec<f64>,
    buf_lp2: Vec<f64>,
    lp1: SecaoRbj,
    lp2: SecaoRbj,
    lp_env: SecaoRbj,
    hp_env: PassaAlta1D,
    /// Envelope suavizado por bloco de 128 frames (`+0x58`).
    pub envelope: Vec<i16>,
}

impl RefinadorDeEnvelope {
    /// Inicializa o refinador (`0x142619aa0`).
    pub fn new(taxa: i32, total_frames: i32) -> Self {
        let bloco = 128usize;
        let taxa_env = f64::from(taxa) / bloco as f64;
        let n_quadros = ((total_frames.max(0) + bloco as i32 - 1) / bloco as i32) as usize;
        Self {
            taxa,
            total: total_frames,
            bloco,
            contados: 0,
            pos_bloco: 0,
            buf_mono: vec![0.0; bloco],
            buf_lp1: vec![0.0; bloco],
            buf_lp2: vec![0.0; bloco],
            lp1: SecaoRbj::passa_baixa_2d(150.0, f64::from(taxa)),
            lp2: SecaoRbj::passa_baixa_2d(150.0, f64::from(taxa)),
            lp_env: SecaoRbj::passa_baixa_2d(5.0, taxa_env),
            hp_env: PassaAlta1D::novo(0.5, taxa_env),
            envelope: vec![0; n_quadros],
        }
    }

    /// Alimenta frames estéreo intercalados `i16` (`0x142619ea0`).
    pub fn alimentar(&mut self, stereo_interleaved: &[i16]) {
        let n_frames = stereo_interleaved.len() / 2;
        if n_frames == 0 || self.contados >= self.total {
            return;
        }
        let mut idx = 0usize;
        while self.contados < self.total && idx < n_frames {
            let falta_bloco = self.bloco - self.pos_bloco;
            let falta_total = (self.total - self.contados) as usize;
            let passo = falta_bloco.min(falta_total).min(n_frames - idx);
            for k in 0..passo {
                let l = f64::from(stereo_interleaved[(idx + k) * 2]);
                let r = f64::from(stereo_interleaved[(idx + k) * 2 + 1]);
                self.buf_mono[self.pos_bloco + k] = (l + r) * 0.5;
            }
            self.pos_bloco += passo;
            self.contados += passo as i32;
            idx += passo;
            self.fechar_bloco_se_pronto();
        }
    }

    /// Alimenta pares estéreo `[L, R]` entregues pelo `Alimentador` (`0x142619ea0`).
    pub fn alimentar_pares(&mut self, estereo: &[[i16; 2]]) {
        let n_frames = estereo.len();
        if n_frames == 0 || self.contados >= self.total {
            return;
        }
        let mut idx = 0usize;
        while self.contados < self.total && idx < n_frames {
            let falta_bloco = self.bloco - self.pos_bloco;
            let falta_total = (self.total - self.contados) as usize;
            let passo = falta_bloco.min(falta_total).min(n_frames - idx);
            for k in 0..passo {
                let [l, r] = estereo[idx + k];
                self.buf_mono[self.pos_bloco + k] = (f64::from(l) + f64::from(r)) * 0.5;
            }
            self.pos_bloco += passo;
            self.contados += passo as i32;
            idx += passo;
            self.fechar_bloco_se_pronto();
        }
    }

    fn fechar_bloco_se_pronto(&mut self) {
        if self.pos_bloco == self.bloco || self.contados >= self.total {
            let n = self.pos_bloco;
            self.lp1
                .processar(&self.buf_mono[..n], &mut self.buf_lp1[..n]);
            self.lp2
                .processar(&self.buf_lp1[..n], &mut self.buf_lp2[..n]);
            let abs0 = self.buf_lp2[0].abs();
            let mut tmp_env = [0.0f64; 1];
            self.lp_env.processar(&[abs0], &mut tmp_env);
            let y_hp = self.hp_env.filtrar_amostra(tmp_env[0]);
            let quadro = ((self.contados - n as i32) / self.bloco as i32) as usize;
            if quadro < self.envelope.len() {
                self.envelope[quadro] = (y_hp as i32) as i16;
            }
            self.pos_bloco = 0;
        }
    }

    /// Extrai picos locais do envelope acima da média dos picos (`0x14261a080`).
    fn extrair_picos(&self) -> (Vec<i16>, Vec<usize>) {
        let n = self.envelope.len();
        let mut picos_val = vec![0i16; n];
        let mut soma: i64 = 0;
        let mut qtd: i64 = 0;
        if n > 1 {
            let mut pico_cand: i16 = 0;
            let mut pico_idx: usize = 0;
            let mut subindo = false;
            for i in 1..n {
                let s = self.envelope[i];
                if subindo || self.envelope[i - 1] <= s {
                    if pico_cand < s {
                        subindo = true;
                        pico_idx = i;
                        pico_cand = s;
                    } else if !subindo {
                        continue;
                    }
                    if s == 0 || (f64::from(pico_cand) / f64::from(s)) > 1.5 {
                        picos_val[pico_idx] = pico_cand;
                        if pico_cand > 0 {
                            soma += i64::from(pico_cand);
                            qtd += 1;
                        }
                        pico_cand = s;
                        subindo = false;
                    }
                } else {
                    pico_cand = s;
                }
            }
        }
        let media = if qtd > 0 { (soma / qtd) as i16 } else { 0 };
        let mut indices = Vec::new();
        for (i, val) in picos_val.iter_mut().enumerate().skip(1) {
            if *val < media {
                *val = 0;
            } else {
                indices.push(i);
            }
        }
        (picos_val, indices)
    }

    /// Refina a fase (meio-tempo) e a numeração de compasso da grade (`0x14261a080`).
    pub fn processar(&self, bpm: f32, grade: &mut Vec<BatidaGrade>, recheck: bool) {
        if bpm <= 0.0 || grade.len() < 2 {
            return;
        }
        let bloco_f = self.bloco as f64;
        let taxa_env = f64::from(self.taxa) / bloco_f;
        let atraso = (taxa_env * 0.11 + 0.5) as i32;

        let mut pos_q = Vec::with_capacity(grade.len());
        let mut int_q = Vec::with_capacity(grade.len());
        let prepended =
            f64::from(grade[0].pos) / bloco_f > f64::from(grade[0].intervalo) * taxa_env * 0.0009;
        if prepended {
            pos_q.push(1);
            int_q.push(taxa_env * f64::from(grade[0].intervalo) * 0.001);
            for b in grade.iter().take(grade.len() - 1) {
                pos_q.push((f64::from(b.pos) / bloco_f + 0.5) as i32);
                int_q.push(taxa_env * f64::from(b.intervalo) * 0.001);
            }
        } else {
            for b in grade.iter() {
                pos_q.push((f64::from(b.pos) / bloco_f + 0.5) as i32);
                int_q.push(taxa_env * f64::from(b.intervalo) * 0.001);
            }
        }
        if pos_q[0] < 1 {
            pos_q[0] = 1;
        }
        let primeira_pos_q = f64::from(pos_q[0]);

        let (picos_val, picos_idx) = self.extrair_picos();
        let mut picos_25x = Vec::new();
        let mut picos_15x = Vec::new();
        let mut cnt_25x: i32 = 0;
        let mut cnt_15x: i32 = 0;

        if picos_idx.len() > 2 {
            for w in picos_idx.windows(3) {
                let (p0, p1, p2) = (w[0] as i32, w[1] as i32, w[2] as i32);
                let d1 = p1 - p0;
                let d2 = p2 - p1;
                let beat_q = Self::intervalo_local(&pos_q, &int_q, p0);
                if d1 > (beat_q * 2.25) as i32
                    && d1 < 1 - (beat_q * -2.75) as i32
                    && ((d2 > (beat_q * 1.35) as i32 && d2 < 1 - (beat_q * -1.65) as i32)
                        || (d2 > (beat_q * 1.8) as i32 && d2 < 1 - (beat_q * -2.2) as i32))
                {
                    cnt_25x += 1;
                    picos_25x.push(p0);
                }
                if d1 > (beat_q * 1.35) as i32 && d1 < 1 - (beat_q * -1.65) as i32 {
                    cnt_15x += 1;
                    picos_15x.push(p0);
                } else if d1 > (beat_q * 0.9) as i32
                    && d1 < 1 - (beat_q * -1.1) as i32
                    && d2 > (beat_q * 0.9) as i32
                    && d2 < 1 - (beat_q * -1.1) as i32
                {
                    cnt_15x -= 1;
                }
            }
        }

        let mut deslocou = false;
        if !recheck {
            let beat_global_q = taxa_env * (60.0 / f64::from(bpm));
            let selecionados: Vec<i32> = if cnt_25x >= 11 {
                picos_25x.clone()
            } else if cnt_15x >= 81 {
                picos_15x.clone()
            } else {
                picos_idx.iter().map(|&i| i as i32).collect()
            };
            let mut fora = 0usize;
            for &pk in &selecionados {
                if let Some(bi) = Self::batida_mais_proxima(&pos_q, pk - atraso) {
                    if (pos_q[bi] - (pk - atraso)).abs() > (beat_global_q * 0.25) as i32 {
                        fora += 1;
                    }
                }
            }
            if fora > selecionados.len() / 2 {
                let shift = if primeira_pos_q >= beat_global_q * 0.45 + 0.5 {
                    -((beat_global_q * 0.5 + 0.5) as i32)
                } else {
                    (beat_global_q * 0.5 + 0.5) as i32
                };
                for p in &mut pos_q {
                    *p += shift;
                }
                if pos_q[0] < 1 {
                    pos_q[0] = 1;
                }
                deslocou = true;
            }
        }

        let mut bar_scores = [0.0f64; 4];
        if !picos_idx.is_empty() {
            let pk0 = picos_idx[0] as i32;
            let atraso0 = atraso.min(pk0);
            let beat_q = Self::intervalo_local(&pos_q, &int_q, pk0);
            let min_1beat = (beat_q * 0.9 + 0.5) as i32;
            let max_1beat = (beat_q * 1.1 + 0.5) as i32;
            let min_2beat = (beat_q * 1.8 + 0.5) as i32;
            let max_2beat = (beat_q * 2.2 + 0.5) as i32;
            let d1 = if picos_idx.len() >= 2 {
                picos_idx[1] as i32 - pk0
            } else {
                0
            };
            if d1 >= min_1beat && d1 <= max_1beat {
                if let Some(bi) = Self::batida_mais_proxima(&pos_q, pk0 - atraso0) {
                    bar_scores[bi & 3] += 2.0 * f64::from(picos_val[pk0 as usize]);
                }
            }
            if d1 >= min_2beat && d1 <= max_2beat {
                if let Some(bi) = Self::batida_mais_proxima(&pos_q, pk0 - atraso0) {
                    bar_scores[bi & 3] += 2.0 * f64::from(picos_val[pk0 as usize]);
                }
            }
            let d02 = if picos_idx.len() >= 3 {
                picos_idx[2] as i32 - pk0
            } else {
                0
            };
            if d02 >= min_1beat && d02 <= max_1beat {
                if let Some(bi) = Self::batida_mais_proxima(&pos_q, pk0 - atraso0) {
                    bar_scores[bi & 3] += 2.0 * f64::from(picos_val[pk0 as usize]);
                }
            }
        }

        if cnt_25x >= 11 {
            for &pk in &picos_25x {
                if let Some(bi) = Self::batida_mais_proxima(&pos_q, pk - atraso) {
                    bar_scores[bi & 3] += 983.04;
                }
            }
        } else if cnt_15x >= 81 {
            for &pk in &picos_15x {
                if let Some(bi) = Self::batida_mais_proxima(&pos_q, pk - atraso) {
                    bar_scores[bi & 3] += 983.04;
                }
            }
        } else if picos_idx.len() > 2 {
            let mut max_pico = 0.0f64;
            for &idx in &picos_idx {
                let v = f64::from(picos_val[idx]);
                if v > max_pico {
                    max_pico = v;
                }
            }
            for w in picos_idx.windows(3) {
                let (p0, p1, p2) = (w[0] as i32, w[1] as i32, w[2] as i32);
                let beat_q = Self::intervalo_local(&pos_q, &int_q, p1);
                if f64::from(p1 - p0) > beat_q * 7.0 {
                    let d2 = p2 - p1;
                    let val_p1 = f64::from(picos_val[p1 as usize]);
                    let em_2beat =
                        d2 >= (beat_q * 1.8 + 0.5) as i32 && d2 <= (beat_q * 2.2 + 0.5) as i32;
                    if em_2beat || val_p1 > max_pico * 0.6 {
                        if let Some(bi) = Self::batida_mais_proxima(&pos_q, p1 - atraso) {
                            bar_scores[bi & 3] += val_p1;
                        }
                    }
                }
            }
        }

        let mut melhor_mod = 0usize;
        for (i, &s) in bar_scores.iter().enumerate().skip(1) {
            if s > bar_scores[melhor_mod] {
                melhor_mod = i;
            }
        }
        if bar_scores[melhor_mod] == 0.0 && !picos_idx.is_empty() {
            if let Some(bi) = Self::batida_mais_proxima(&pos_q, picos_idx[0] as i32) {
                melhor_mod = bi & 3;
            }
        }
        let mut fase = (4 - (melhor_mod & 3)) & 3;
        let shift_samples = if deslocou {
            if !prepended {
                pos_q.get(1).copied().unwrap_or(pos_q[0]) * self.bloco as i32 - grade[0].pos
            } else {
                pos_q.get(2).copied().unwrap_or(pos_q[0]) * self.bloco as i32 - grade[0].pos
            }
        } else {
            0
        };
        if prepended {
            fase = (fase + 1) & 3;
        }
        for b in grade.iter_mut() {
            b.pos += shift_samples;
            b.numero = ((fase & 3) + 1) as i32;
            fase += 1;
        }

        if let Some(ultima) = grade.last().cloned() {
            if ultima.pos < self.total {
                let passo =
                    (f64::from(self.taxa) * f64::from(ultima.intervalo) * 0.001 + 0.5) as i32;
                if ultima.pos + passo < self.total {
                    let mut prox = ultima;
                    prox.pos += passo;
                    prox.numero = (prox.numero & 3) + 1;
                    grade.push(prox);
                }
            } else {
                grade.pop();
            }
        }

        if !grade.is_empty() {
            if grade[0].pos < 0 {
                grade.remove(0);
            } else {
                let passo =
                    (f64::from(self.taxa) * f64::from(grade[0].intervalo) * 0.001 + 0.5) as i32;
                if grade[0].pos - passo >= 0 {
                    let mut ant = grade[0];
                    ant.pos -= passo;
                    ant.numero = ((ant.numero + 2) & 3) + 1;
                    grade.insert(0, ant);
                }
            }
        }
    }

    fn intervalo_local(pos_q: &[i32], int_q: &[f64], alvo: i32) -> f64 {
        if pos_q.is_empty() || int_q.is_empty() {
            return 0.0;
        }
        let idx = pos_q.partition_point(|&p| p <= alvo).saturating_sub(1);
        int_q[idx]
    }

    fn batida_mais_proxima(pos_q: &[i32], alvo: i32) -> Option<usize> {
        if pos_q.is_empty() {
            return None;
        }
        let mut lo = 0isize;
        let mut hi = pos_q.len() as isize - 1;
        while lo <= hi {
            let mid = ((hi - lo) >> 1) + lo;
            let limite = if mid == 0 {
                pos_q[0]
            } else {
                (pos_q[(mid - 1) as usize] + pos_q[mid as usize]) / 2
            };
            if alvo < limite {
                hi = mid - 1;
            } else {
                lo = mid + 1;
            }
        }
        Some(hi.max(0) as usize)
    }
}

/// `UnitBeatAdjust` (`+0x608` do `Analyzer`), refinador final do modo 0:
/// init `0x142609070`, alimentação `0x142609260` e ajuste `0x142609560`,
/// conferidos no assembly.
#[derive(Debug, Clone)]
pub struct AjusteDeBatidas {
    /// `taxa/4` (`+0xc`).
    pub fs4: i32,
    /// Total de frames (`+8`) e limite do envelope (`+0x1c`,
    /// `min(total/4, fs4·900)`).
    pub total: i32,
    pub limite: i32,
    /// Envelope decimado por 4 (`+0x48`).
    pub envelope: Vec<i16>,
    contados: i32,
    hp: SecaoRbj,
    lp: SecaoRbj,
    /// Grades candidatas (`+0x30`) e o índice da próxima (`+0x18`).
    candidatas: Vec<Vec<i32>>,
    batidas: i32,
}

// As negações e o `max`/`min` reproduzem `comisd`/`vmaxsd`/`vminsd` com NaN.
#[allow(clippy::neg_cmp_op_on_partial_ord, clippy::manual_clamp)]
impl AjusteDeBatidas {
    pub fn novo(taxa: i32, total: i32) -> Self {
        let fs4 = taxa / 4;
        let limite = (total / 4).min(fs4 * 900);
        Self {
            fs4,
            total,
            limite,
            envelope: Vec::new(),
            contados: 0,
            hp: SecaoRbj::passa_alta_2d(2000.0, f64::from(fs4)),
            lp: SecaoRbj::passa_baixa_2d(1000.0, f64::from(fs4)),
            candidatas: Vec::new(),
            batidas: 0,
        }
    }

    /// Meia janela de acúmulo: `(int)(fs4·0,05)`.
    fn meia(&self) -> i32 {
        (f64::from(self.fs4) * 0.05) as i32
    }

    /// Alimentação por bloco (`0x142609260`): 1 de cada 4 frames,
    /// `x = (R + L)·0,5` → passa-alta 2D de 2 000 Hz → `|·|` → passa-baixa 2D
    /// de 1 000 Hz, uma amostra por chamada, limitado a `[0, 32767]` em `i16`.
    /// Respeita o total de frames e o limite de 900 s.
    pub fn alimentar(&mut self, estereo: &[[i16; 2]]) {
        let mut n = estereo.len() as i32;
        if self.total < self.contados * 4 + n {
            n = self.total - self.contados * 4;
        }
        let cap = self.fs4 * 900;
        let len = self.envelope.len() as i32;
        if cap < n / 4 + len {
            n = (cap - len) * 4;
        }
        let mut k = 0;
        while k < n {
            let [l, r] = estereo[k as usize];
            let x = [(f64::from(r) + f64::from(l)) * 0.5];
            let mut y = [0.0];
            self.hp.processar(&x, &mut y);
            let a = [y[0].abs()];
            self.lp.processar(&a, &mut y);
            let v = y[0];
            let s = if v <= 32767.0 {
                if 0.0 <= v {
                    v as i32 as i16
                } else {
                    0
                }
            } else {
                0x7fff
            };
            self.envelope.push(s);
            self.contados += 1;
            k += 4;
        }
    }

    /// Soma do envelope em `lo..hi` no acumulador (`0x14260a9a0`).
    fn acumular(&self, acc: &mut [f64], lo: i32, hi: i32) {
        let n = hi - lo;
        if lo < 0 || lo >= hi || hi > self.limite || n > self.meia() * 2 {
            return;
        }
        if n as usize > self.envelope.len() {
            return;
        }
        for (i, a) in acc.iter_mut().take(n as usize).enumerate() {
            // shortcut: o original lê além do envelope se `hi` passar dele; aqui o resto conta 0.
            let e = self.envelope.get(lo as usize + i).copied().unwrap_or(0);
            *a += f64::from(e);
        }
    }

    /// Máximo do acumulador e índice do primeiro (`0x14260ac30`); parte de 0.
    fn pico(acc: &[f64]) -> (f64, i32) {
        let (mut m, mut idx) = (0.0f64, 0i32);
        for (i, &v) in acc.iter().enumerate() {
            if v > m {
                idx = i as i32;
            }
            m = if v > m { v } else { m };
        }
        (m, idx)
    }

    /// Grade candidata de um BPM (`0x142609b50`): `(int)(fma(passo, k, início)
    /// + 0,5)` a `fs/4`, até `total/4`.
    fn candidata(&mut self, bpm: f64, inicio: i32) {
        if !(bpm > 0.0) || inicio < 0 || inicio >= self.limite || self.batidas <= 0 {
            return;
        }
        let passo = f64::from(self.fs4) * (60.0 / bpm);
        let fim = f64::from(self.total / 4);
        let mut g = Vec::new();
        let mut x = 0.0f64;
        for k in 0..self.batidas {
            x = passo.mul_add(f64::from(k), f64::from(inicio));
            if x < fim {
                g.push((x + 0.5) as i32);
            }
        }
        while passo + x < fim {
            x += passo;
            g.push((x + 0.5) as i32);
        }
        self.candidatas.push(g);
    }

    /// Varredura de `passos` BPMs a partir de `bpm0` em 0,002 (`0x14260a270`).
    fn varrer(
        &mut self,
        bpm0: f64,
        passos: i32,
        inicio: i32,
        n: i32,
        melhor: &mut (f64, f64, i32),
    ) {
        if !(bpm0 > 0.0)
            || !(1..=17).contains(&passos)
            || inicio < 0
            || inicio >= self.limite
            || n <= 0
        {
            return;
        }
        let w = self.meia();
        let mut nota = 0.0f64;
        for i in 0..passos {
            let b = f64::from(i).mul_add(0.002, bpm0).max(70.0).min(180.0);
            let antes = self.candidatas.len();
            self.candidata(b, inicio);
            let idx = self.candidatas.len() as i32 - 1;
            if self.candidatas.len() > antes {
                let g = self.candidatas[idx as usize].clone();
                let m = (g.len() as i32).min(n);
                let mut acc = vec![0.0f64; (w * 2).max(0) as usize];
                for &p in g.iter().take((m - 1).max(1) as usize).skip(1) {
                    self.acumular(&mut acc, p - w, p + w);
                }
                nota = Self::pico(&acc).0;
            }
            if melhor.1 < nota {
                *melhor = (b, nota, idx);
            }
        }
    }

    /// Ajuste final (`0x142609560`): varre o BPM a ±0,05 em torno do atual
    /// (`0x142609780`), aplica a melhor grade candidata (`0x14260a560`) e
    /// desloca a grade pela fase de maior energia (`0x14260ad60`).
    pub fn ajustar(&mut self, bpm: &mut f32, grade: &mut Vec<BatidaGrade>) {
        if !(*bpm > 0.0) || grade.is_empty() {
            return;
        }
        self.batidas = grade.len() as i32;
        let cap = self.fs4 * 900;
        let n = if (self.envelope.len() as i32) < cap {
            grade.len() as i32
        } else {
            ((-(f64::from(grade[0].pos))).mul_add(0.25, f64::from(cap)) / f64::from(self.fs4)
                * f64::from(*bpm * 0.016_666_668)) as i32
        };
        self.buscar(bpm, grade, n);
        let w = self.meia();
        let mut acc = vec![0.0f64; (w * 2).max(0) as usize];
        let m = grade.len();
        for b in grade.iter().take(m.saturating_sub(1)).skip(1) {
            let p = b.pos / 4;
            self.acumular(&mut acc, p - w, p + w);
        }
        let (_, idx) = Self::pico(&acc);
        self.deslocar(w, idx, grade);
    }

    /// `0x142609780` e `0x14260a560`.
    fn buscar(&mut self, bpm: &mut f32, grade: &mut Vec<BatidaGrade>, n: i32) {
        if f64::from(*bpm) <= 0.05 || n < 1 {
            return;
        }
        let inicio = grade[0].pos / 4;
        let b = f64::from(*bpm);
        let mut melhor = (0.0f64, 0.0f64, 0i32);
        let base = self.candidatas.len() as i32;
        let r = |m: &(f64, f64, i32)| (m.2 - base) as u32;
        self.varrer(b - 0.016, 17, inicio, n, &mut melhor);
        let mut extra = None;
        if r(&melhor) < 3 {
            self.varrer(b - 0.034, 9, inicio, n, &mut melhor);
            if r(&melhor).wrapping_sub(17) <= 2 {
                extra = Some(b - 0.05);
            }
        } else if r(&melhor).wrapping_sub(15) <= 2 {
            self.varrer(b + 0.018, 9, inicio, n, &mut melhor);
            if r(&melhor).wrapping_sub(24) <= 2 {
                extra = Some(b + 0.036);
            }
        }
        if let Some(x) = extra {
            self.varrer(x, 8, inicio, n, &mut melhor);
        }
        let d13 = melhor.0 - 0.001;
        let d12 = melhor.0 * 1000.0;
        let f6 = *bpm * 1000.0;
        if (f6 as i32) - 50 < d12 as i32 {
            self.varrer(d13, 1, inicio, n, &mut melhor);
        }
        if (d12 as i32) < (f6 as i32) + 50 {
            self.varrer(d13 + 0.002, 1, inicio, n, &mut melhor);
        }
        self.aplicar(melhor.0, melhor.2, bpm, grade);
    }

    /// `0x14260a560`: se o BPM mudar no milésimo, a grade passa a ser a
    /// candidata (posições ×4, intervalo `60000/bpm`) e o BPM, `(f32)bpm`.
    fn aplicar(&mut self, melhor: f64, idx: i32, bpm: &mut f32, grade: &mut Vec<BatidaGrade>) {
        let Some(g) = usize::try_from(idx)
            .ok()
            .and_then(|i| self.candidatas.get(i))
            .cloned()
        else {
            return;
        };
        if !(melhor > 0.0) || !(*bpm > 0.0) || grade.is_empty() {
            return;
        }
        // `novo <= atual − 1 || atual + 1 <= novo` no original: muda no milésimo.
        let atual = (*bpm * 1000.0) as i32;
        let novo = (melhor * 1000.0) as i32;
        if novo == atual {
            return;
        }
        let iv = (60000.0 / melhor) as f32;
        while self.batidas != g.len() as i32 {
            if self.batidas < g.len() as i32 {
                let mut c = grade[self.batidas as usize - 1];
                c.numero = if c.numero == 4 { 1 } else { c.numero + 1 };
                c.ancora = false;
                grade.push(c);
                self.batidas += 1;
            } else {
                grade.pop();
                self.batidas -= 1;
            }
        }
        for (b, &p) in grade.iter_mut().zip(&g) {
            b.pos = p << 2;
            b.intervalo = iv;
        }
        *bpm = melhor as f32;
    }

    /// `0x14260ad60`: desloca a grade em `(idx − w)·4` amostras (menos 1 ms
    /// quando a primeira batida ficaria antes de uma batida de distância ou
    /// o deslocamento passa de 5 ms), apara o que sai do sinal e estende as
    /// bordas em passos de uma batida, renumerando.
    fn deslocar(&mut self, w: i32, idx: i32, grade: &mut Vec<BatidaGrade>) {
        let lim = self.meia() * 2;
        if w < 0 || w > lim || idx < 0 || idx > lim || grade.is_empty() {
            return;
        }
        let ms4 = (f64::from(self.fs4) * 0.001) as i32;
        let mut desl = (idx - w) * 4;
        let batida = (grade[0].intervalo * 0.004 * self.fs4 as f32) as i32;
        if grade[0].pos + desl < batida || 0 < desl - ms4 * 20 {
            desl -= ms4 * 4;
        }
        let (mut neg, mut fora) = (0usize, 0usize);
        for b in grade.iter_mut() {
            b.pos += desl;
            if b.pos < 0 {
                neg += 1;
            }
            if b.pos + desl > self.total {
                fora += 1;
            }
        }
        grade.drain(..neg.min(grade.len()));
        let manter = grade.len().saturating_sub(fora);
        grade.truncate(manter);
        if grade.is_empty() || batida <= 0 {
            self.batidas = grade.len() as i32;
            return;
        }
        while grade[0].pos - batida >= 0 {
            let mut c = grade[0];
            c.pos -= batida;
            c.numero = if c.numero == 1 { 4 } else { c.numero - 1 };
            c.ancora = false;
            grade.insert(0, c);
        }
        while grade[grade.len() - 1].pos + batida <= self.total {
            let mut c = grade[grade.len() - 1];
            c.pos += batida;
            c.numero = if c.numero == 4 { 1 } else { c.numero + 1 };
            c.ancora = false;
            grade.push(c);
        }
        self.batidas = grade.len() as i32;
    }
}

/// Orquestrador completo do motor 1 (`BeatAnalyzer_1_0`, `0x142606e40` /
/// `0x142607110`, rekordbox 7.2.19) no modo 0.
///
/// Liga `Alimentador` → `SensorDeBatidas` → `contar_bpm_modo0` →
/// `RefinadorDeEnvelope` → `AjusteDeBatidas` numa API única em blocos.
#[derive(Debug, Clone)]
pub struct AnalisadorDeBatidas {
    pub taxa: u32,
    pub total: i32,
    pub frase: Frase,
    alimentador: Alimentador,
    sensor: SensorDeBatidas,
    ref_env: RefinadorDeEnvelope,
    ajuste: AjusteDeBatidas,
    fechado: bool,
}

impl AnalisadorDeBatidas {
    /// Inicializa o analisador para uma faixa de `total_frames` a `taxa` Hz.
    pub fn novo(taxa: u32, total_frames: i32, frase: Frase) -> Option<Self> {
        let sensor = SensorDeBatidas::novo(taxa, total_frames)?;
        Some(Self {
            taxa,
            total: total_frames,
            frase,
            alimentador: Alimentador::novo(taxa, total_frames.max(0) as usize),
            sensor,
            ref_env: RefinadorDeEnvelope::new(taxa as i32, total_frames),
            ajuste: AjusteDeBatidas::novo(taxa as i32, total_frames),
            fechado: false,
        })
    }

    /// Alimenta blocos `l`/`r` (mesmo comprimento). Devolve `true` quando
    /// `total` frames forem atingidos.
    ///
    /// Como em `0x142606e40`, `sensor`, `ref_env` e `ajuste` são alimentados
    /// exclusivamente dentro do callback de bloco do `Alimentador` (`taxa/75`
    /// frames por bloco), garantindo invariância exata ao tamanho do chunk de
    /// entrada.
    pub fn alimentar(&mut self, l: &[i16], r: &[i16]) -> bool {
        if self.fechado {
            return true;
        }
        let n = l.len().min(r.len());
        let mut lf = Vec::with_capacity(n);
        let mut rf = Vec::with_capacity(n);
        for i in 0..n {
            lf.push(f32::from(l[i]));
            rf.push(f32::from(r[i]));
        }
        let sensor = &mut self.sensor;
        let ref_env = &mut self.ref_env;
        let ajuste = &mut self.ajuste;
        let fim = self.alimentador.alimentar(&lf, &rf, |bloco| {
            sensor.alimentar(bloco.mono);
            ref_env.alimentar_pares(bloco.estereo);
            ajuste.alimentar(bloco.estereo);
        });
        if fim {
            self.fechado = true;
        }
        fim
    }

    /// Executa o fechamento (`0x142607110`): contagem modo 0, refinamento de
    /// envelope e `UnitBeatAdjust`.
    pub fn fechar(&mut self) -> (f32, Vec<BatidaGrade>) {
        self.fechar_modo(0)
    }

    /// Executa o fechamento no modo especificado (`0` = modo 0, `1` = modo 1),
    /// conforme `0x142607110` (`decomp/142607110.c:44-54`): `UnitBeatAdjust`
    /// (`0x142609560`) só roda quando `modo == 0`.
    pub fn fechar_modo(&mut self, modo: u8) -> (f32, Vec<BatidaGrade>) {
        let mut lista = self.sensor.lista.batidas.clone();
        let (mut bpm, mut grade) = if modo == 1 {
            contar_bpm_modo1(&mut lista, self.taxa as i32, self.total, self.frase)
        } else {
            contar_bpm_modo0(&mut lista, self.taxa as i32, self.total, self.frase)
        };
        if bpm > 0.0 && !grade.is_empty() {
            self.ref_env.processar(bpm, &mut grade, false);
            if modo == 0 {
                self.ajuste.ajustar(&mut bpm, &mut grade);
            }
        }
        (bpm, grade)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ajuste_refina_bpm_e_fase() {
        // Cliques a 120,02 BPM; a contagem diz 120,00.
        let taxa = 44_100;
        let passo = 60.0 / 120.02 * 44_100.0;
        let n = 44_100 * 30;
        let mut est = vec![[0i16; 2]; n];
        for b in 0..58 {
            let i0 = (5000.0 + b as f64 * passo) as usize;
            for (k, v) in est[i0..i0 + 88].iter_mut().enumerate() {
                let s = if k % 2 == 0 { 20_000 } else { -20_000 };
                *v = [s, s];
            }
        }
        let mut aj = AjusteDeBatidas::novo(taxa, n as i32);
        for bloco in est.chunks(588) {
            aj.alimentar(bloco);
        }
        assert_eq!(aj.envelope.len(), n / 4);
        let mut grade: Vec<_> = (0..58)
            .map(|k| BatidaGrade {
                pos: 5000 + k * 22_050,
                campo4: 0.0,
                intervalo: 500.0,
                numero: k % 4 + 1,
                marca: -1,
                ancora: false,
            })
            .collect();
        let mut bpm = 120.0f32;
        aj.ajustar(&mut bpm, &mut grade);
        assert!((bpm - 120.02).abs() < 0.0015, "{bpm}");
        assert!(grade.windows(2).all(|w| w[0].pos < w[1].pos));
        assert!(grade.iter().all(|b| b.pos >= 0 && b.pos <= n as i32));
    }

    #[test]
    fn regrade_e_numeracao() {
        let b = |pos, marca| BatidaGrade {
            pos,
            campo4: 0.0,
            intervalo: 0.0,
            numero: 0,
            marca,
            ancora: false,
        };
        let mut g = vec![b(30_000, 0), b(52_050, -1), b(74_100, 3)];
        regrade_uniforme(&mut g, 120_000, 0, 2, 44_100);
        // passo 22 050; k = 1; início 7 950; 2 + 2 + (int)(45 900/22 050) = 2.
        let pos: Vec<i32> = g.iter().map(|x| x.pos).collect();
        assert_eq!(pos, vec![7950, 30_000, 52_050, 74_100, 96_150, 118_200]);
        assert!(g[1].ancora && g[1].marca == 0 && g[3].marca == 3);
        assert!(g.iter().all(|x| x.intervalo == 500.0));
        numerar(&mut g, 30_000, 120_000, Frase::PADRAO);
        let num: Vec<i32> = g.iter().map(|x| x.numero).collect();
        assert_eq!(num, vec![4, 1, 2, 3, 4, 1]);
    }

    #[test]
    fn contagem_modo0_de_ponta_a_ponta() {
        // Lista de sensor a 120 BPM por 20 s.
        let mut l: Vec<_> = (0..40)
            .map(|k| BatidaSensor {
                pos: 3000 + k * 22_050,
                batidas: 1.0,
                intervalo: 500.0,
                etiqueta: 3,
                banda: 0,
            })
            .collect();
        let total = 3000 + 40 * 22_050;
        let (bpm, g) = contar_bpm_modo0(&mut l, 44_100, total, Frase::PADRAO);
        assert!((bpm - 120.0).abs() < 0.01, "{bpm}");
        assert!(g.len() >= 38);
        assert!(g.iter().all(|b| b.pos >= 0 && b.pos <= total));
        assert!(g.iter().all(|b| (1..=4).contains(&b.numero)));
        assert!(
            contar_bpm_modo0(&mut Vec::new(), 44_100, 1000, Frase::PADRAO)
                .1
                .is_empty()
        );
    }

    #[test]
    fn audio_de_cliques_vira_120_bpm() {
        // Cliques de 2 ms a 120 BPM, 20 s a 44,1 kHz: sensor → lista → contagem.
        let n = 44_100 * 20;
        let mut x = vec![0.0f32; n];
        for b in 0..40 {
            let i0 = b * 22_050 + 1000;
            for (k, v) in x[i0..i0 + 88].iter_mut().enumerate() {
                *v = if k % 2 == 0 { 0.8 } else { -0.8 };
            }
        }
        let mut s = SensorDeBatidas::novo(44_100, n as i32).unwrap();
        for bloco in x.chunks(588) {
            s.alimentar(bloco);
        }
        let mut lista = s.lista.batidas.clone();
        let (bpm, g) = contar_bpm_modo0(&mut lista, 44_100, n as i32, Frase::PADRAO);
        assert!((bpm - 120.0).abs() < 0.5, "{bpm}");
        assert!(g.len() >= 30, "{}", g.len());
    }

    #[test]
    fn grade_de_tempo_constante() {
        // 40 batidas a 120 BPM a partir de 3 000, banda 0 (compensação −30 ms
        // = −1 323 amostras na âncora).
        let l: Vec<_> = (0..40)
            .map(|k| BatidaSensor {
                pos: 3000 + k * 22_050,
                batidas: 1.0,
                intervalo: 500.0,
                etiqueta: 0,
                banda: 0,
            })
            .collect();
        let total = 3000 + 40 * 22_050;
        let mut segs = vec![Segmento::from(l[0])];
        ajustar_fase(&l, &mut segs, 44_100, total);
        let g = gerar_grade(
            &l,
            &segs,
            &[PONTUACAO_INICIAL],
            44_100,
            total,
            Frase::PADRAO,
        );
        let b = &g.batidas;
        assert!(b.len() >= 38, "{}", b.len());
        assert!(b.windows(2).all(|w| w[0].pos < w[1].pos));
        // Passo constante de 22 050 amostras (500 ms) entre as batidas.
        for w in b.windows(2) {
            assert!(
                (w[1].pos - w[0].pos - 22_050).abs() <= 1,
                "{:?}",
                (w[0].pos, w[1].pos)
            );
        }
        assert_eq!(b.iter().filter(|x| x.ancora).count(), 1);
        assert!((b.last().unwrap().intervalo - 500.0).abs() < 0.01);
        assert!(g.primeiro_forte >= 0);
    }

    #[test]
    fn mais_proxima_respeita_janela_e_empate() {
        let l: Vec<_> = [100, 200, 300].iter().map(|&p| batida(p, 500.0)).collect();
        assert_eq!(mais_proxima(&l, 190, 20), Some(1));
        assert_eq!(mais_proxima(&l, 250, 50), Some(1)); // empate: a primeira
        assert_eq!(mais_proxima(&l, 250, 10), None);
        assert_eq!(mais_proxima(&l, 50, 60), Some(0));
        assert_eq!(mais_proxima(&[], 0, 10), None);
    }

    #[test]
    fn pontuacao_soma_proximidade() {
        let l: Vec<_> = (0..4).map(|k| batida(1000 + k * 22_050, 500.0)).collect();
        let (s, p) = pontuar_grade(&l, 1000.0, 90_000.0, 22_050.0, 100);
        assert_eq!((s, p), (400.0, 1000.0));
        let (s, _) = pontuar_grade(&l, 1050.0, 90_000.0, 22_050.0, 100);
        assert_eq!(s, 200.0);
    }

    #[test]
    fn fase_cai_na_primeira_batida() {
        // Batidas a 120 BPM a partir de 3 000 amostras.
        let l: Vec<_> = (0..16).map(|k| batida(3000 + k * 22_050, 500.0)).collect();
        let mut segs = vec![Segmento::from(l[0])];
        ajustar_fase(&l, &mut segs, 44_100, 3000 + 16 * 22_050);
        let s = segs[0];
        // Com k = 4 (passo de 4 batidas), t = 25 056 casa a segunda batida a
        // 6 amostras, melhor que t = 2 976 (24 da primeira): recuo de 1
        // passo, fase 25 050 − 22 050 = 3 000.
        assert_eq!(s.intervalo, 500.0);
        assert_eq!(s.campo_c, 16);
        assert_eq!((s.pos, s.campo10), (3000, 1));
    }

    #[test]
    fn deduplicacao_fica_com_a_mais_forte() {
        let b = |pos, etiqueta| BatidaSensor {
            pos,
            batidas: 1.0,
            intervalo: 500.0,
            etiqueta,
            banda: 0,
        };
        // Geração 0 com força 3, geração 1 com força 6.
        let mut lista = vec![
            b(0, 3),
            b(1000, 3),
            b(1003, 0x106),
            b(22_050, 3),
            b(22_052, 3),
        ];
        let segs = [Segmento::from(lista[0])];
        // Distância: (int)(44100·500·1,25e−4) = 2756.
        deduplicar(&mut lista, &segs, 44_100, 100_000);
        let pos: Vec<i32> = lista.iter().map(|b| b.pos).collect();
        // 1000 está a 1000 de 0: sai 1000 (empate de força → sai a atual);
        // 1003 (força 6) tira 0; 22 052 sai (empate).
        assert_eq!(pos, vec![1003, 22_050]);
    }

    #[test]
    fn bpm_vem_do_segmento_mais_longo() {
        let s = |pos, intervalo| Segmento {
            pos,
            campo4: 0,
            intervalo,
            campo_c: 4,
            campo10: 0,
        };
        let segs = [s(10_000, 500.0), s(50_000, 400.0), s(200_000, 600.0)];
        // Extensões: 0..50 000, 50 000..200 000, 200 000..260 000.
        assert_eq!(bpm_do_mais_longo(&segs, 260_000), 150.0);
        assert_eq!(bpm_do_mais_longo(&[], 1), 0.0);
    }

    #[test]
    fn segmentos_compativeis_sao_fundidos() {
        // Duas regiões de 500 ms (120 BPM) e uma de 400 ms (150 BPM).
        let mut lista = Vec::new();
        for k in 0..8 {
            lista.push(batida(k * 22_050, 500.0));
        }
        for k in 0..8 {
            lista.push(batida(176_400 + k * 17_640, 400.0));
        }
        let mut segs = vec![
            Segmento::from(lista[0]),
            Segmento::from(lista[4]),
            Segmento::from(lista[8]),
        ];
        refinar_segmentos(&lista, &mut segs, 400_000);
        // O segundo (120 BPM) cai na janela do primeiro e é fundido.
        let r: Vec<(i32, f32, u32)> = segs
            .iter()
            .map(|s| (s.pos, s.intervalo, s.campo_c))
            .collect();
        assert_eq!(r, vec![(0, 500.0, 4), (176_400, 400.0, 4)]);
        assert!(segs.iter().all(|s| s.campo4 == 0));
    }

    fn batida(pos: i32, intervalo: f32) -> BatidaSensor {
        BatidaSensor {
            pos,
            batidas: 1.0,
            intervalo,
            etiqueta: 0,
            banda: 0,
        }
    }

    #[test]
    fn tabela_conta_e_refina() {
        let lista: Vec<_> = (0..6).map(|k| batida(k * 22_050, 500.0)).collect();
        let mut t = TabelaBpm::nova(400, 0);
        // Janela de 3 batidas: conta as de pos 0..=66 150 (4 batidas).
        assert_eq!(t.contar(&lista, 0, 3 * 22_050), 4);
        assert_eq!(t.caixas[120], 4);
        assert_eq!(t.primeiro_maximo(0, 400), 120);
        let mut f = TabelaBpm::nova(4000, 1);
        f.contar(&lista, 0, i32::MAX);
        assert_eq!(f.caixas[1200], 6);
        assert_eq!(f.refinar(120, 0), 1200);
        assert_eq!(f.refinar(130, 0), -1);
    }

    #[test]
    fn modo0_estima_e_poda() {
        let mut lista = vec![
            batida(0, 500.0),
            batida(22_050, 500.5),
            batida(30_000, 333.0), // fora de 500 ± 20, do dobro e da metade
            batida(44_100, 499.8),
            batida(66_150, 1000.0), // metade = 500: fica
        ];
        let (iv0, semente) = estimar_modo0(&mut lista, i32::MAX).unwrap();
        assert_eq!(iv0, 500.0);
        assert_eq!(semente.pos, 0);
        assert_eq!(semente.intervalo, 500.0);
        let pos: Vec<i32> = lista.iter().map(|b| b.pos).collect();
        assert_eq!(pos, vec![0, 22_050, 44_100, 66_150]);
        assert!(estimar_modo0(&mut Vec::new(), 100).is_none());
    }

    #[test]
    fn casamento_de_intervalos_iguais() {
        // Quatro intervalos de 500 ms no anel e x = 500. Linhas de FAIXAS_X:
        // 1,25 não fecha (passa de 4 com T = 4), 0,75 também; com r = 1,
        // três do anel somam 4 batidas; a extensão usa o quarto.
        let mut r = RegistroDeIntervalos::default();
        for _ in 0..4 {
            r.anel[r.escrita as usize] = Intervalo {
                pos: 0,
                pico: 1.0,
                ms: 500.0,
            };
            r.escrita += 1;
            r.quantidade += 1;
        }
        let c = r.casar(500.0, 0).unwrap();
        assert_eq!(
            c,
            Casamento {
                proxima: 5,
                media: 500.0,
                erro: 0.0,
                contagem: 4
            }
        );
        assert_eq!(r.casar(500.0, 5), None);
        assert!(r.agrupar(500.0));
        assert_eq!((r.intervalo, r.contagem), (500.0, 4));
        // Anel vazio: nada casa.
        assert!(!RegistroDeIntervalos::default().agrupar(500.0));
    }

    #[test]
    fn seguidor_fecha_picos_espacados() {
        // Pulso de 1,0 num passo a cada 2 756 passos (22 048 amostras).
        let mut s = SeguidorDePico::novo(44_100, 8, 0);
        let mut fechados = Vec::new();
        for t in 0..20_000 {
            let v = if t % 2756 == 100 { 1.0 } else { 0.0 };
            if let Some(p) = s.passo(v) {
                fechados.push(p);
            }
        }
        assert!(fechados.len() >= 6, "{fechados:?}");
        for w in fechados.windows(2) {
            assert_eq!(w[1].0 - w[0].0, 2756 * 8);
            assert_eq!(w[1].1, 1.0);
        }
    }

    #[test]
    fn sensor_acha_o_intervalo_de_cliques() {
        // Cliques de 2 ms a 120 BPM (22 050 amostras), 12 s a 44,1 kHz.
        let n = 44_100 * 12;
        let mut x = vec![0.0f32; n];
        for b in 0..24 {
            let i0 = b * 22_050 + 1000;
            for (k, v) in x[i0..i0 + 88].iter_mut().enumerate() {
                *v = if k % 2 == 0 { 0.8 } else { -0.8 };
            }
        }
        let mut s = SensorDeBatidas::novo(44_100, n as i32).unwrap();
        let mut escolheu = false;
        for bloco in x.chunks(588) {
            escolheu |= s.alimentar(bloco);
        }
        assert!(escolheu);
        let (iv, _, _) = s.selecao.unwrap();
        assert!((iv - 500.0).abs() < 2.0, "{iv}");
        // A moda do histograma cai em 120 BPM.
        assert_eq!(s.histograma.moda(), 120);
        // Lista ordenada. Bandas diferentes marcam o mesmo clique com atrasos
        // de filtro diferentes (dezenas de amostras); só a posição exata é
        // deduplicada. O início de cada grupo fica a 22 050 amostras do
        // anterior.
        let l = &s.lista.batidas;
        assert!(l.len() >= 4, "{l:?}");
        assert!(l.windows(2).all(|w| w[0].pos <= w[1].pos));
        let mut grupos = vec![l[0].pos];
        for w in l.windows(2).filter(|w| w[1].pos - w[0].pos > 2000) {
            grupos.push(w[1].pos);
        }
        assert!(grupos.len() >= 3, "{grupos:?}");
        for g in grupos.windows(2) {
            let d = g[1] - g[0];
            assert!((d - 22_050).abs() <= 300, "{grupos:?}");
        }
        assert!(l.iter().all(|b| (b.intervalo - 500.0).abs() < 2.0));
    }

    fn registro_com(intervalos: &[f32], media: f32, contagem: i32) -> RegistroDeIntervalos {
        let mut r = RegistroDeIntervalos::default();
        for (k, &ms) in intervalos.iter().enumerate() {
            r.anel[r.escrita as usize] = Intervalo {
                pos: 1000 * (k as i32 + 1),
                pico: 1.0,
                ms,
            };
            r.escrita += 1;
            r.quantidade += 1;
        }
        r.intervalo = media;
        r.contagem = contagem;
        r.erro = 0.0;
        r.valido = true;
        r
    }

    #[test]
    fn escolha_exige_tres_votos_e_invalida_incompativeis() {
        let mut h = HistogramaBpm::default();
        let mut regs = vec![
            registro_com(&[500.0; 4], 500.0, 4),
            registro_com(&[700.0; 4], 700.0, 4),
        ];
        // Votos: 120 BPM com peso 4 e 86 BPM com peso 4; empate: primeira
        // caixa (86) é a moda. A melhor (menor erro, primeira) é 500 ms,
        // longe de 60000/86; a caixa da melhor tem 4 votos: poda. Sobra 120.
        assert_eq!(h.escolher(&mut regs), Some(0));
        assert_eq!(h.moda(), 120);
        assert_eq!(h.caixas[86], 0);
        assert!(regs[0].valido && !regs[1].valido);
        // Sem votos suficientes: nenhuma escolha.
        let mut h = HistogramaBpm::default();
        let mut regs = vec![registro_com(&[500.0; 4], 500.0, 4)];
        regs[0].erro = 1.0; // peso (int)(4/2) = 2 < 3
        assert_eq!(h.escolher(&mut regs), None);
    }

    #[test]
    fn lista_insere_em_ordem_e_fica_com_o_mais_forte() {
        let mut l = ListaDeBatidas::default();
        let fraco = registro_com(&[500.0; 4], 500.0, 1);
        l.registrar(500.0, &fraco, 0);
        // contagem 1 → 3 entradas, da mais antiga (pos 2000) à recente (4000).
        let pos: Vec<i32> = l.batidas.iter().map(|b| b.pos).collect();
        assert_eq!(pos, vec![2000, 3000, 4000]);
        assert!(l
            .batidas
            .iter()
            .all(|b| b.batidas == 1.0 && b.etiqueta == 3));
        assert_eq!(l.etiqueta, 0x100);
        // Mesmas posições, mesmo tempo e força maior: substitui.
        let forte = registro_com(&[500.0; 4], 500.0, 2);
        l.registrar(500.0, &forte, 1);
        assert_eq!(l.batidas.len(), 4);
        assert!(l
            .batidas
            .iter()
            .all(|b| b.banda == 1 && b.etiqueta == 0x104));
        // Força menor no mesmo tempo: não entra.
        l.registrar(500.0, &fraco, 2);
        assert!(l.batidas.iter().all(|b| b.banda == 1));
    }

    /// |H(e^jw)| de uma seção.
    fn ganho(s: &SecaoRbj, f: f64, fs: f64) -> f64 {
        let [b0, b1, b2, a1, a2] = s.c;
        let w = std::f64::consts::TAU * f / fs;
        let (z1r, z1i) = (w.cos(), -w.sin());
        let (z2r, z2i) = ((2.0 * w).cos(), -(2.0 * w).sin());
        let nr = b0 + b1 * z1r + b2 * z2r;
        let ni = b1 * z1i + b2 * z2i;
        let dr = 1.0 + a1 * z1r + a2 * z2r;
        let di = a1 * z1i + a2 * z2i;
        ((nr * nr + ni * ni) / (dr * dr + di * di)).sqrt()
    }

    #[test]
    fn projetos_rbj_tem_os_ganhos_esperados() {
        let fs = 44_100.0;
        let lp = SecaoRbj::passa_baixa(283.0, fs, Q_BUTTERWORTH);
        assert!((ganho(&lp, 0.0, 44_100.0) - 1.0).abs() < 1e-12);
        assert!((ganho(&lp, 283.0, 44_100.0) - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-3);
        let hp = SecaoRbj::passa_alta(7900.0, fs, Q_BUTTERWORTH);
        assert!(ganho(&hp, 0.0, 44_100.0) < 1e-12);
        assert!((ganho(&hp, 22_049.0, 44_100.0) - 1.0).abs() < 1e-3);
        let bp = SecaoRbj::passa_faixa(150.0, fs, 1.5);
        assert!((ganho(&bp, 150.0, 44_100.0) - 1.0).abs() < 1e-3);
        assert!(ganho(&bp, 0.0, 44_100.0) < 1e-12);
        assert_eq!(bp.c[1], 0.0);
        assert_eq!(bp.c[2], -bp.c[0]);
        assert_eq!(DOIS_PI.to_bits(), 0x401921fb54442d11);
    }

    #[test]
    fn biquad_segue_a_recorrencia_e_continua_entre_blocos() {
        let x: Vec<f64> = (0..600)
            .map(|i| ((i * 37 % 101) as f64 - 50.0) / 50.0)
            .collect();
        let s0 = SecaoRbj::passa_baixa(283.0, 44_100.0, Q_BUTTERWORTH);
        let [b0, b1, b2, a1, a2] = s0.c;
        let mut r = vec![0.0f64; x.len()];
        for i in 0..x.len() {
            let g = |v: &[f64], k: usize| if i >= k { v[i - k] } else { 0.0 };
            r[i] = b0 * x[i] + b1 * g(&x, 1) + b2 * g(&x, 2) - a1 * g(&r, 1) - a2 * g(&r, 2);
        }
        let mut s = s0.clone();
        let mut y = vec![0.0; x.len()];
        s.processar(&x, &mut y);
        let mut s2 = s0;
        let mut y2 = vec![0.0; x.len()];
        s2.processar(&x[..1], &mut y2[..1]);
        s2.processar(&x[1..7], &mut y2[1..7]);
        s2.processar(&x[7..], &mut y2[7..]);
        for i in 0..x.len() {
            assert!((y[i] - r[i]).abs() < 1e-12, "{i}");
            assert!((y2[i] - r[i]).abs() < 1e-12, "{i}");
        }
        assert_eq!(s.c, s2.c);
    }

    fn seno(f: f32, fs: f32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| (std::f32::consts::TAU * f * i as f32 / fs).sin())
            .collect()
    }

    fn rms(v: &[f32]) -> f32 {
        (v.iter().map(|x| x * x).sum::<f32>() / v.len() as f32).sqrt()
    }

    #[test]
    fn banco_separa_graves_e_agudos() {
        let mut banco = BancoDeBandas::novo(44_100).unwrap();
        assert_eq!(banco.bloco, 588);
        assert_eq!(banco.bandas[4].media.as_ref().unwrap().len(), 44);
        assert_eq!(banco.bandas[5].media.as_ref().unwrap().len(), 9);
        let graves = banco.processar(&seno(80.0, 44_100.0, 44_100));
        let cauda = 22_050..;
        assert!(rms(&graves[0][cauda.clone()]) > 0.5);
        assert!(rms(&graves[3][cauda.clone()]) < 1e-3);
        let mut banco = BancoDeBandas::novo(44_100).unwrap();
        let agudos = banco.processar(&seno(12_000.0, 44_100.0, 44_100));
        assert!(rms(&agudos[3][cauda.clone()]) > 0.5);
        assert!(rms(&agudos[0][cauda.clone()]) < 1e-3);
        // Retificação e média móvel: nada negativo.
        assert!(graves.iter().flatten().all(|v| *v >= 0.0));
        assert!(BancoDeBandas::novo(22_050).is_none());
        assert_eq!(BancoDeBandas::novo(48_000).unwrap().bloco, 640);
    }

    #[test]
    fn alimentacao_mistura_e_corta_em_blocos() {
        let mut a = Alimentador::novo(44_100, 1300);
        let l = vec![16_384.0f32; 1000];
        let r = vec![-8_192.5f32; 1000];
        let mut blocos = Vec::new();
        let fim = a.alimentar(&l, &r, |b| {
            blocos.push((b.mono.len(), b.mono[0], b.estereo[0], b.ultimo))
        });
        assert!(!fim);
        assert_eq!(
            blocos,
            vec![(
                588,
                (16_384.0 - 8_192.5) * ESCALA_MONO,
                [16_384, -8_192],
                false
            )]
        );
        blocos.clear();
        let fim = a.alimentar(&l, &r, |b| {
            blocos.push((b.mono.len(), 0.0, [0, 0], b.ultimo))
        });
        assert!(fim);
        // 412 restantes + 176 = bloco cheio (1 176 frames). O próximo bloco
        // não é cortado no total: enche 588 (1 764 >= 1 300) e só então fecha.
        assert_eq!(
            blocos,
            vec![(588, 0.0, [0, 0], false), (588, 0.0, [0, 0], true)]
        );
    }

    #[test]
    fn refinador_de_envelope_preenche_e_numera_grade() {
        let taxa = 44_100i32;
        let duracao_s = 8;
        let total = taxa * duracao_s;
        let mut stereo = vec![0i16; (total * 2) as usize];
        // Pulso grave de 80 Hz a cada 0,5 s (120 BPM), com acento na batida 1/4.
        let passo = taxa / 2;
        for b in 0..(duracao_s * 2) {
            let amp = if b % 4 == 0 { 24_000.0 } else { 12_000.0 };
            let ini = (b * passo) as usize;
            for k in 0..1024usize {
                if ini + k < total as usize {
                    let s = (amp
                        * (2.0 * std::f64::consts::PI * 80.0 * k as f64 / f64::from(taxa)).sin())
                        as i16;
                    stereo[(ini + k) * 2] = s;
                    stereo[(ini + k) * 2 + 1] = s;
                }
            }
        }
        let mut ref_env = RefinadorDeEnvelope::new(taxa, total);
        ref_env.alimentar(&stereo);
        assert!(ref_env.envelope.iter().any(|&v| v > 0));

        let mut grade: Vec<BatidaGrade> = (0..16)
            .map(|i| BatidaGrade {
                pos: i * passo,
                campo4: 0.0,
                intervalo: 500.0,
                numero: 0,
                marca: -1,
                ancora: false,
            })
            .collect();
        ref_env.processar(120.0, &mut grade, false);
        assert!(!grade.is_empty());
        assert!(grade.iter().all(|b| (1..=4).contains(&b.numero)));
    }

    #[test]
    fn analisador_de_batidas_ponta_a_ponta() {
        let taxa = 44_100u32;
        let duracao_s = 12;
        let total = (taxa * duracao_s) as usize;
        let mut l = vec![0i16; total];
        let mut r = vec![0i16; total];
        // Cliques de 120 BPM (a cada 0,5 s) com acento na batida 1/4.
        let passo = (taxa / 2) as usize;
        for b in 0..(duracao_s * 2) as usize {
            let amp = if b % 4 == 0 { 26_000.0 } else { 14_000.0 };
            let ini = b * passo;
            for k in 0..256usize {
                if ini + k < total {
                    let s = (amp
                        * (2.0 * std::f64::consts::PI * 100.0 * k as f64 / f64::from(taxa)).sin())
                        as i16;
                    l[ini + k] = s;
                    r[ini + k] = s;
                }
            }
        }
        let mut analisador = AnalisadorDeBatidas::novo(taxa, total as i32, Frase::PADRAO).unwrap();
        for (cl, cr) in l.chunks(4096).zip(r.chunks(4096)) {
            analisador.alimentar(cl, cr);
        }
        let (bpm, grade) = analisador.fechar();
        assert!((bpm - 120.0).abs() < 1.0, "bpm={bpm}");
        assert!(!grade.is_empty());
        assert!(grade.iter().all(|b| (1..=4).contains(&b.numero)));

        // Invariância exata ao tamanho do bloco de entrada (513 vs 4096).
        let mut analisador_513 =
            AnalisadorDeBatidas::novo(taxa, total as i32, Frase::PADRAO).unwrap();
        for (cl, cr) in l.chunks(513).zip(r.chunks(513)) {
            analisador_513.alimentar(cl, cr);
        }
        let (bpm_513, grade_513) = analisador_513.fechar();
        assert_eq!(bpm.to_bits(), bpm_513.to_bits());
        assert_eq!(grade, grade_513);
    }

    #[test]
    fn passa_alta_1d_coeficientes_e_resposta() {
        let mut hp = PassaAlta1D::novo(0.5, 44_100.0 / 128.0);
        assert!(hp.b0 > 0.99 && hp.b0 < 1.0);
        assert_eq!(hp.b1, -hp.b0);
        assert_eq!(hp.a0, 1.0);
        assert!(hp.a1 < -0.98 && hp.a1 > -1.0);
        // Degrau DC deve decair em direção a zero (tau ~ 110 amostras).
        let y0 = hp.filtrar_amostra(1000.0);
        let mut y_fim = y0;
        for _ in 0..1000 {
            y_fim = hp.filtrar_amostra(1000.0);
        }
        assert!(y0 > 990.0);
        assert!(y_fim.abs() < 1.0);
    }

    #[test]
    fn contar_bpm_modo1_multi_segmento_e_ajuste_local() {
        let taxa = 44_100i32;
        let total = taxa * 30;
        let mut lista = Vec::new();
        // Segmento 1 (0..15 s): 120 BPM (intervalo = 500 ms, passo = 22050).
        let mut pos = 0i32;
        for g in 0..30u32 {
            lista.push(BatidaSensor {
                pos,
                batidas: 1.0,
                intervalo: 500.0,
                etiqueta: (g << 8) | 10,
                banda: 0,
            });
            pos += 22_050;
        }
        // Segmento 2 (15..30 s): 150 BPM (intervalo = 400 ms, passo = 17640).
        for g in 30..65u32 {
            if pos >= total {
                break;
            }
            lista.push(BatidaSensor {
                pos,
                batidas: 1.0,
                intervalo: 400.0,
                etiqueta: (g << 8) | 10,
                banda: 0,
            });
            pos += 17_640;
        }
        let (bpm, grade) = contar_bpm_modo1(&mut lista, taxa, total, Frase::PADRAO);
        assert!(bpm > 0.0);
        assert!(!grade.is_empty());
        assert!(grade.iter().all(|b| (1..=4).contains(&b.numero)));
    }
}
