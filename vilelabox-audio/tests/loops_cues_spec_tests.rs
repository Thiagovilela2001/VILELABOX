//! Casos de `spec/rekordbox/32-loops-cues.md` (observados no rekordbox 7.2.8,
//! dossiê `evidence/p6-1-loops-cues.md`).
use std::sync::Arc;
use vilelabox_audio::{AudioBuffer, Deck, TransportState};

fn deck() -> Deck {
    let mut d = Deck::new();
    d.load_audio(Arc::new(AudioBuffer::silence(44_100, 44_100 * 60)), None);
    d
}

#[test]
fn hot_cue_pula_e_toca_parado_ou_tocando() {
    let mut d = deck();
    d.jump_and_play(100_000);
    assert_eq!((d.position_frames, d.state), (100_000, TransportState::Playing));
    d.seek_frame(500_000);
    d.jump_and_play(100_000);
    assert_eq!((d.position_frames, d.state), (100_000, TransportState::Playing));
}

#[test]
fn memory_cue_define_cue_e_para() {
    let mut d = deck();
    d.play();
    d.set_cue_and_standby(1102);
    assert_eq!((d.cue_frame, d.position_frames), (1102, 1102));
    assert_eq!(d.state, TransportState::CueStandby);
}

#[test]
fn loop_comeca_na_posicao_e_metade_dobro_mantem_inicio() {
    let mut d = deck();
    d.play();
    d.seek_frame(1_649_781); // posição da captura
    d.auto_loop(4 * 20_000);
    assert_eq!(d.loop_range, Some((1_649_781, 1_649_781 + 80_000)));
    d.loop_cut(2, 1);
    assert_eq!(d.loop_range, Some((1_649_781, 1_649_781 + 160_000)));
    d.loop_cut(1, 2);
    d.loop_cut(1, 2);
    assert_eq!(d.loop_range, Some((1_649_781, 1_649_781 + 40_000)));
    d.clear_loop();
    assert!(!d.loop_active);
}
