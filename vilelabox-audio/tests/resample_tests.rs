use std::sync::Arc;
use vilelabox_audio::{AudioBuffer, Deck};

/// Faixa de 44,1 kHz numa saída de 48 kHz: 1 s de saída avança 1 s da faixa.
#[test]
fn saida_48k_avanca_no_tempo_da_faixa() {
    let mut deck = Deck::new();
    deck.out_rate = 48_000;
    deck.load_audio(Arc::new(AudioBuffer::synthetic_sine(44_100, 440.0, 2.0)), None);
    deck.play();
    let (mut l, mut r) = (vec![0.0f32; 480], vec![0.0f32; 480]);
    for _ in 0..100 {
        deck.process_block(&mut l, &mut r);
    }
    let pos = deck.position_frames as i64;
    assert!((pos - 44_100).abs() <= 1, "posição {pos}");
}
