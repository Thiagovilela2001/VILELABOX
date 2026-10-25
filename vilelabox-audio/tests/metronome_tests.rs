use std::sync::Arc;

use vilelabox_audio::{AudioBuffer, Deck};

#[test]
fn metronome_generates_clicks_on_grid_and_can_be_disabled() {
    let mut deck = Deck::new();
    deck.load_audio(Arc::new(AudioBuffer::silence(44_100, 44_100)), None);
    deck.set_metronome(3, vec![(0, 1), (500, 2)]);
    deck.play();

    let mut left = vec![0.0; 2_048];
    let mut right = vec![0.0; 2_048];
    deck.process_block(&mut left, &mut right);
    assert!(left.iter().any(|sample| sample.abs() > 0.001));
    assert_eq!(left, right);

    deck.seek_frame(0);
    deck.set_metronome(0, vec![(0, 1)]);
    left.fill(0.0);
    right.fill(0.0);
    deck.process_block(&mut left, &mut right);
    assert!(left.iter().all(|sample| *sample == 0.0));
}
