use std::sync::Arc;
use vilelabox_audio::{render_offline, AudioBuffer, Deck, TransportState};

fn sample_audio() -> Arc<AudioBuffer> {
    Arc::new(AudioBuffer::synthetic_sine(44100, 440.0, 5.0))
}

#[test]
fn play_pause_transicoes_basicas() {
    let mut deck = Deck::new();
    let audio = sample_audio();
    deck.load_audio(audio, Some(0));

    assert_eq!(deck.state, TransportState::CueStandby);
    assert_eq!(deck.position_frames, 0);

    // Play -> Playing
    deck.play();
    assert_eq!(deck.state, TransportState::Playing);

    // Renderizar 100 frames
    let (l, _) = render_offline(&mut deck, 100, 32);
    assert_eq!(deck.position_frames, 100);
    assert!(l.iter().any(|&s| s != 0.0));

    // Pause -> Stopped na mesma posição
    deck.pause();
    assert_eq!(deck.state, TransportState::Stopped);
    assert_eq!(deck.position_frames, 100);

    // Renderizar pausado não move a agulha e produz silêncio
    let (silence_l, _) = render_offline(&mut deck, 50, 32);
    assert_eq!(deck.position_frames, 100);
    assert!(silence_l.iter().all(|&s| s == 0.0));

    // Play_pause retoma
    deck.play_pause();
    assert_eq!(deck.state, TransportState::Playing);
}

#[test]
fn cue_logica_cdj_completa() {
    let mut deck = Deck::new();
    let audio = sample_audio();
    deck.load_audio(audio, Some(0));

    // 1. Tocar até frame 500
    deck.play();
    let _ = render_offline(&mut deck, 500, 64);
    assert_eq!(deck.position_frames, 500);

    // 2. CUE tocando -> para e volta ao cue ativo (0)
    deck.cue_down();
    assert_eq!(deck.state, TransportState::CueStandby);
    assert_eq!(deck.position_frames, 0);

    // 3. Mover agulha para 1000 e pausar lá
    deck.seek_frame(1000);
    assert_eq!(deck.position_frames, 1000);
    assert_eq!(deck.state, TransportState::Stopped);

    // 4. CUE pausado fora do cue ativo -> define novo ponto de CUE
    deck.cue_down();
    assert_eq!(deck.state, TransportState::CueStandby);
    assert_eq!(deck.cue_frame, 1000);
    assert_eq!(deck.position_frames, 1000);

    // 5. Cue down em CueStandby -> entra em CuePreview
    deck.cue_down();
    assert_eq!(deck.state, TransportState::CuePreview);
    let _ = render_offline(&mut deck, 200, 64);
    assert_eq!(deck.position_frames, 1200);

    // 6. Cue up em CuePreview -> volta ao CUE e entra em CueStandby
    deck.cue_up();
    assert_eq!(deck.state, TransportState::CueStandby);
    assert_eq!(deck.position_frames, 1000);

    // 7. Play durante CuePreview -> transita para Playing contínuo
    deck.cue_down();
    assert_eq!(deck.state, TransportState::CuePreview);
    deck.play();
    assert_eq!(deck.state, TransportState::Playing);
    deck.cue_up(); // Soltar CUE não deve voltar se já deu Play
    assert_eq!(deck.state, TransportState::Playing);
}

#[test]
fn seek_e_limites_da_faixa() {
    let mut deck = Deck::new();
    let audio = sample_audio();
    let total = audio.total_frames;
    deck.load_audio(audio, Some(0));

    // Seek para além do fim limita ao total
    deck.seek_frame(total + 10000);
    assert_eq!(deck.position_frames, total);

    // Seek para o início
    deck.seek_frame(0);
    assert_eq!(deck.position_frames, 0);

    // Seek por segundos
    deck.seek_seconds(1.0);
    assert_eq!(deck.position_frames, 44100);
}

#[test]
fn loop_volta_ao_inicio_automaticamente() {
    let mut deck = Deck::new();
    let audio = sample_audio();
    deck.load_audio(audio, Some(0));

    // Definir loop entre 100 e 200
    deck.set_loop(100, 200);
    deck.seek_frame(100);
    deck.play();

    // Renderizar 150 frames: deve chegar a 200, voltar para 100 e avançar até 150
    let _ = render_offline(&mut deck, 150, 32);
    assert_eq!(deck.position_frames, 150);
}

#[test]
fn fim_da_faixa_cessa_reproducao() {
    let mut deck = Deck::new();
    let audio = Arc::new(AudioBuffer::synthetic_sine(44100, 440.0, 0.01)); // ~441 frames
    let total = audio.total_frames;
    deck.load_audio(audio, Some(0));
    deck.play();

    let _ = render_offline(&mut deck, (total as usize) + 500, 64);
    assert_eq!(deck.position_frames, total);
    assert_eq!(deck.state, TransportState::Stopped);
}
