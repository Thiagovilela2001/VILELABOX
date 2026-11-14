//! Primitivas de análise de frases (etapa A2.3, rekordbox 7.2.19).
//!
//! Portadas de `0x1412d0aa0`, `0x1412e7460`, `0x1412e8de0` e `0x1412e93f0`.

/// Normaliza um vetor por batida pelo seu máximo (usando `0.003` se o máximo
/// for zero) e devolve as médias por compasso de 4 batidas a partir de
/// `primeiro_forte` (1-indexado no binário; 0-indexado aqui).
///
/// Corresponde a `0x1412d0aa0` e `0x1412e93f0`.
pub fn normalizar_e_agrupar_compassos(valores: &mut [f64], primeiro_forte: usize) -> Vec<f64> {
    if valores.is_empty() {
        return Vec::new();
    }
    let max_val = valores.iter().copied().fold(0.0_f64, f64::max);
    let fator = if max_val == 0.0 { 0.003 } else { 1.0 / max_val };
    for v in valores.iter_mut() {
        *v *= fator;
    }
    agrupar_compassos_media(valores, primeiro_forte)
}

/// Agrupa um vetor por batida em médias de compassos de 4 batidas a partir de
/// `primeiro_forte`.
pub fn agrupar_compassos_media(valores: &[f64], primeiro_forte: usize) -> Vec<f64> {
    if primeiro_forte + 4 > valores.len() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut i = primeiro_forte;
    while i + 4 <= valores.len() {
        let soma: f64 = valores[i..i + 4].iter().sum();
        out.push(soma * 0.25);
        i += 4;
    }
    out
}

/// Detecta transições de energia entre compassos (`0x1412e7460`):
/// marca `1` quando `c[i+1] > c[i] * 1.2` ou `c[i] > c[i+1] * 1.2`, exceto se
/// ambos os compassos forem `<= 750.0` e as batidas 3 e 4 do compasso atual
/// forem `<= 0.09`.
pub fn detectar_transicoes_energia(
    por_batida: &[f64],
    primeiro_forte: usize,
) -> (Vec<f64>, Vec<u8>) {
    let mut compassos = Vec::new();
    let mut i = primeiro_forte;
    while i + 4 <= por_batida.len() {
        let soma: f64 = por_batida[i..i + 4].iter().sum();
        compassos.push(soma);
        i += 4;
    }
    let mut marcas = vec![0u8; compassos.len()];
    for k in 0..compassos.len().saturating_sub(1) {
        let c0 = compassos[k];
        let c1 = compassos[k + 1];
        if (c1 > c0 * 1.2) || (c0 > c1 * 1.2) {
            marcas[k + 1] = 1;
        }
        let b_idx = primeiro_forte + k * 4;
        if c1 <= 750.0
            && c0 <= 750.0
            && por_batida.get(b_idx + 3).copied().unwrap_or(0.0) <= 0.09
            && por_batida.get(b_idx + 4).copied().unwrap_or(0.0) <= 0.09
        {
            marcas[k + 1] = 0;
        }
    }
    (compassos, marcas)
}

/// Detecta transições por razão entre compassos (`0x1412e8de0`):
/// marca `1` quando `c[i+1] / c[i] > 1.2` ou `c[i] / c[i+1] > 1.21`
/// (substituindo zeros por `0.003`).
pub fn detectar_transicoes_razao(por_batida: &[f64], primeiro_forte: usize) -> (Vec<f64>, Vec<u8>) {
    let mut compassos = Vec::new();
    let mut i = primeiro_forte;
    while i + 4 <= por_batida.len() {
        let soma: f64 = por_batida[i..i + 4].iter().sum();
        compassos.push(soma);
        i += 4;
    }
    let mut marcas = vec![0u8; compassos.len()];
    for k in 0..compassos.len().saturating_sub(1) {
        let c0 = if compassos[k] == 0.0 {
            0.003
        } else {
            compassos[k]
        };
        let c1 = if compassos[k + 1] == 0.0 {
            0.003
        } else {
            compassos[k + 1]
        };
        if (c1 > c0 && (c1 / c0) > 1.2) || (c1 < c0 && (c0 / c1) > 1.21) {
            marcas[k + 1] = 1;
        }
    }
    (compassos, marcas)
}

/// Normaliza uma matriz espectral (`n_quadros × n_bandas`, 72 bandas em
/// `0x1412dff80`) pelo máximo global e conta, para cada quadro, os picos
/// locais confirmados (queda abaixo de `0.9 * pico` com vale `> 0.001`) cuja
/// amplitude normalizada supera `0.1` (`0x1457ff728`, `disasm/1412dff80.asm:112-218`).
pub fn extrair_picos_espectrais(matriz: &mut [Vec<f64>]) -> Vec<i32> {
    let max_global = matriz
        .iter()
        .flat_map(|linha| linha.iter().copied())
        .fold(0.0_f64, f64::max);
    if max_global <= 0.0 {
        return vec![0; matriz.len()];
    }
    let inv = 1.0 / max_global;
    for linha in matriz.iter_mut() {
        for v in linha.iter_mut() {
            *v *= inv;
        }
    }

    let mut contagens = Vec::with_capacity(matriz.len());
    for linha in matriz.iter() {
        let mut picos_quadro = Vec::new();
        if linha.len() >= 2 {
            let mut pico = linha[0];
            let mut subindo = false;
            let mut tem_pico = false;
            for i in 1..linha.len() {
                let atual = linha[i];
                let ant = linha[i - 1];
                if atual > ant {
                    subindo = true;
                } else {
                    if subindo {
                        if ant > pico {
                            pico = ant;
                            tem_pico = true;
                        }
                    } else if tem_pico && pico * 0.9 > atual {
                        if atual > 0.001 {
                            picos_quadro.push(pico);
                        }
                        tem_pico = false;
                        pico = 0.0;
                    }
                    subindo = false;
                }
            }
        }
        let qtd = picos_quadro.into_iter().filter(|&p| p > 0.1).count() as i32;
        contagens.push(qtd);
    }
    contagens
}

/// Segunda metade de `0x1412dff80` (`disasm/1412dff80.asm:219-546`): agrega as
/// contagens de picos espectrais por quadro nos compassos de 4 batidas,
/// normaliza pelo maior compasso (`max_compasso.max(1)`), marca compassos
/// ativos (`> 0.36`, `0x1438265e0`) e subidas (`> 1.2×` ao entrar em estado
/// ativo, ou `> 0.2` com razão `> 1.8`, `0x1457ff9f8`), expandindo 4× por
/// batida.
pub fn agregar_picos_espectrais_compassos(
    contagens_quadro: &[i32],
    primeiro_forte: usize,
    n_batidas: usize,
) -> (Vec<u8>, Vec<u8>, Vec<f64>) {
    let n_compassos = n_batidas / 4;
    if n_compassos == 0 || contagens_quadro.is_empty() {
        return (vec![0; 1], vec![0; 1], vec![0.0; 1]);
    }
    let passo_q = (contagens_quadro.len() as f64 * 4.0) / (n_batidas as f64);
    let mut soma_compassos = vec![0.0f64; n_compassos];
    let mut q_idx = primeiro_forte;
    let mut acumulado_passos = 0i32;
    for (c_idx, slot) in soma_compassos.iter_mut().enumerate() {
        let alvo = ((c_idx + 1) as f64).mul_add(passo_q, 0.5) as i32;
        while acumulado_passos < alvo && q_idx < contagens_quadro.len() {
            *slot += f64::from(contagens_quadro[q_idx]);
            q_idx += 1;
            acumulado_passos += 1;
        }
    }
    let max_c = (soma_compassos.iter().copied().fold(0.0_f64, f64::max) as i32).max(1);
    let inv_max = 1.0 / f64::from(max_c);
    let norm_compassos: Vec<f64> = soma_compassos.iter().map(|&v| v * inv_max).collect();

    let n_exp = n_compassos * 4;
    let mut ativo = vec![0u8; n_exp];
    let mut subida = vec![0u8; n_exp];
    for c in 0..n_compassos {
        let v = norm_compassos[c];
        if v > 0.36 {
            ativo[c] = 1;
        }
        if c > 0 {
            let v_ant = norm_compassos[c - 1];
            let subiu = if ativo[c - 1] < ativo[c] {
                v > v_ant * 1.2
            } else {
                v > 0.2 && v_ant > 0.0 && (v / v_ant) > 1.8
            };
            if subiu {
                subida[c] = 1;
            }
        }
    }
    let mut expandido = Vec::with_capacity(n_exp);
    for &v in &norm_compassos {
        expandido.extend([v; 4]);
    }
    (ativo, subida, expandido)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normaliza_e_agrupa_quatro_batidas_por_compasso() {
        let mut v = vec![0.0, 2.0, 2.0, 4.0, 4.0, 1.0, 1.0, 1.0, 1.0];
        let c = normalizar_e_agrupar_compassos(&mut v, 1);
        assert_eq!(c.len(), 2);
        assert!((c[0] - 0.75).abs() < 1e-9);
        assert!((c[1] - 0.25).abs() < 1e-9);
    }

    #[test]
    fn transicoes_energia_e_razao_marcam_degrau() {
        let v = vec![
            1000.0, 1000.0, 1000.0, 1000.0, 2000.0, 2000.0, 2000.0, 2000.0,
        ];
        let (_, me) = detectar_transicoes_energia(&v, 0);
        assert_eq!(me, vec![0, 1]);
        let (_, mr) = detectar_transicoes_razao(&v, 0);
        assert_eq!(mr, vec![0, 1]);
    }

    #[test]
    fn extrair_picos_espectrais_normaliza_e_conta_picos() {
        let mut m = vec![
            vec![0.0, 0.2, 1.0, 0.95, 0.4, 0.1, 0.3, 0.8, 0.78, 0.3],
            vec![0.0; 10],
        ];
        let c = extrair_picos_espectrais(&mut m);
        assert_eq!(c.len(), 2);
        assert_eq!(c[0], 2);
        assert_eq!(c[1], 0);

        let (ativo, subida, exp) =
            agregar_picos_espectrais_compassos(&[1, 1, 1, 1, 4, 4, 4, 4], 0, 8);
        assert_eq!(ativo.len(), 8);
        assert_eq!(ativo[0], 0);
        assert_eq!(ativo[1], 1);
        assert_eq!(subida[1], 1);
        assert_eq!(exp.len(), 8);
    }
}
