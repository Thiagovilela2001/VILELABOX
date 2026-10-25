//! Casos de `spec/rekordbox/30-deck.md` e `32-loops-cues.md` para MT, SLIP,
//! BEAT SYNC, KEY SYNC e Q (rekordbox 7.2.8, dossiê
//! `evidence/p6-2-sync-slip-quantize.md`).
use std::sync::Arc;
use vilelabox_audio::{AudioBuffer, AudioCommand, AudioMixer, Deck};

fn deck() -> Deck {
    let mut d = Deck::new();
    d.load_audio(Arc::new(AudioBuffer::silence(44_100, 44_100 * 60)), None);
    d
}

#[test]
fn beat_sync_no_master_nao_troca_andamento() {
    let mut d = deck();
    d.set_beat_sync(true, 130.0, None);
    assert!(d.beat_sync);
    assert_eq!(d.speed_ratio, 1.0);
}

#[test]
fn beat_sync_nao_master_assume_andamento_do_master() {
    let mut d = deck();
    d.set_beat_sync(true, 130.0, Some(130.14));
    let bpm = 130.0 * d.speed_ratio;
    assert!((bpm - 130.14).abs() < 1e-9);
    // +0,1 % na tela
    assert_eq!(format!("{:+.1}", (d.speed_ratio - 1.0) * 100.0), "+0.1");
}

#[test]
fn alinhamento_de_fase_usa_posicoes_atuais_do_motor() {
    let mut mixer = AudioMixer::offline();
    let audio = Arc::new(AudioBuffer::silence(1_000, 5_000));
    mixer.deck1.load_audio(Arc::clone(&audio), None);
    mixer.deck2.load_audio(audio, None);
    mixer.deck1.seek_seconds(1.625);
    mixer.deck2.seek_seconds(1.95);
    mixer.apply(AudioCommand::AlignBeatPhase {
        deck: 2,
        master: 1,
        master_beats: vec![(1_000, 1), (1_500, 2), (2_000, 3), (2_500, 4), (3_000, 1)],
        slave_beats: vec![(800, 1), (1_400, 2), (2_000, 3), (2_600, 4), (3_200, 1)],
    });
    assert!((mixer.deck2.snapshot().position_seconds - 1.55).abs() < 0.001);
}

#[test]
fn key_sync_guarda_deslocamento() {
    let mut d = deck();
    d.set_key_shift(-2); // Gm ±0 -> Fm -2
    assert_eq!(d.snapshot().key_shift, -2);
}

#[test]
fn q_e_so_estado_e_nao_muda_o_loop() {
    // 0x10010 = 16/1: numerador nos 16 bits baixos, denominador nos altos (p6-1).
    let code: u32 = 0x10010;
    let batidas = u64::from(code & 0xffff) / u64::from(code >> 16);
    assert_eq!(batidas, 16);
    let len = batidas * 20_000;

    let mut sem_q = deck();
    sem_q.seek_frame(1_649_781);
    sem_q.auto_loop(len);

    let mut com_q = deck();
    com_q.set_quantize(true);
    com_q.seek_frame(1_649_781);
    com_q.auto_loop(len);

    assert!(com_q.quantize && !sem_q.quantize);
    assert_eq!(com_q.loop_range, sem_q.loop_range);
}

#[test]
fn auto_loop_at_usa_inicio_quantizado_pelo_app() {
    // 120 BPM a 44,1 kHz: batida = 22 050 frames. Posição 1,7 s = 74 970;
    // o app quantiza para a batida de 1,5 s = 66 150 (regra para baixo, p6-2).
    let len = 4 * 22_050;
    let mut d = deck();
    d.set_quantize(true);
    d.seek_frame(74_970);
    d.auto_loop_at(66_150, len);
    assert_eq!(d.loop_range, Some((66_150, 66_150 + len)));

    // Q desligado: o app não passa início e o loop sai da posição exata.
    let mut sem_q = deck();
    sem_q.seek_frame(74_970);
    sem_q.auto_loop(len);
    assert_eq!(sem_q.loop_range, Some((74_970, 74_970 + len)));
}

#[test]
fn mt_e_slip_alternam_estado() {
    let mut d = deck();
    assert!(d.master_tempo);
    d.set_master_tempo(false);
    d.set_slip(true);
    let s = d.snapshot();
    assert!(!s.master_tempo && s.slip);
}
