use std::sync::Arc;
use vilelabox_audio::{render_offline, AudioBuffer, Deck};

#[test]
fn render_determinismo_em_blocos_diferentes() {
    let audio = Arc::new(AudioBuffer::synthetic_sine(44100, 440.0, 1.0));
    let n_frames = 2048;

    let mut deck64 = Deck::new();
    deck64.load_audio(Arc::clone(&audio), Some(0));
    deck64.play();
    let (l64, r64) = render_offline(&mut deck64, n_frames, 64);

    let mut deck128 = Deck::new();
    deck128.load_audio(Arc::clone(&audio), Some(0));
    deck128.play();
    let (l128, r128) = render_offline(&mut deck128, n_frames, 128);

    let mut deck512 = Deck::new();
    deck512.load_audio(Arc::clone(&audio), Some(0));
    deck512.play();
    let (l512, r512) = render_offline(&mut deck512, n_frames, 512);

    let mut deck1024 = Deck::new();
    deck1024.load_audio(Arc::clone(&audio), Some(0));
    deck1024.play();
    let (l1024, r1024) = render_offline(&mut deck1024, n_frames, 1024);

    assert_eq!(l64, l128);
    assert_eq!(l128, l512);
    assert_eq!(l512, l1024);

    assert_eq!(r64, r128);
    assert_eq!(r128, r512);
    assert_eq!(r512, r1024);

    assert_eq!(deck64.position_frames, n_frames as u64);
    assert_eq!(deck1024.position_frames, n_frames as u64);
}

#[test]
fn render_amostra_a_amostra_bate_com_audio_buffer() {
    let audio = Arc::new(AudioBuffer::synthetic_sine(44100, 440.0, 0.5));
    let mut deck = Deck::new();
    deck.load_audio(Arc::clone(&audio), Some(0));
    deck.play();

    let (l, r) = render_offline(&mut deck, 500, 64);
    for i in 0..500 {
        let (expected_l, expected_r) = audio.get_frame(i as u64).unwrap();
        assert_eq!(l[i], expected_l);
        assert_eq!(r[i], expected_r);
    }
}
