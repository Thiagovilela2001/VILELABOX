//! Casos 1 a 12 de spec/rekordbox/32-loops-cues.md no fluxo do app: faixa
//! carregada pelo `AppState`, botão pela ação de `acoes` (a mesma que a tela
//! chama), comando aplicado ao `AudioMixer` real e estado lido do motor.

use crate::acoes;
use crate::library::Track;
use crate::state::{AppState, Command, DeckId, DeckState};
use std::sync::Arc;
use vilelabox_anlz::{Beat, Grid};
use vilelabox_audio::{AudioBuffer, AudioCommand, AudioMixer, Deck, TransportState};

const SR: f64 = 44_100.0;

/// Batidas a 120 BPM (uma a cada 0,5 s) a partir de 1,0 s: 1,0 / 1,5 / 2,0 ...
fn grade_120() -> Grid {
    let beats = (0..200)
        .map(|k| Beat {
            beat_number: (k % 4) as u16 + 1,
            bpm: 120.0,
            time_ms: 1000 + k * 500,
        })
        .collect();
    Grid {
        bpm: 120.0,
        variable: false,
        first_beat_time_s: 1.0,
        downbeat_index: 0,
        beats,
    }
}

struct App {
    st: AppState,
    motor: AudioMixer,
    dir: std::path::PathBuf,
}

impl App {
    /// Faixas de 120 s em silêncio carregadas nos dois decks, cues em pasta temporária.
    fn new(nome: &str, grid: Option<Grid>) -> Self {
        let dir = std::env::temp_dir().join(format!("vilelabox-e2e-{nome}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut st = AppState::new();
        st.cues_dir = Some(dir.clone());
        st.apply(Command::ScanFinished {
            // Uma faixa por deck: cada uma tem o próprio arquivo de cues.
            tracks: ["1", "2"]
                .map(|d| Track {
                    title: format!("A - Faixa {d}.mp3"),
                    dat: None,
                    path: Some(format!("/musica/{nome}-{d}.mp3")),
                    meta: None,
                    ext: None,
                    grid: grid.clone(),
                })
                .into(),
            summary: String::new(),
        });
        let mut app = Self {
            st,
            motor: AudioMixer::offline(),
            dir,
        };
        app.carregar(DeckId::Deck1);
        app.carregar(DeckId::Deck2);
        app
    }

    fn carregar(&mut self, id: DeckId) {
        let filtered_index = if id == DeckId::Deck1 { 0 } else { 1 };
        self.st.apply(Command::LoadToDeck {
            deck: id,
            filtered_index,
        });
        let buf = Arc::new(AudioBuffer::silence(44_100, 120 * 44_100));
        let cmd = acoes::carregar_audio(&mut self.st, id, buf);
        self.motor.apply(cmd);
    }

    fn deck(&self, i: usize) -> &Deck {
        if i == 0 {
            &self.motor.deck1
        } else {
            &self.motor.deck2
        }
    }

    fn pos(&self, i: usize) -> f64 {
        self.deck(i).snapshot().position_seconds
    }

    fn enviar(&mut self, cmd: Option<AudioCommand>) {
        if let Some(c) = cmd {
            self.motor.apply(c);
        }
    }

    fn ir(&mut self, i: usize, s: f64) {
        self.motor.apply(AudioCommand::SeekSeconds {
            deck: i as u8 + 1,
            seconds: s,
        });
    }

    fn tocar(&mut self, i: usize) {
        self.motor.apply(AudioCommand::Play { deck: i as u8 + 1 });
    }

    fn loop_alternar(&mut self, i: usize) {
        let pos = self.pos(i);
        let c = acoes::loop_alternar(&mut self.st, i, pos);
        self.enviar(c);
    }

    fn memory(&mut self, i: usize) {
        let pos = self.pos(i);
        acoes::memory(&mut self.st, i, pos);
    }

    fn loop_cut(&mut self, i: usize, dobro: bool) {
        let c = acoes::loop_cut(&mut self.st, i, dobro);
        self.enviar(c);
    }

    fn quantize(&mut self, i: usize) {
        let c = acoes::quantize(&mut self.st, i);
        self.motor.apply(c);
    }

    fn cue_linha(&mut self, i: usize, linha: usize) {
        let pos = self.pos(i);
        let c = acoes::cue_linha(&mut self.st, i, linha, pos);
        self.enviar(c);
    }

    /// (início, tamanho) do loop no motor, em frames.
    fn loop_motor(&self, i: usize) -> (u64, u64) {
        let (a, b) = self.deck(i).loop_range.expect("loop no motor");
        (a, b - a)
    }

    /// Loop aberto com a cabeça em `s`; devolve o início no motor.
    fn inicio_loop_em(&mut self, s: f64) -> u64 {
        self.ir(0, s);
        self.loop_alternar(0);
        let (a, _) = self.loop_motor(0);
        self.loop_alternar(0);
        a
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Batidas `n` em frames, na grade de 120 BPM.
fn batidas(n: f64) -> u64 {
    (n * 0.5 * SR).round() as u64
}

#[test]
fn caso_1_loop_tocando_x2_duas_vezes_mantem_inicio() {
    let mut app = App::new("c1", Some(grade_120()));
    app.ir(0, 37.41);
    app.tocar(0);
    app.st.loop_batidas[0] = 4.0;
    app.loop_alternar(0);
    let inicio = (37.41 * SR).round() as u64;
    assert_eq!(app.loop_motor(0), (inicio, batidas(4.0)));
    app.loop_cut(0, true);
    assert_eq!(app.loop_motor(0), (inicio, batidas(8.0)));
    app.loop_cut(0, true);
    assert_eq!(app.loop_motor(0), (inicio, batidas(16.0)));
    assert_eq!(app.st.loop_batidas[0], 16.0);
    assert_eq!(app.deck(0).state, TransportState::Playing);
}

#[test]
fn caso_2_loop_8_metade_quatro_vezes() {
    let mut app = App::new("c2", Some(grade_120()));
    app.ir(0, 10.0);
    app.st.loop_batidas[0] = 8.0;
    app.loop_alternar(0);
    let (inicio, _) = app.loop_motor(0);
    for esperado in [4.0, 2.0, 1.0, 0.5] {
        app.loop_cut(0, false);
        assert_eq!(app.st.loop_batidas[0], esperado);
        assert_eq!(app.loop_motor(0), (inicio, batidas(esperado)));
    }
}

#[test]
fn caso_3_hot_cue_a_volta_e_toca_do_ponto() {
    let mut app = App::new("c3", Some(grade_120()));
    app.ir(1, 10.0);
    app.cue_linha(1, 0); // A vazio: grava
    app.ir(1, 30.0); // deck parado em outro lugar
    assert_eq!(app.deck(1).state, TransportState::Stopped);
    app.cue_linha(1, 0);
    assert_eq!(app.deck(1).position_frames, (10.0 * SR) as u64);
    assert_eq!(app.deck(1).state, TransportState::Playing);
}

#[test]
fn caso_4_memory_cue_tocando_chamar_para_em_espera() {
    let mut app = App::new("c4", Some(grade_120()));
    app.ir(0, 0.025);
    app.memory(0);
    app.ir(0, 40.0);
    app.tocar(0);
    app.cue_linha(0, 0);
    let d = app.deck(0);
    assert_eq!(d.state, TransportState::CueStandby);
    assert_eq!(d.position_frames, d.cue_frame);
    assert!((d.snapshot().position_seconds - 0.025).abs() <= 1.0 / SR);
}

#[test]
fn caso_5_q_nao_muda_o_tamanho_do_loop() {
    let mut app = App::new("c5", Some(grade_120()));
    app.st.loop_batidas[0] = 16.0;
    app.ir(0, 1.7);
    app.loop_alternar(0);
    let (_, sem_q) = app.loop_motor(0);
    app.loop_alternar(0);
    app.quantize(0);
    assert!(app.deck(0).quantize);
    app.loop_alternar(0);
    let (_, com_q) = app.loop_motor(0);
    assert_eq!((sem_q, com_q), (batidas(16.0), batidas(16.0)));
}

#[test]
fn caso_6_au_ma_so_troca_o_estado_da_caixa() {
    let mut app = App::new("c6", Some(grade_120()));
    app.ir(0, 5.0);
    app.loop_alternar(0);
    let antes = (
        app.deck(0).clone().snapshot(),
        app.st.loop_batidas,
        app.st.loop_ativo,
    );
    acoes::loop_modo(&mut app.st, 0); // AU -> MA, sem comando de motor
    assert!(app.st.loop_manual[0]);
    acoes::loop_modo(&mut app.st, 0);
    assert!(!app.st.loop_manual[0]);
    assert_eq!(
        (
            app.deck(0).snapshot(),
            app.st.loop_batidas,
            app.st.loop_ativo
        ),
        antes
    );
}

#[test]
fn caso_7_memory_grava_linha_sem_transporte() {
    let mut app = App::new("c7", Some(grade_120()));
    app.ir(0, 51.718);
    let antes = app.deck(0).snapshot();
    app.memory(0);
    let linhas = crate::marks::lista_memoria(&app.st.deck1.cues);
    assert_eq!(linhas.len(), 1);
    assert!((app.st.deck1.cues[0].start_time_s - 51.718).abs() <= 1.0 / SR);
    assert_eq!(app.deck(0).snapshot(), antes);
}

#[test]
fn casos_8_9_10_quantize_do_inicio_do_loop() {
    let mut app = App::new("c8", Some(grade_120()));
    app.quantize(0);
    assert_eq!(app.inicio_loop_em(1.7), 66_150); // 8
    assert_eq!(app.inicio_loop_em(1.999), 66_150); // 8: nunca 2,0 s
    assert_eq!(app.inicio_loop_em(1.5), 66_150); // 9
    app.quantize(0);
    assert_eq!(app.inicio_loop_em(1.7), 74_970); // 10
}

#[test]
fn caso_11_hot_cue_quantizado_com_q() {
    let mut app = App::new("c11", Some(grade_120()));
    app.quantize(1);
    app.ir(1, 1.7);
    app.cue_linha(1, 0); // A com Q: 1,5 s
    app.quantize(1);
    app.cue_linha(1, 1); // B sem Q: 1,7 s
    app.ir(1, 30.0);
    app.cue_linha(1, 0);
    assert_eq!(app.deck(1).position_frames, 66_150);
    app.cue_linha(1, 1);
    assert_eq!(app.deck(1).position_frames, 74_970);
}

#[test]
fn caso_12_sem_no_anterior_posicao_intacta() {
    // Sem grade: o loop automático não abre (não há batida); o hot cue fica em 1,7 s.
    let mut app = App::new("c12a", None);
    app.quantize(1);
    app.ir(1, 1.7);
    app.cue_linha(1, 0);
    assert_eq!(app.st.deck2.cues[0].start_time_s, 1.7);
    app.quantize(0);
    app.ir(0, 1.7);
    app.loop_alternar(0);
    assert!(app.deck(0).loop_range.is_none());
    assert!(!app.st.loop_ativo[0]);
    // Primeira batida em 1,0 s, posição 0,3 s: início em 0,3 s.
    let mut app = App::new("c12b", Some(grade_120()));
    app.quantize(0);
    assert_eq!(app.inicio_loop_em(0.3), 13_230);
}

#[test]
fn persistencia_com_loop_e_q_volta_ao_recarregar() {
    let mut app = App::new("persist", Some(grade_120()));
    // Deck 2: Q ligado, hot cue A em 1,7 s (grava 1,5 s) e loop aberto.
    app.quantize(1);
    app.ir(1, 1.7);
    app.cue_linha(1, 0);
    app.loop_alternar(1);
    assert!(app.st.loop_ativo[1]);
    // Deck 1: memory cue em 3,25 s.
    app.ir(0, 3.25);
    app.memory(0);
    // Recarrega a mesma faixa nos dois decks.
    app.st.deck1 = DeckState::empty();
    app.st.deck2 = DeckState::empty();
    app.carregar(DeckId::Deck1);
    app.carregar(DeckId::Deck2);
    assert!(!app.st.loop_ativo[1]);
    assert!(app.deck(1).loop_range.is_none());
    assert!(app.st.quantize[1]); // Q é do deck, não da faixa
    let hot: Vec<_> = app
        .st
        .deck2
        .cues
        .iter()
        .map(|c| (c.hot_cue, c.start_time_s))
        .collect();
    assert_eq!(hot, [(1, 1.5)]);
    let mem: Vec<_> = app
        .st
        .deck1
        .cues
        .iter()
        .map(|c| (c.hot_cue, c.start_time_s))
        .collect();
    assert_eq!(mem, [(0, 3.25)]);
    // O cue recarregado leva ao mesmo ponto no motor.
    app.ir(1, 30.0);
    app.cue_linha(1, 0);
    assert_eq!(app.deck(1).position_frames, 66_150);
    app.cue_linha(0, 0);
    assert_eq!(app.deck(0).position_frames, (3.25 * SR) as u64);
    assert_eq!(app.deck(0).state, TransportState::CueStandby);
}

#[test]
fn loop_manual_in_out_exit_reloop() {
    // Captura p6-5: IN em 3,20 s, OUT em 7,42 s, EXIT, tocando até 12 s, RELOOP.
    let mut app = App::new("manual", Some(grade_120()));
    app.tocar(0);
    app.ir(0, 3.2);
    let c = acoes::loop_in(&mut app.st, 0, 3.2);
    app.enviar(c);
    assert_eq!(app.deck(0).cue_frame, (3.2 * SR) as u64); // IN vira o ponto de cue
    assert_eq!(app.deck(0).state, TransportState::Playing); // transporte segue
    app.ir(0, 7.42);
    let c = acoes::loop_out(&mut app.st, 0, 7.42);
    app.enviar(c);
    let faixa = Some(((3.2 * SR).round() as u64, (7.42 * SR).round() as u64));
    assert_eq!(app.deck(0).loop_range, faixa);
    assert!(app.deck(0).loop_active);
    // EXIT: encerra, a reprodução segue de onde está.
    for c in acoes::loop_exit_reloop(&mut app.st, 0) {
        app.motor.apply(c);
    }
    assert!(!app.deck(0).loop_active);
    assert_eq!(app.deck(0).state, TransportState::Playing);
    // RELOOP depois do OUT: volta ao IN e repete o mesmo trecho.
    app.ir(0, 12.0);
    for c in acoes::loop_exit_reloop(&mut app.st, 0) {
        app.motor.apply(c);
    }
    assert_eq!(app.deck(0).position_frames, (3.2 * SR).round() as u64);
    assert_eq!(app.deck(0).loop_range, faixa);
    assert!(app.deck(0).loop_active);
    assert_eq!(app.deck(0).state, TransportState::Playing);
}
