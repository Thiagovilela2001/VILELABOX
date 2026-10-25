//! Curvas de ganho do mixer medidas na saída do rekordbox 7.2.8
//! (spec/rekordbox/33-mixer-fx.md, dossiê `evidence/p7-0-mixer-curvas.md`).
//!
//! Valores dos controles nas unidades do rekordbox (UIA): fader 0..100,
//! TRIM -10..10, crossfader -10..10. Ganho em dB relativo à posição neutra,
//! interpolado linearmente em dB entre os pontos medidos.

/// Fader de canal: (valor, dB). 0 = mudo.
const FADER: [(f32, f32); 13] = [
    (1.0, -95.32),
    (2.0, -88.26),
    (5.0, -76.45),
    (10.0, -61.90),
    (20.0, -39.44),
    (30.0, -24.76),
    (40.0, -16.18),
    (50.0, -12.00),
    (60.0, -9.56),
    (70.0, -7.13),
    (80.0, -4.70),
    (90.0, -2.23),
    (100.0, 0.0),
];

/// TRIM: (valor, dB). -10 = mudo. Positivos medidos com o fader em 50
/// (fora do limitador do master).
const TRIM: [(f32, f32); 10] = [
    (-8.0, -24.17),
    (-6.0, -11.87),
    (-4.0, -6.06),
    (-2.0, -2.55),
    (0.0, 0.0),
    (2.0, 1.97),
    (4.0, 3.51),
    (6.0, 4.94),
    (8.0, 7.18),
    (10.0, 8.78),
];

/// Crossfader visto do canal da esquerda (deck 1): (valor, dB relativo ao
/// lado todo do canal). +10 = mudo.
const CROSS: [(f32, f32); 13] = [
    (-10.0, 0.0),
    (-2.0, 0.0),
    (-1.0, -0.14),
    (0.0, -1.06),
    (1.0, -1.93),
    (2.0, -2.91),
    (3.0, -4.07),
    (4.0, -5.40),
    (5.0, -6.99),
    (6.0, -8.94),
    (7.0, -11.53),
    (8.0, -15.02),
    (9.0, -21.00),
];

fn interp(t: &[(f32, f32)], v: f32) -> f32 {
    if v <= t[0].0 {
        return t[0].1;
    }
    for w in t.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        if v <= x1 {
            return y0 + (y1 - y0) * (v - x0) / (x1 - x0);
        }
    }
    t[t.len() - 1].1
}

fn lin(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

/// Ganho linear do fader de canal. O rekordbox só aceita valores inteiros
/// (0,75 vira 1 e 0,25 vira 0); o mesmo arredondamento aqui.
#[must_use]
pub fn ganho_fader(v: f32) -> f32 {
    let v = v.round();
    if v <= 0.0 { 0.0 } else { lin(interp(&FADER, v.min(100.0))) }
}

/// Ganho linear do TRIM.
#[must_use]
pub fn ganho_trim(v: f32) -> f32 {
    if v <= -10.0 { 0.0 } else { lin(interp(&TRIM, v.min(10.0))) }
}

/// Ganho linear do crossfader para o canal `canal` (0 = esquerda/deck 1,
/// 1 = direita/deck 2). O lado direito é o espelho do medido (INFERIDO).
#[must_use]
pub fn ganho_cross(canal: usize, v: f32) -> f32 {
    let v = if canal == 0 { v } else { -v };
    if v >= 10.0 {
        return 0.0;
    }
    if v > 9.0 {
        // ponytail: +9..+10 não medido (o deck parou na medição fina); queda
        // linear em ganho de -21 dB até o mudo. Medir 9,25/9,5/9,75 e trocar.
        return lin(-21.0) * (10.0 - v);
    }
    lin(interp(&CROSS, v.max(-10.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db(g: f32) -> f32 {
        20.0 * g.log10()
    }

    #[test]
    fn pontos_medidos() {
        assert!((db(ganho_fader(50.0)) + 12.0).abs() < 0.01);
        assert!((db(ganho_fader(90.0)) + 2.23).abs() < 0.01);
        assert_eq!(ganho_fader(0.0), 0.0);
        assert!((db(ganho_trim(-2.0)) + 2.55).abs() < 0.01);
        assert!((db(ganho_trim(10.0)) - 8.78).abs() < 0.01);
        assert_eq!(ganho_trim(-10.0), 0.0);
        assert!((db(ganho_cross(0, 0.0)) + 1.06).abs() < 0.01);
        assert!((db(ganho_cross(0, -5.0))).abs() < 0.01);
        assert_eq!(ganho_cross(0, 10.0), 0.0);
        // espelho: o deck 2 no lado dele fica cheio, no lado oposto mudo
        assert!((db(ganho_cross(1, 5.0))).abs() < 0.01);
        assert_eq!(ganho_cross(1, -10.0), 0.0);
    }

    #[test]
    fn interpolacao_em_db() {
        // entre 40 (-16,18) e 50 (-12,00): 45 -> -14,09
        assert!((db(ganho_fader(45.0)) + 14.09).abs() < 0.01);
    }
}
