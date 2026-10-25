//! Conta de loops por batidas a partir da grade do ANLZ (spec/rekordbox/32-loops-cues.md).

use vilelabox_anlz::Beat;

/// Duração de uma batida, em segundos, na posição `pos_s`: intervalo entre a
/// batida da grade que contém a posição e a seguinte; na falta, o BPM dela.
pub fn batida_s(beats: &[Beat], pos_s: f64) -> Option<f64> {
    let pos_ms = pos_s * 1000.0;
    let i = beats.iter().rposition(|b| f64::from(b.time_ms) <= pos_ms).unwrap_or(0);
    let b = beats.get(i)?;
    match beats.get(i + 1) {
        Some(n) if n.time_ms > b.time_ms => Some(f64::from(n.time_ms - b.time_ms) / 1000.0),
        _ if b.bpm > 0.0 => Some(60.0 / b.bpm),
        _ => None,
    }
}

/// Q (p6-2, `0x142ab7130`): com `q`, a posição cai no nó da grade
/// imediatamente antes ou exatamente sobre ela (arredondamento para baixo).
/// Sem `q`, sem grade ou antes da primeira batida: posição intacta.
pub fn quantizar_s(beats: &[Beat], pos_s: f64, q: bool) -> f64 {
    let pos_ms = pos_s * 1000.0;
    match beats.iter().rfind(|b| f64::from(b.time_ms) <= pos_ms) {
        Some(b) if q => f64::from(b.time_ms) / 1000.0,
        _ => pos_s,
    }
}

/// Texto do tamanho do loop: "4", "16", "1/2", "1/4".
pub fn rotulo(batidas: f64) -> String {
    if batidas >= 1.0 {
        format!("{}", batidas.round() as u64)
    } else {
        format!("1/{}", (1.0 / batidas).round() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(t: u32) -> Beat {
        Beat { beat_number: 1, bpm: 120.0, time_ms: t }
    }

    #[test]
    fn intervalo_da_grade_e_rotulo() {
        let g = [b(1000), b(1500), b(2000)];
        assert_eq!(batida_s(&g, 1.2), Some(0.5));
        assert_eq!(batida_s(&g, 2.5), Some(0.5)); // última: usa o BPM
        assert_eq!(batida_s(&[], 1.0), None);
        assert_eq!(rotulo(16.0), "16");
        assert_eq!(rotulo(0.5), "1/2");
    }

    // Casos de Quantize em 32-loops-cues.md (dossiê p6-2, regra para baixo).
    // Grade a 120 BPM: uma batida a cada 0,5 s.
    #[test]
    fn q_ligado_entre_batidas_cai_na_anterior() {
        let g = [b(1000), b(1500), b(2000)];
        assert_eq!(quantizar_s(&g, 1.7, true), 1.5);
        assert_eq!(quantizar_s(&g, 1.999, true), 1.5); // nunca na seguinte
    }

    #[test]
    fn q_ligado_sobre_batida_preserva() {
        let g = [b(1000), b(1500), b(2000)];
        assert_eq!(quantizar_s(&g, 1.5, true), 1.5);
    }

    #[test]
    fn q_desligado_nao_ajusta() {
        let g = [b(1000), b(1500), b(2000)];
        assert_eq!(quantizar_s(&g, 1.7, false), 1.7);
    }

    #[test]
    fn sem_grade_nao_quantiza() {
        assert_eq!(quantizar_s(&[], 1.7, true), 1.7);
        // antes da primeira batida não há nó anterior: não inventar
        assert_eq!(quantizar_s(&[b(1000)], 0.3, true), 0.3);
    }
}
