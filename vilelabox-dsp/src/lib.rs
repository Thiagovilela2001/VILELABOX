//! `vilelabox-dsp`: primitivas numéricas da análise (P4.1).
//!
//! Só entra aqui o que tem evidência em
//! `spec/rekordbox/evidence/p4-1-analise-batida.md` (rekordbox 7.2.8, análise
//! estática). Cada função cita o endereço de origem. Desvios necessários para
//! Rust seguro são marcados como ADAPTAÇÃO.

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
/// `comprimento` é o campo `motor+0x1c` do original.
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

/// Soma de uma batida (60/BPM s) do sinal (`0x142476a40`, buffer `motor+0x80`).
pub fn soma_batida(sinal: &[i16], comprimento: i32, taxa: i32, bpm: f32, pos: i32) -> f64 {
    soma_janela(sinal, comprimento, taxa, bpm, pos, 60.0)
}

/// Soma de meia batida (30/BPM s) do sinal (`0x1424768e0`, buffer `motor+0x78`).
pub fn soma_meia_batida(sinal: &[i16], comprimento: i32, taxa: i32, bpm: f32, pos: i32) -> f64 {
    soma_janela(sinal, comprimento, taxa, bpm, pos, 30.0)
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
}
