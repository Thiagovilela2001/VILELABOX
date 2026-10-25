//! SLIP em sombra durante loop automático (spec/rekordbox/30-deck.md,
//! dossiê `evidence/p6-6-slip-sombra.md`). Rodadas da captura: loop de 1
//! batida (~0,46 s) aberto em 13,31 s, 7 s tocando dentro dele, saída.
use std::sync::Arc;
use vilelabox_audio::{AudioBuffer, Deck};

const SR: u64 = 44_100;
const BATIDA: u64 = 20_350; // ~0,46 s, 1 batida a ~130 BPM

fn rodada(slip: bool) -> f64 {
    let mut d = Deck::new();
    d.load_audio(Arc::new(AudioBuffer::silence(44_100, SR * 60)), None);
    d.set_slip(slip);
    d.play();
    d.seek_frame(586_794); // 13,31 s, início do loop na captura
    d.auto_loop(BATIDA);
    let (mut l, mut r) = (vec![0.0; 441], vec![0.0; 441]);
    for _ in 0..700 {
        d.process_block(&mut l, &mut r); // 700 x 10 ms = 7 s
    }
    assert!(d.position_frames < 586_794 + BATIDA, "dentro do loop");
    d.clear_loop();
    d.position_frames as f64 / SR as f64
}

#[test]
fn sem_slip_sai_de_onde_o_loop_estava() {
    let t = rodada(false);
    assert!((13.31..13.31 + 0.47).contains(&t), "saiu em {t}");
}

#[test]
fn com_slip_sai_onde_a_musica_estaria() {
    let t = rodada(true);
    assert!((t - (13.306 + 7.0)).abs() < 0.01, "saiu em {t}");
}

#[test]
fn desligar_slip_no_loop_descarta_a_sombra() {
    let mut d = Deck::new();
    d.load_audio(Arc::new(AudioBuffer::silence(44_100, SR * 60)), None);
    d.set_slip(true);
    d.play();
    d.auto_loop(BATIDA);
    let (mut l, mut r) = (vec![0.0; 441], vec![0.0; 441]);
    for _ in 0..200 {
        d.process_block(&mut l, &mut r);
    }
    d.set_slip(false);
    d.clear_loop();
    assert!(d.position_frames < BATIDA);
}
