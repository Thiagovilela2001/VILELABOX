//! `vilelabox-dsp`: primitivas numéricas da análise (P4.1).
//!
//! Só entra aqui o que tem evidência em
//! `spec/rekordbox/evidence/p4-1-analise-batida.md` (rekordbox 7.2.8, análise
//! estática). Cada função cita o endereço de origem. Desvios necessários para
//! Rust seguro são marcados como ADAPTAÇÃO.

pub mod espectro;
pub mod estagios;
pub mod filtro;
pub mod passada_c;
pub mod tonalidade;

/// Faixas de BPM da preferência de análise, `(mínimo, máximo)` em BPM inteiro.
/// Tabela em `0x1434eb4b0`; o índice 0 repete o padrão do índice 1.
pub const FAIXAS_BPM: [(i32, i32); 11] = [
    (70, 180),
    (70, 180),
    (48, 95),
    (58, 115),
    (68, 135),
    (78, 155),
    (88, 175),
    (98, 195),
    (108, 215),
    (118, 235),
    (128, 255),
];

/// BPM × 100 entregue pelo motor: `(int)(bpm * 100.0f + 0.5f)` em `f32`
/// (`0x14107c200`, `0x14247a220`).
///
/// ADAPTAÇÃO: `as i32` satura em estouro e dá 0 para NaN, onde o x86 daria
/// `i32::MIN`.
pub fn bpm_centesimos(bpm: f32) -> i32 {
    (bpm * 100.0f32 + 0.5f32) as i32
}

/// BPM × 100 de uma batida a partir do intervalo em ms:
/// `(int)(6000000.0f / intervalo + 0.5f)` (`0x14247a290`).
pub fn bpm_centesimos_do_intervalo(intervalo_ms: f32) -> i32 {
    (6_000_000.0f32 / intervalo_ms + 0.5f32) as i32
}

/// Leva o BPM × 100 para a faixa `(mínimo, máximo)`: dobra enquanto estiver
/// abaixo do mínimo × 100 e divide por dois (divisão inteira) enquanto estiver
/// acima do máximo × 100 (`0x1411301d0`).
///
/// ADAPTAÇÃO: valores `<= 0` voltam inalterados; no original o laço de dobrar
/// não terminaria.
pub fn ajustar_a_faixa(bpm100: i32, (minimo, maximo): (i32, i32)) -> i32 {
    if bpm100 <= 0 {
        return bpm100;
    }
    let mut b = bpm100;
    while b < minimo * 100 {
        b *= 2;
    }
    while b > maximo * 100 {
        b /= 2;
    }
    b
}

/// Soma de `n = (int)(segundos / bpm * taxa + 0.5)` amostras de `sinal` a
/// partir de `(int)(pos * 0.25 + 0.5)` (`0x142476a40` com 60 s, `0x1424768e0`
/// com 30 s). Devolve −1,0 para BPM nulo, posição `pos / 4 < 0` ou janela
/// além de `comprimento`, e 0,0 quando `n < 1`.
///
/// `comprimento` é o campo `obj+0x1c` do original.
/// ADAPTAÇÃO: a checagem original admite ler o índice `comprimento`; aqui,
/// qualquer leitura fora de `sinal` também devolve −1,0.
fn soma_janela(
    sinal: &[i16],
    comprimento: i32,
    taxa: i32,
    bpm: f32,
    pos: i32,
    segundos: f64,
) -> f64 {
    if bpm.abs() <= f32::EPSILON || pos / 4 < 0 {
        return -1.0;
    }
    let n = (segundos / bpm as f64 * taxa as f64 + 0.5) as i32;
    let inicio = (pos as f64 * 0.25 + 0.5) as i32;
    if comprimento < n - 1 + inicio {
        return -1.0;
    }
    if n < 1 {
        return 0.0;
    }
    match sinal.get(inicio as usize..(inicio + n) as usize) {
        Some(janela) => janela.iter().map(|&s| s as f64).sum(),
        None => -1.0,
    }
}

/// Soma de uma batida (60/BPM s) do sinal (`0x142476a40`, buffer `obj+0x80`).
pub fn soma_batida(sinal: &[i16], comprimento: i32, taxa: i32, bpm: f32, pos: i32) -> f64 {
    soma_janela(sinal, comprimento, taxa, bpm, pos, 60.0)
}

/// Soma de meia batida (30/BPM s) do sinal (`0x1424768e0`, buffer `obj+0x78`).
pub fn soma_meia_batida(sinal: &[i16], comprimento: i32, taxa: i32, bpm: f32, pos: i32) -> f64 {
    soma_janela(sinal, comprimento, taxa, bpm, pos, 30.0)
}

/// Pesos da mistura para mono do detector legado (`0x1411705c0`):
/// 0,99/32768 para o canal do 1º ponteiro e 0,01/32768 para o do 2º
/// (`0x143537e78` e `0x143537e70`). A escala supõe amostras em ±32768.
pub const PESO_CANAL_A: f64 = 3.021240234375e-05;
pub const PESO_CANAL_B: f64 = 3.0517578125e-07;

/// Mistura dois canais planares `f32` em mono `f64`:
/// `b[i] * PESO_CANAL_B + a[i] * PESO_CANAL_A` (`0x1411705c0`). Usa o menor
/// comprimento.
///
/// DESCONHECIDO: qual canal (L ou R) o original passa em `a`.
pub fn mixar_mono(a: &[f32], b: &[f32]) -> Vec<f64> {
    a.iter()
        .zip(b)
        .map(|(&a, &b)| b as f64 * PESO_CANAL_B + a as f64 * PESO_CANAL_A)
        .collect()
}

/// Frames por bloco de leitura do detector legado:
/// `(int)(taxa * 1.7066666666666668 + 0.5)` (`0x141170020`, `0x143537e80`).
pub fn frames_por_bloco(taxa: i32) -> i32 {
    (taxa as f64 * 1.706_666_666_666_666_8 + 0.5) as i32
}

/// Posição (em amostras da taxa do motor) onde o nível por batida "entra",
/// recuperada de `0x142470b60`, que grava o resultado em `motor+0x50`.
///
/// Entradas: `posicoes` = primeiro `i32` de cada batida da lista em
/// `motor+0x58`; `bpm` = `motor+0x48`; `taxa_motor` = `motor+0x14`; `sinal`,
/// `comprimento` e `taxa_sinal` = o objeto em `motor+0x640` das somas.
///
/// INFERIDO: o significado musical (início da parte com energia). O cálculo é
/// OBSERVADO; ver `evidence/p4-1-analise-batida.md`.
pub fn posicao_de_entrada(
    sinal: &[i16],
    comprimento: i32,
    taxa_sinal: i32,
    bpm: f32,
    posicoes: &[i32],
    taxa_motor: i32,
) -> u32 {
    let n = posicoes.len();
    if n == 0 {
        return 0;
    }
    let m = n - 1;
    // Nível por batida (meia batida), com -1,0 substituído pelo anterior.
    let mut x = vec![0.0f64; n];
    for i in 0..m {
        let v = soma_meia_batida(sinal, comprimento, taxa_sinal, bpm, posicoes[i]);
        x[i] = if i != 0 && v < 0.0 { x[i - 1] } else { v };
    }
    // Normalização pelo máximo (piso 0). O original divide em blocos de 16 e
    // multiplica pelo inverso no resto; o arredondamento difere, então é mantido.
    let max = x[..m]
        .iter()
        .fold(0.0f64, |a, &b| if b > a { b } else { a });
    if max.abs() > f64::EPSILON {
        let vetorial = if m > 15 { m - m % 16 } else { 0 };
        let inv = 1.0 / max;
        for (i, v) in x[..m].iter_mut().enumerate() {
            *v = if i < vetorial { *v / max } else { inv * *v };
        }
    }
    // Diferença entre batidas vizinhas; d[0] e d[n-1] ficam 0.
    let mut d = vec![0.0f64; n];
    for i in 1..n.saturating_sub(1) {
        d[i] = x[i] - x[i - 1];
    }
    let (mut ultimo, mut candidato) = (-1i32, -1i32);
    let (mut ligado, mut contagem, mut quietos) = (false, 0i32, 0i32);
    let mut i = 0usize;
    while i + 2 < n {
        let conta = if i == 0 {
            if x[0] > 0.4 {
                ligado = true;
                candidato = 0;
                ultimo = 0;
                true
            } else {
                false
            }
        } else if ligado {
            x[i] > 0.4 || (x[i] > 0.2 && x[i + 1] > 0.2 && x[i + 2] > 0.2)
        } else if x[i] > 0.2 && d[i] > 0.15 {
            if ultimo < 0 || quietos > 32 {
                candidato = i as i32;
            }
            ligado = true;
            quietos = 0;
            ultimo = i as i32;
            true
        } else {
            false
        };
        if conta {
            contagem += 1;
        }
        if contagem > 12 {
            break;
        }
        if (x[i] < 0.2 && x[i + 1] < 0.2 && x[i + 2] < 0.2)
            || (d[i] < -0.4 && d[i + 1] < 0.2 && d[i + 2] < 0.2)
        {
            quietos += 1;
            ligado = false;
            contagem = 0;
        }
        i += 1;
    }
    if contagem > 12 {
        candidato = ultimo;
    }
    let idx = candidato.max(0);
    let pos = (60.0 / bpm as f64) * (idx - 4) as f64 * taxa_motor as f64;
    // `(uint)` do original via 64 bits; negativo vira 0.
    let pos = pos as i64 as i32;
    if pos < 0 {
        0
    } else {
        pos as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centesimos_arredondam_como_o_motor() {
        assert_eq!(bpm_centesimos(128.0), 12800);
        assert_eq!(bpm_centesimos(127.996), 12800);
        assert_eq!(bpm_centesimos(127.994), 12799);
        // 6e6 / 468,75 ms = 12800 (128 BPM).
        assert_eq!(bpm_centesimos_do_intervalo(468.75), 12800);
        assert_eq!(bpm_centesimos_do_intervalo(500.0), 12000);
    }

    #[test]
    fn faixa_dobra_e_divide() {
        let padrao = FAIXAS_BPM[1];
        assert_eq!(ajustar_a_faixa(6000, padrao), 12000);
        assert_eq!(ajustar_a_faixa(25000, padrao), 12500);
        assert_eq!(ajustar_a_faixa(12800, padrao), 12800);
        // 48–95: 140 BPM cai para 70.
        assert_eq!(ajustar_a_faixa(14000, FAIXAS_BPM[2]), 7000);
        // Divisão inteira: 36001 / 2 = 18000.
        assert_eq!(ajustar_a_faixa(36001, padrao), 18000);
        assert_eq!(ajustar_a_faixa(0, padrao), 0);
    }

    #[test]
    fn soma_janela_segue_os_limites_do_original() {
        let sinal: Vec<i16> = (0..100).collect();
        // 120 BPM a taxa 10: n = 60/120*10 = 5; pos 8 -> início 2.
        assert_eq!(
            soma_batida(&sinal, 100, 10, 120.0, 8),
            (2 + 3 + 4 + 5 + 6) as f64
        );
        // Meia batida: n = round(2,5) = 3 (0,5 arredonda para cima).
        assert_eq!(
            soma_meia_batida(&sinal, 100, 10, 120.0, 8),
            (2 + 3 + 4) as f64
        );
        // BPM nulo, posição <= -4 e janela além do comprimento.
        assert_eq!(soma_batida(&sinal, 100, 10, 0.0, 8), -1.0);
        assert_eq!(soma_batida(&sinal, 100, 10, 120.0, -4), -1.0);
        // Fim da janela em 2 + 5 - 1 = 6: comprimento 5 recusa, 6 aceita.
        assert_eq!(soma_batida(&sinal, 5, 10, 120.0, 8), -1.0);
        assert_eq!(soma_batida(&sinal, 6, 10, 120.0, 8), 20.0);
        // pos -1..-3 passa e começa em 0.
        assert_eq!(soma_batida(&sinal, 100, 10, 120.0, -3), 10.0);
        // n < 1 dá 0.
        assert_eq!(soma_batida(&sinal, 100, 1, 1000.0, 0), 0.0);
        // Leitura no índice `comprimento` (aceita pelo original) fica fora da fatia.
        assert_eq!(soma_batida(&sinal[..6], 6, 10, 120.0, 8), -1.0);
    }

    #[test]
    fn entrada_acha_a_primeira_subida_sustentada() {
        // Sinal a taxa 4 (1 amostra por batida a 240 BPM: n = 30/240*4 = 0,5 -> 1).
        // Batidas de 4 em 4 posições -> início = pos/4 = índice da batida.
        let mut sinal = vec![10i16; 40];
        for v in &mut sinal[20..] {
            *v = 1000;
        }
        let posicoes: Vec<i32> = (0..40).map(|b| b * 4).collect();
        // Subida na batida 20; 13 batidas fortes fecham a contagem.
        // Resultado: (60/240) * (20 - 4) * taxa_motor 100 = 400.
        assert_eq!(
            posicao_de_entrada(&sinal, 40, 4, 240.0, &posicoes, 100),
            400
        );
        // Forte desde o começo: índice 0 -> (0 - 4) negativo -> 0.
        let alto = vec![1000i16; 40];
        assert_eq!(posicao_de_entrada(&alto, 40, 4, 240.0, &posicoes, 100), 0);
        assert_eq!(posicao_de_entrada(&alto, 40, 4, 240.0, &[], 100), 0);
    }

    #[test]
    fn mistura_e_bloco_do_detector_legado() {
        let m = mixar_mono(&[32768.0, 0.0], &[0.0, 32768.0, 5.0]);
        assert_eq!(m.len(), 2);
        assert!((m[0] - 0.99).abs() < 1e-12);
        assert!((m[1] - 0.01).abs() < 1e-12);
        assert_eq!(frames_por_bloco(44100), 75264);
        assert_eq!(frames_por_bloco(48000), 81920);
    }
}
