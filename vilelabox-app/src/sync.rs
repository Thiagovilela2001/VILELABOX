//! BEAT SYNC, MASTER, KEY SYNC, AU/MA e MEMORY no nível do app
//! (spec/rekordbox/30-deck.md, 32-loops-cues.md; dossiê
//! `evidence/p6-2-sync-slip-quantize.md`).

use crate::state::AppState;
#[cfg(test)]
use vilelabox_anlz::Beat;
use vilelabox_anlz::RawCue;

/// Tom em (semitom 0..12, menor). Aceita "Gm", "F#m", "Bb", "A".
fn tom(s: &str) -> Option<(i32, bool)> {
    let mut c = s.trim().chars();
    let base: i32 = match c.next()? {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let resto = c.as_str();
    let (acid, resto) = match resto.chars().next() {
        Some('#') => (1, &resto[1..]),
        Some('b') => (-1, &resto[1..]),
        _ => (0, resto),
    };
    let menor = match resto {
        "m" => true,
        "" => false,
        _ => return None,
    };
    Some(((base + acid).rem_euclid(12), menor))
}

/// KEY SYNC: semitons para levar `own` a uma tonalidade compatível com
/// `reference`. A regra vem de `p6-10`: o destino é a tonalidade do outro deck,
/// sua dominante (+7) ou sua subdominante (+5) — as vizinhas no círculo de
/// quintas — e vale a de menor deslocamento absoluto. Partindo de `G -2`, de
/// `C +3` e de `F -4` contra um deck em `F`, o rekordbox levou o deck a `Bb +1`
/// nas três, ou seja, o deslocamento anterior é ignorado. `p6-2` (`Gm` -> `Fm`,
/// −2) é o mesmo cálculo com outro par. Só para o mesmo modo (menor/maior);
/// modo diferente é DESCONHECIDO, e o desempate de ±6 também.
pub fn key_sync_shift(own: &str, reference: &str) -> Option<i8> {
    let (a, ma) = tom(own)?;
    let (b, mb) = tom(reference)?;
    if ma != mb {
        return None;
    }
    let mut melhor: Option<i8> = None;
    for cand in [b, (b + 7).rem_euclid(12), (b + 5).rem_euclid(12)] {
        let d = ((cand - a + 6).rem_euclid(12) - 6) as i8;
        if melhor.is_none_or(|m| (d as i32).abs() < (m as i32).abs()) {
            melhor = Some(d);
        }
    }
    melhor
}

/// Tom deslocado `n` semitons: `Gm` -2 = `Fm`. Grafia: `Bb` observado na
/// tela (p6-7, A +1 = `Bb`); as demais notas alteradas não foram observadas e
/// ficam com sustenido.
pub fn transpor(key: &str, n: i8) -> Option<String> {
    const NOTAS: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "Bb", "B",
    ];
    if n == 0 {
        return None;
    }
    let (t, menor) = tom(key)?;
    let nota = NOTAS[(t + i32::from(n)).rem_euclid(12) as usize];
    Some(format!("{nota}{}", if menor { "m" } else { "" }))
}

/// Deslocamento de tom exibido no deck `i`: o do KEY SYNC mais, com MT
/// desligado, o efeito da velocidade arredondado ao semitom (p6-7: A a
/// 172/163 = `Bb +1`).
pub fn shift_exibido(st: &AppState, i: usize) -> i8 {
    let v = st.velocidade[i];
    let pela_velocidade = if !st.master_tempo[i] && v > 0.0 {
        (12.0 * v.log2()).round() as i8
    } else {
        0
    };
    st.key_shift[i].saturating_add(pela_velocidade)
}

/// Texto do deslocamento como na tela: "±0", "-2", "+3".
pub fn rotulo_shift(n: i8) -> String {
    if n == 0 {
        "±0".into()
    } else {
        format!("{n:+}")
    }
}

/// Variação de andamento como na tela: 130,00 -> 130,14 = "+0.1%" (p6-2).
pub fn rotulo_pitch(velocidade: f64) -> String {
    format!("{:+.1}%", (velocidade - 1.0) * 100.0)
}

/// BPM da faixa carregada no deck `i` (sem ajuste).
pub fn bpm_base(st: &AppState, i: usize) -> f64 {
    let d = if i == 0 { &st.deck1 } else { &st.deck2 };
    d.loaded_track_idx
        .and_then(|t| st.tracks.get(t))
        .map_or(0.0, |t| t.bpm())
}

/// Andamento tocado no deck `i` (base × velocidade).
pub fn bpm_efetivo(st: &AppState, i: usize) -> f64 {
    bpm_base(st, i) * st.velocidade[i]
}

/// BEAT SYNC: `None` se o deck é o master (nada muda, p6-2); senão o BPM do
/// master, que o deck assume.
pub fn bpm_do_master(st: &AppState, i: usize) -> Option<f64> {
    (i != st.master).then(|| bpm_efetivo(st, st.master))
}

/// Espelha `Deck::set_beat_sync` para a tela.
pub fn aplicar_sync(st: &mut AppState, i: usize) {
    if let (true, Some(m)) = (st.beat_sync[i], bpm_do_master(st, i)) {
        let own = bpm_base(st, i);
        if own > 0.0 && m > 0.0 {
            st.velocidade[i] = m / own;
        }
    }
}

/// Posição no áudio do deck escravo que casa com a fase e o número da batida
/// atualmente ouvida no master. O andamento é tratado separadamente por
/// `aplicar_sync`; aqui corrigimos a defasagem entre as duas grades.
#[must_use]
#[cfg(test)]
pub fn posicao_sincronizada(
    master: &[Beat],
    master_pos_s: f64,
    slave: &[Beat],
    slave_pos_s: f64,
) -> Option<f64> {
    if master.len() < 2 || slave.len() < 2 {
        return None;
    }

    let master_ms = master_pos_s.max(0.0) * 1000.0;
    let mi = master
        .iter()
        .rposition(|b| f64::from(b.time_ms) <= master_ms)
        .unwrap_or(0)
        .min(master.len() - 2);
    let ma = f64::from(master[mi].time_ms);
    let mb = f64::from(master[mi + 1].time_ms);
    if mb <= ma {
        return None;
    }
    let fase = ((master_ms - ma) / (mb - ma)).clamp(0.0, 1.0);
    let numero = master[mi].beat_number;
    let slave_ms = slave_pos_s.max(0.0) * 1000.0;

    // Casar também 1/2/3/4 evita uma defasagem de uma ou mais batidas.
    let si = (0..slave.len() - 1)
        .filter(|&i| slave[i].beat_number == numero)
        .min_by(|&a, &b| {
            (f64::from(slave[a].time_ms) - slave_ms)
                .abs()
                .total_cmp(&(f64::from(slave[b].time_ms) - slave_ms).abs())
        })?;
    let sa = f64::from(slave[si].time_ms);
    let sb = f64::from(slave[si + 1].time_ms);
    (sb > sa).then_some((sa + fase * (sb - sa)) / 1000.0)
}

/// MEMORY (p6-2): grava um memory cue na posição atual, na lista do app;
/// nenhum método do deck foi chamado. O chamador persiste (`cues_store`).
pub fn gravar_memory_cue(cues: &mut Vec<RawCue>, pos_s: f64) {
    cues.push(memory_cue(pos_s));
}

/// Hot cue `slot` (1..8 = A..H) gravado em `pos_s`, já quantizada pelo
/// chamador (`loops::quantizar_s`, slot 11 do comportamento quantizado, p6-2).
/// Um cue por slot: o novo substitui o anterior.
pub fn gravar_hot_cue(cues: &mut Vec<RawCue>, slot: u32, pos_s: f64) {
    cues.retain(|c| c.is_loop || c.hot_cue != slot);
    cues.push(RawCue {
        hot_cue: slot,
        ..memory_cue(pos_s)
    });
}

fn memory_cue(pos_s: f64) -> RawCue {
    RawCue {
        hot_cue: 0,
        status: 1,
        is_loop: false,
        start_time_s: pos_s,
        end_time_s: None,
        label: None,
        color: None,
    }
}

/// Casos de `32-loops-cues.md` e `30-deck.md` (dossiê p6-2) no nível do app.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_sync_escolhe_a_tonalidade_compativel_mais_proxima() {
        // p6-10: deck 1 em A contra deck 2 em F -> Bb +1 (subdominante de F),
        // partindo de G -2, de C +3 e de F -4 (o deslocamento anterior e ignorado).
        assert_eq!(key_sync_shift("A", "F"), Some(1));
        assert_eq!(transpor("A", 1).as_deref(), Some("Bb"));
        // p6-2: Gm contra Fm -> Fm -2 (a propria tonalidade do outro deck).
        assert_eq!(key_sync_shift("Gm", "Fm"), Some(-2));
        // p6-7: Bb contra F -> nada a fazer, Bb ja e a subdominante de F.
        assert_eq!(key_sync_shift("Bb", "F"), Some(0));
        // mesma tonalidade: sem deslocamento.
        assert_eq!(key_sync_shift("A", "A"), Some(0));
    }

    #[test]
    fn key_sync_gm_para_fm_menos_2() {
        assert_eq!(key_sync_shift("Gm", "Fm"), Some(-2));
        assert_eq!(rotulo_shift(-2), "-2");
        assert_eq!(transpor("Gm", -2).as_deref(), Some("Fm"));
        assert_eq!(rotulo_shift(0), "±0");
        assert_eq!(key_sync_shift("Gm", "Gm"), Some(0));
        assert_eq!(key_sync_shift("Gm", "F"), None); // modos diferentes: desconhecido
    }

    #[test]
    fn beat_sync_master_nao_muda_e_nao_master_segue() {
        let mut st = AppState::new();
        st.master = 0;
        st.beat_sync = [true, true];
        assert_eq!(bpm_do_master(&st, 0), None);
        aplicar_sync(&mut st, 0);
        assert_eq!(st.velocidade[0], 1.0);
        assert_eq!(rotulo_pitch(130.14 / 130.0), "+0.1%");
    }

    #[test]
    fn beat_sync_alinha_fase_e_numero_da_batida() {
        let mk = |n, ms| Beat {
            beat_number: n,
            bpm: 120.0,
            time_ms: ms,
        };
        let master = [
            mk(1, 1000),
            mk(2, 1500),
            mk(3, 2000),
            mk(4, 2500),
            mk(1, 3000),
        ];
        let slave = [
            mk(1, 800),
            mk(2, 1400),
            mk(3, 2000),
            mk(4, 2600),
            mk(1, 3200),
        ];

        let alvo = posicao_sincronizada(&master, 1.625, &slave, 1.95).unwrap();
        assert!((alvo - 1.55).abs() < 1e-9);
    }

    #[test]
    fn beat_sync_sem_grade_nao_inventa_posicao() {
        assert_eq!(posicao_sincronizada(&[], 1.0, &[], 2.0), None);
    }

    #[test]
    fn mt_desligado_tom_segue_a_velocidade() {
        // p6-7: deck 1 em A, sync com 172 BPM sobre faixa de 163, MT desligado -> Bb +1.
        let mut st = AppState::new();
        st.velocidade[0] = 172.0 / 163.0;
        st.master_tempo[0] = false;
        assert_eq!(shift_exibido(&st, 0), 1);
        assert_eq!(transpor("A", 1).as_deref(), Some("Bb"));
        assert_eq!(rotulo_shift(1), "+1");
        st.master_tempo[0] = true; // MT ligado: tom preso
        assert_eq!(shift_exibido(&st, 0), 0);
    }

    #[test]
    fn au_ma_alterna_so_o_modo() {
        let mut st = AppState::new();
        let antes = (st.loop_batidas, st.loop_ativo);
        st.loop_manual[0] = !st.loop_manual[0];
        assert!(st.loop_manual[0]);
        st.loop_manual[0] = !st.loop_manual[0];
        assert!(!st.loop_manual[0]);
        assert_eq!((st.loop_batidas, st.loop_ativo), antes);
    }

    #[test]
    fn memory_grava_na_lista_sem_motor() {
        let mut cues = Vec::new();
        gravar_memory_cue(&mut cues, 51.718);
        let linhas = crate::marks::lista_memoria(&cues);
        assert_eq!(linhas.len(), 1);
        assert_eq!(cues[0].hot_cue, 0);
        assert_eq!(cues[0].start_time_s, 51.718);
    }

    #[test]
    fn hot_cue_q_ligado_batida_anterior_desligado_posicao_exata() {
        use vilelabox_anlz::Beat;
        // 120 BPM: batidas em 1,0 / 1,5 / 2,0 s.
        let g: Vec<Beat> = [1000, 1500, 2000]
            .map(|t| Beat {
                beat_number: 1,
                bpm: 120.0,
                time_ms: t,
            })
            .into();
        let mut cues = Vec::new();
        gravar_hot_cue(&mut cues, 1, crate::loops::quantizar_s(&g, 1.7, true));
        gravar_hot_cue(&mut cues, 2, crate::loops::quantizar_s(&g, 1.7, false));
        assert_eq!((cues[0].hot_cue, cues[0].start_time_s), (1, 1.5));
        assert_eq!((cues[1].hot_cue, cues[1].start_time_s), (2, 1.7));
    }
}
