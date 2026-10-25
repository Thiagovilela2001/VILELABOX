//! BEAT SYNC, MASTER, KEY SYNC, AU/MA e MEMORY no nível do app
//! (spec/rekordbox/30-deck.md, 32-loops-cues.md; dossiê
//! `evidence/p6-2-sync-slip-quantize.md`).

use crate::state::AppState;
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

/// KEY SYNC: semitons para levar `own` ao tom de `reference`. Observado
/// `Gm ±0 -> Fm -2` (p6-2). Só para o mesmo modo (menor/maior); casar modos
/// diferentes é DESCONHECIDO. Faixa -6..=5: a escolha no empate de 6
/// semitons é DESCONHECIDA.
pub fn key_sync_shift(own: &str, reference: &str) -> Option<i8> {
    let (a, ma) = tom(own)?;
    let (b, mb) = tom(reference)?;
    (ma == mb).then(|| ((b - a + 6).rem_euclid(12) - 6) as i8)
}

/// Tom deslocado `n` semitons, na grafia com sustenido: `Gm` -2 = `Fm`.
/// A grafia do rekordbox para as notas alteradas não foi observada.
pub fn transpor(key: &str, n: i8) -> Option<String> {
    const NOTAS: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    if n == 0 {
        return None;
    }
    let (t, menor) = tom(key)?;
    let nota = NOTAS[(t + i32::from(n)).rem_euclid(12) as usize];
    Some(format!("{nota}{}", if menor { "m" } else { "" }))
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
    cues.push(RawCue { hot_cue: slot, ..memory_cue(pos_s) });
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
        let g: Vec<Beat> =
            [1000, 1500, 2000].map(|t| Beat { beat_number: 1, bpm: 120.0, time_ms: t }).into();
        let mut cues = Vec::new();
        gravar_hot_cue(&mut cues, 1, crate::loops::quantizar_s(&g, 1.7, true));
        gravar_hot_cue(&mut cues, 2, crate::loops::quantizar_s(&g, 1.7, false));
        assert_eq!((cues[0].hot_cue, cues[0].start_time_s), (1, 1.5));
        assert_eq!((cues[1].hot_cue, cues[1].start_time_s), (2, 1.7));
    }
}
