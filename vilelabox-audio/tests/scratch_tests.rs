use std::sync::Arc;

use vilelabox_audio::{AudioBuffer, Deck, TransportState};

fn ramp() -> Arc<AudioBuffer> {
    let mut samples = Vec::with_capacity(4_000);
    for i in 0..2_000 {
        let value = i as f32 / 2_000.0;
        samples.extend([value, value]);
    }
    Arc::new(AudioBuffer::new(1_000, 2, samples))
}

#[test]
fn arrasto_para_tras_reproduz_buffer_em_ordem_reversa() {
    let mut deck = Deck::new();
    deck.load_audio(ramp(), None);
    deck.seek_frame(1_500);
    deck.scratch_begin();
    deck.scratch_to(0.5);

    let (mut left, mut right) = (vec![0.0; 100], vec![0.0; 100]);
    deck.process_block(&mut left, &mut right);

    assert!(left[1] < left[0], "o áudio deve caminhar para trás");
    assert!((deck.position_frames as i64 - 500).abs() <= 1);
    assert_eq!(left, right);

    deck.scratch_end();
    assert_eq!(deck.state, TransportState::Stopped);
    deck.process_block(&mut left, &mut right);
    assert!(left.iter().all(|sample| *sample == 0.0));
}

#[test]
fn soltar_restaura_reproducao_que_estava_ativa() {
    let mut deck = Deck::new();
    deck.load_audio(ramp(), None);
    deck.seek_frame(1_000);
    deck.play();
    deck.scratch_begin();
    deck.scratch_to(0.8);
    let (mut left, mut right) = (vec![0.0; 20], vec![0.0; 20]);
    deck.process_block(&mut left, &mut right);
    deck.scratch_end();

    assert_eq!(deck.state, TransportState::Playing);
    let before = deck.position_frames;
    deck.process_block(&mut left, &mut right);
    assert!(deck.position_frames > before);
}
