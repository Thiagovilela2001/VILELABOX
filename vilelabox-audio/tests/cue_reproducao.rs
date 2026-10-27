//! Reprodução do relato: marcar o cue, tocar e apertar CUE deve voltar ao
//! cue marcado (spec/rekordbox/30-deck.md, CUE "tocando"), não ao começo.
use std::sync::Arc;
use vilelabox_audio::{AudioBuffer, AudioCommand, AudioMixer, TransportState};

fn tocar(m: &mut AudioMixer, frames: usize) {
    let mut out = vec![0.0f32; 2 * frames];
    m.process_interleaved(&mut out);
}

fn cue(m: &mut AudioMixer) {
    m.apply(AudioCommand::CueDown { deck: 1 });
    m.apply(AudioCommand::CueUp { deck: 1 });
}

#[test]
fn cue_marcado_parado_volta_nele_ao_apertar_cue_tocando() {
    let mut m = AudioMixer::offline();
    m.apply(AudioCommand::LoadTrack {
        deck: 1,
        audio: Arc::new(AudioBuffer::synthetic_sine(44_100, 440.0, 10.0)),
        cue_frame: Some(0),
    });
    // Toca, pausa no meio e marca o cue ali.
    m.apply(AudioCommand::PlayPause { deck: 1 });
    tocar(&mut m, 44_100);
    m.apply(AudioCommand::PlayPause { deck: 1 });
    let marcado = m.deck1.position_frames;
    assert!(marcado > 0);
    cue(&mut m);
    assert_eq!(m.deck1.cue_frame, marcado);
    // Toca de novo e aperta CUE com a música rodando.
    m.apply(AudioCommand::PlayPause { deck: 1 });
    tocar(&mut m, 44_100);
    cue(&mut m);
    assert_eq!(m.deck1.state, TransportState::CueStandby);
    assert_eq!(m.deck1.position_frames, marcado);
}
