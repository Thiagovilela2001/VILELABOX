//! Mixer pelo `AudioMixer` real (spec/rekordbox/33-mixer-fx.md, dossiê
//! `evidence/p7-0-mixer-curvas.md`): ganho de saída nos pontos medidos.
use std::sync::Arc;
use vilelabox_audio::{AudioBuffer, AudioCommand, AudioMixer};

/// RMS em dB da saída com o deck 1 tocando um seno, após `cmds`.
fn saida_db(cmds: Vec<AudioCommand>) -> f32 {
    let mut m = AudioMixer::offline();
    m.apply(AudioCommand::LoadTrack {
        deck: 1,
        audio: Arc::new(AudioBuffer::synthetic_sine(44_100, 440.0, 2.0)),
        cue_frame: Some(0),
    });
    m.apply(AudioCommand::Play { deck: 1 });
    for c in cmds {
        m.apply(c);
    }
    let mut out = vec![0.0f32; 2 * 4410];
    m.process_interleaved(&mut out);
    let rms = (out.iter().map(|x| x * x).sum::<f32>() / out.len() as f32).sqrt();
    20.0 * rms.max(1e-12).log10()
}

#[test]
fn fader_trim_e_crossfader_nos_pontos_medidos() {
    let base = saida_db(vec![]); // fader 100, TRIM 0, crossfader 0 (-1,06 dB)
    let rel = |c: Vec<AudioCommand>| saida_db(c) - base;
    assert!((rel(vec![AudioCommand::SetFader { deck: 1, valor: 50.0 }]) + 12.0).abs() < 0.05);
    assert!((rel(vec![AudioCommand::SetTrim { deck: 1, valor: -2.0 }]) + 2.55).abs() < 0.05);
    // crossfader todo para o deck 1: +1,06 dB em relação ao centro
    assert!((rel(vec![AudioCommand::SetCrossfader { valor: -10.0 }]) - 1.06).abs() < 0.05);
    // mudos
    assert!(rel(vec![AudioCommand::SetFader { deck: 1, valor: 0.0 }]) < -150.0);
    assert!(rel(vec![AudioCommand::SetCrossfader { valor: 10.0 }]) < -150.0);
    assert!(rel(vec![AudioCommand::SetTrim { deck: 1, valor: -10.0 }]) < -150.0);
}
