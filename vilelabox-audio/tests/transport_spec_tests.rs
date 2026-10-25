//! Casos de teste de `spec/rekordbox/30-deck.md` (observados no rekordbox
//! 7.2.8, dossiê `evidence/p6-0-transporte.md`). Posição em amostras.
use std::sync::Arc;
use vilelabox_audio::{AudioBuffer, Deck, TransportState};

const CUE: u64 = 393_151; // 8,915 s a 44,1 kHz, valor da captura

fn deck_tocando_ate(pos: u64) -> Deck {
    let mut d = Deck::new();
    d.load_audio(Arc::new(AudioBuffer::silence(44_100, 44_100 * 20)), None);
    d.play();
    d.seek_frame(pos);
    d
}

#[test]
fn play_pausa_e_retoma() {
    let mut d = deck_tocando_ate(CUE);
    d.play_pause();
    assert_eq!(d.state, TransportState::Stopped);
    d.play_pause();
    assert_eq!(d.state, TransportState::Playing);
}

#[test]
fn cue_parado_fora_do_cue_grava_e_espera() {
    let mut d = deck_tocando_ate(CUE);
    d.pause();
    d.cue_down();
    assert_eq!(d.cue_frame, CUE);
    assert_eq!(d.position_frames, CUE);
    assert_eq!(d.state, TransportState::CueStandby);
    d.cue_up(); // soltar: sem efeito novo
    assert_eq!(d.state, TransportState::CueStandby);
}

#[test]
fn segurar_cue_em_espera_toca_previa_e_soltar_volta() {
    let mut d = deck_tocando_ate(CUE);
    d.pause();
    d.cue_down();
    d.cue_up();
    d.cue_down();
    assert_eq!(d.state, TransportState::CuePreview);
    let (mut l, mut r) = (vec![0.0; 512], vec![0.0; 512]);
    d.process_block(&mut l, &mut r);
    assert!(d.position_frames > CUE, "prévia avança");
    d.cue_up();
    assert_eq!(d.position_frames, CUE);
    assert_eq!(d.state, TransportState::CueStandby);
}

#[test]
fn cue_tocando_volta_ao_cue_e_para() {
    let mut d = deck_tocando_ate(CUE);
    d.pause();
    d.cue_down();
    d.cue_up();
    d.play_pause();
    d.seek_frame(CUE + 44_100);
    d.cue_down();
    assert_eq!(d.position_frames, CUE);
    assert_eq!(d.state, TransportState::CueStandby);
    d.cue_up();
    assert_eq!(d.position_frames, CUE);
    assert_eq!(d.state, TransportState::CueStandby);
}
