//! Aplicativo desktop do VILELABOX em Rust (Slint).
//!
//! Implementa a arquitetura de comandos e snapshots (U0) do `PLANO-UI-FUNCOES.md`.
//! A thread da interface abre imediatamente sem bloquear no scan da biblioteca (U0.5).

mod acoes;
mod beatgrid;
mod cues_store;
mod grid_store;
mod library;
mod loops;
#[cfg(test)]
mod loops_cues_e2e;
mod marks;
mod presentation;
pub mod state;
mod sync;
mod waveform;

use crate::beatgrid::{EditRange, GridAction};
use crate::state::{AppState, Command, DeckId, SortColumn};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use vilelabox_audio::{start_audio_engine, AudioCommand, AudioController};

slint::include_modules!();

fn decode_percent_encoded(s: &str) -> String {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) =
                u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
            {
                result.push(byte);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&result).into_owned()
}

fn search_dir_recursive(dir: &std::path::Path, target_name: &str, depth: usize) -> Option<PathBuf> {
    let direct = dir.join(target_name);
    if direct.is_file() {
        return Some(direct);
    }
    search_dir_match(dir, depth, &|name| name.eq_ignore_ascii_case(target_name))
}

fn search_dir_match(
    dir: &std::path::Path,
    depth: usize,
    ok: &dyn Fn(&str) -> bool,
) -> Option<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return None;
    };
    let mut subdirs = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if path.file_name().and_then(|n| n.to_str()).is_some_and(ok) {
                return Some(path);
            }
        } else if depth > 0 && path.is_dir() {
            subdirs.push(path);
        }
    }
    if depth > 0 {
        for subdir in subdirs {
            if let Some(found) = search_dir_match(&subdir, depth - 1, ok) {
                return Some(found);
            }
        }
    }
    None
}

const AUDIO_EXTS: [&str; 7] = ["mp3", "wav", "flac", "m4a", "aac", "aiff", "aif"];

/// Nome comparável: sem extensão de áudio, sem numeração inicial ("01 - ",
/// "01. ") e em minúsculas. `01 - X.mp3` casa com `X.wav`.
fn nome_normalizado(name: &str) -> Option<String> {
    let (stem, ext) = name.rsplit_once('.')?;
    if !AUDIO_EXTS.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
        return None;
    }
    let s = stem.trim_start_matches(|c: char| c.is_ascii_digit() || " .-_".contains(c));
    (!s.is_empty()).then(|| s.to_lowercase())
}

fn find_audio_file(raw_path: Option<&str>, title: &str) -> Option<PathBuf> {
    // 1. Limpar esquemas de URL (file://localhost/ ou file:///)
    if let Some(raw) = raw_path {
        let clean = if let Some(stripped) = raw.strip_prefix("file://localhost/") {
            decode_percent_encoded(stripped)
        } else if let Some(stripped) = raw.strip_prefix("file:///") {
            decode_percent_encoded(stripped)
        } else if let Some(stripped) = raw.strip_prefix("file://") {
            decode_percent_encoded(stripped)
        } else {
            raw.to_string()
        };

        let p = std::path::Path::new(&clean);
        if p.is_file() {
            return Some(p.to_path_buf());
        }

        // Tentar sem prefixo ?/ ou ?\
        let stripped = clean.trim_start_matches(['?', '/', '\\']);
        let p2 = std::path::Path::new(stripped);
        if p2.is_file() {
            return Some(p2.to_path_buf());
        }
    }

    // 2. Extrair o nome do arquivo a partir de raw_path ou de title
    let candidate_names: Vec<String> = {
        let mut list = Vec::new();
        if let Some(raw) = raw_path {
            let clean = if let Some(stripped) = raw.strip_prefix("file://localhost/") {
                decode_percent_encoded(stripped)
            } else if let Some(stripped) = raw.strip_prefix("file:///") {
                decode_percent_encoded(stripped)
            } else {
                raw.to_string()
            };
            let name = clean
                .rsplit(['/', '\\'])
                .next()
                .unwrap_or(&clean)
                .trim_start_matches(['?', '/', '\\']);
            if !name.is_empty() {
                list.push(name.to_string());
            }
        }
        if !title.is_empty() {
            let name = title.rsplit(['/', '\\']).next().unwrap_or(title);
            if !list
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(name))
            {
                list.push(name.to_string());
            }
        }
        list
    };

    // 3. Pastas onde procurar (Música do usuário, Downloads, pasta atual)
    let mut search_dirs: Vec<PathBuf> = Vec::new();
    if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        let base = PathBuf::from(user_profile);
        search_dirs.push(base.join("Music"));
        search_dirs.push(base.join("Músicas"));
        search_dirs.push(base.join("Downloads"));
        search_dirs.push(base.join("Desktop"));
    }
    if let Ok(cur) = std::env::current_dir() {
        search_dirs.push(cur.clone());
        search_dirs.push(cur.join("tests"));
    }

    // Procurar os nomes de arquivo candidatos nas pastas de busca
    for target in &candidate_names {
        for base_dir in &search_dirs {
            if let Some(found) = search_dir_recursive(base_dir, target, 3) {
                return Some(found);
            }
        }
        // Se o nome não tiver extensão (.mp3/.wav/etc.), tentar com extensões comuns
        if !target.contains('.') {
            for ext in &AUDIO_EXTS {
                let target_with_ext = format!("{target}.{ext}");
                for base_dir in &search_dirs {
                    if let Some(found) = search_dir_recursive(base_dir, &target_with_ext, 3) {
                        return Some(found);
                    }
                }
            }
        }
    }

    // Última tentativa: arquivo renomeado (sem a numeração) ou convertido.
    for target in &candidate_names {
        let Some(alvo) = nome_normalizado(target) else {
            continue;
        };
        let ok = |name: &str| nome_normalizado(name).as_deref() == Some(alvo.as_str());
        for base_dir in &search_dirs {
            if let Some(found) = search_dir_match(base_dir, 3, &ok) {
                return Some(found);
            }
        }
    }

    None
}

fn resolve_audio_buffer(
    path: Option<&str>,
    title: &str,
) -> Result<Arc<vilelabox_audio::AudioBuffer>, String> {
    // Sem substituto sintético: tocar um WAV de teste no lugar da faixa soava
    // como "só estalos" e escondia o arquivo ausente.
    let found =
        find_audio_file(path, title).ok_or_else(|| format!("Áudio não encontrado: {title}"))?;
    vilelabox_audio::decode_audio(&found)
        .map(Arc::new)
        .map_err(|e| format!("Falha ao decodificar {}: {e}", found.display()))
}

/// Índice 0/1 do deck 1/2 vindo da UI.
fn deck_idx(n: i32) -> Option<usize> {
    match n {
        1 => Some(0),
        2 => Some(1),
        _ => None,
    }
}

/// Posição do deck `i` (0/1) no motor, em segundos.
fn pos_s(ctrl: &AudioController, i: usize) -> f64 {
    let snap = ctrl.get_snapshot();
    if i == 0 {
        snap.deck1.position_seconds
    } else {
        snap.deck2.position_seconds
    }
}

/// Enfileira o alinhamento no motor. A posição é calculada quando o callback
/// consome o comando, evitando usar um snapshot que já ficou para trás.
fn send_phase_alignment(ctrl: &AudioController, st: &AppState, slave_i: usize) -> bool {
    if slave_i == st.master {
        return false;
    }
    let master = if st.master == 0 { &st.deck1 } else { &st.deck2 };
    let slave = if slave_i == 0 { &st.deck1 } else { &st.deck2 };
    if master.beats.len() < 2 || slave.beats.len() < 2 {
        return false;
    }
    ctrl.send(AudioCommand::AlignBeatPhase {
        deck: (slave_i + 1) as u8,
        master: (st.master + 1) as u8,
        master_beats: master
            .beats
            .iter()
            .map(|b| (b.time_ms, b.beat_number))
            .collect(),
        slave_beats: slave
            .beats
            .iter()
            .map(|b| (b.time_ms, b.beat_number))
            .collect(),
    })
}

fn render_ui(ui: &slint::Weak<AppWindow>, st: &AppState) {
    if let Some(ui) = ui.upgrade() {
        presentation::render(st, &ui);
    }
}

/// Decodifica fora da thread da UI (symphonia leva segundos por faixa) e
/// aplica no event loop; descarta se o deck trocou de faixa nesse meio-tempo.
fn load_audio_async(
    ctrl: &AudioController,
    state: &Arc<Mutex<AppState>>,
    ui: &slint::Weak<AppWindow>,
    st: &AppState,
    deck: DeckId,
    track_idx: usize,
) {
    let Some(track) = st.tracks.get(track_idx) else {
        return;
    };
    let (path, title) = (track.path.clone(), track.title.clone());
    let (ctrl, state, ui) = (ctrl.clone(), state.clone(), ui.clone());
    std::thread::spawn(move || {
        let audio_buf = resolve_audio_buffer(path.as_deref(), &title);
        let _ = slint::invoke_from_event_loop(move || {
            let Ok(mut st) = state.lock() else {
                return;
            };
            let atual = match deck {
                DeckId::Deck1 => st.deck1.loaded_track_idx,
                DeckId::Deck2 => st.deck2.loaded_track_idx,
            };
            if atual != Some(track_idx) {
                return;
            }
            match audio_buf {
                Ok(buf) => {
                    apply_audio(&ctrl, &mut st, deck, buf);
                }
                Err(msg) => {
                    // Silencia o deck (a faixa anterior não segue tocando) e avisa.
                    ctrl.send(AudioCommand::LoadTrack {
                        deck: if deck == DeckId::Deck1 { 1 } else { 2 },
                        audio: Arc::new(vilelabox_audio::AudioBuffer::silence(44100, 1)),
                        cue_frame: Some(0),
                    });
                    st.status_text = msg;
                }
            }
            if let Some(ui) = ui.upgrade() {
                presentation::render(&st, &ui);
            }
        });
    });
}

fn apply_audio(
    controller: &AudioController,
    state: &mut AppState,
    deck: DeckId,
    audio_buf: Arc<vilelabox_audio::AudioBuffer>,
) {
    controller.send(acoes::carregar_audio(state, deck, audio_buf));
    let i = if deck == DeckId::Deck1 { 0 } else { 1 };
    send_metronome(controller, state, i);
}

fn send_metronome(controller: &AudioController, state: &AppState, i: usize) {
    let beats = if i == 0 {
        &state.deck1.beats
    } else {
        &state.deck2.beats
    };
    controller.send(AudioCommand::SetMetronome {
        deck: (i + 1) as u8,
        level: state.grid_edit[i].metronome,
        beats: beats.iter().map(|b| (b.time_ms, b.beat_number)).collect(),
    });
}

/// Pasta onde o rekordbox grava as análises (`.DAT`/`.EXT`). É um caminho de
/// dados de terceiros lido em modo somente leitura — interoperabilidade, não
/// marca: o nome aparece porque é onde os arquivos estão.
fn default_dir() -> PathBuf {
    let appdata = std::env::var_os("APPDATA").unwrap_or_default();
    PathBuf::from(appdata).join(r"Pioneer\rekordbox\share\PIONEER\USBANLZ")
}

fn main() -> Result<(), slint::PlatformError> {
    // Argumento: pasta de análises, um documento XML ou a cópia sem
    // criptografia do catálogo (`.db`, gerada por tools/catalog/exportar_catalogo.py).
    // XML e catálogo dão coleção, tom, cues e listas; grade e formas de onda
    // vêm das análises em `default_dir`.
    let arg = std::env::args_os().nth(1).map(PathBuf::from);
    let xml = arg.clone().filter(|p| {
        p.extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("xml") || e.eq_ignore_ascii_case("db"))
    });
    let dir = match (&xml, arg) {
        (None, Some(d)) => d,
        _ => default_dir(),
    };

    let mut initial_state = AppState::new();
    initial_state.cues_dir = cues_store::dir_padrao();
    initial_state.clock_text = presentation::system_clock_str();
    initial_state.apply(Command::ScanStarted {
        dir_display: dir.display().to_string(),
    });

    let (audio_controller, _stream) =
        start_audio_engine().expect("falha ao inicializar motor de áudio");

    let state = Arc::new(Mutex::new(initial_state));
    let ui = AppWindow::new()?;

    // Sem VLBX_ESCALA, a escala acompanha a largura da janela (binding no Slint).
    if let Ok(raw) = std::env::var("VLBX_ESCALA") {
        ui.set_s(presentation::parse_scale(Some(&raw)));
    }

    // Renderiza snapshot inicial imediatamente (janela abre sem esperar o scan)
    if let Ok(st) = state.lock() {
        presentation::render(&st, &ui);
    }

    // U0.5: Varredura em thread de background com entrega via invoke_from_event_loop
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let scan_dir = dir.clone();
        std::thread::spawn(move || {
            let (tracks, playlists, summary) = match &xml {
                Some(x) => match if x.extension().is_some_and(|e| e.eq_ignore_ascii_case("db")) {
                    library::load_catalogo(x, &scan_dir).map_err(|e| e.to_string())
                } else {
                    std::fs::read_to_string(x)
                        .map_err(|e| e.to_string())
                        .and_then(|s| library::load_xml(&s, &scan_dir).map_err(|e| e.to_string()))
                } {
                    Ok((t, p)) => {
                        let com_analise = t.iter().filter(|t| t.ext.is_some()).count();
                        let s = format!(
                            "{} faixas ({} com análise), {} listas de {}",
                            t.len(),
                            com_analise,
                            p.len(),
                            x.display()
                        );
                        (t, p, s)
                    }
                    Err(e) => (Vec::new(), Vec::new(), format!("Coleção ilegível: {e}")),
                },
                None => {
                    let t = library::scan(&scan_dir);
                    let s = format!("{} faixas em {}", t.len(), scan_dir.display());
                    (t, Vec::new(), s)
                }
            };
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    if let Ok(mut st) = state_ref.lock() {
                        st.apply(Command::PlaylistsLoaded(playlists));
                        st.apply(Command::ScanFinished { tracks, summary });
                        presentation::render(&st, &ui);
                    }
                }
            });
        });
    }

    // Timer do relógio (U3.2) a cada segundo
    let clock_timer = slint::Timer::default();
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        clock_timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_secs(1),
            move || {
                let now_str = presentation::system_clock_str();
                if let Ok(mut st) = state_ref.lock() {
                    if st.clock_text != now_str {
                        st.apply(Command::TickClock(now_str));
                        if let Some(ui) = ui_weak.upgrade() {
                            presentation::render(&st, &ui);
                        }
                    }
                }
            },
        );
    }

    // Timer de sincronização do motor de áudio com a interface (40 FPS)
    let audio_sync_timer = slint::Timer::default();
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let ctrl = audio_controller.clone();
        audio_sync_timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(25),
            move || {
                let snap = ctrl.get_snapshot();
                if let Ok(mut st) = state_ref.lock() {
                    let d1_active = st.deck1.audio_ready
                        && (snap.deck1.is_playing
                            || st.deck1.is_playing
                            || (st.deck1.pos_s - snap.deck1.position_seconds).abs() > 0.05);
                    let d2_active = st.deck2.audio_ready
                        && (snap.deck2.is_playing
                            || st.deck2.is_playing
                            || (st.deck2.pos_s - snap.deck2.position_seconds).abs() > 0.05);
                    if d1_active || d2_active {
                        st.apply(Command::SyncAudioSnapshots {
                            deck1_pos_s: snap.deck1.position_seconds,
                            deck1_playing: snap.deck1.is_playing,
                            deck1_cue: snap.deck1.is_cue_standby,
                            deck2_pos_s: snap.deck2.position_seconds,
                            deck2_playing: snap.deck2.is_playing,
                            deck2_cue: snap.deck2.is_cue_standby,
                        });
                        if let Some(ui) = ui_weak.upgrade() {
                            presentation::render_transport(&st, &ui);
                        }
                    }
                }
            },
        );
    }

    // Callbacks da interface (Command/Snapshot)
    // 1. Seleção de faixa na biblioteca (U1.6: clique apenas seleciona)
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        ui.on_track_selected(move |idx| {
            if let Ok(idx) = usize::try_from(idx) {
                if let Ok(mut st) = state_ref.lock() {
                    st.apply(Command::SelectTrack(idx));
                    if let Some(ui) = ui_weak.upgrade() {
                        presentation::render(&st, &ui);
                    }
                }
            }
        });
    }

    // 1b. Duplo clique em faixa na biblioteca (carregamento inteligente)
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let ctrl_load = audio_controller.clone();
        ui.on_track_double_clicked(move |idx| {
            if let Ok(idx) = usize::try_from(idx) {
                if let Ok(mut st) = state_ref.lock() {
                    st.apply(Command::SelectTrack(idx));
                    let target_deck = if st.deck1.loaded_track_idx.is_none() {
                        DeckId::Deck1
                    } else if st.deck2.loaded_track_idx.is_none()
                        || (st.deck1.is_playing && !st.deck2.is_playing)
                    {
                        DeckId::Deck2
                    } else {
                        DeckId::Deck1
                    };
                    st.apply(Command::LoadToDeck {
                        deck: target_deck,
                        filtered_index: idx,
                    });
                    let track_opt = match target_deck {
                        DeckId::Deck1 => st.deck1.loaded_track_idx,
                        DeckId::Deck2 => st.deck2.loaded_track_idx,
                    };
                    if let Some(t_idx) = track_opt {
                        load_audio_async(&ctrl_load, &state_ref, &ui_weak, &st, target_deck, t_idx);
                    }
                    if let Some(ui) = ui_weak.upgrade() {
                        presentation::render(&st, &ui);
                    }
                }
            }
        });
    }

    // 2. Carregar seleção em um deck (U0.4 / U1.6)
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let ctrl_load = audio_controller.clone();
        ui.on_load_selected_deck(move |deck_num| {
            let deck = match deck_num {
                1 => DeckId::Deck1,
                2 => DeckId::Deck2,
                _ => return,
            };
            if let Ok(mut st) = state_ref.lock() {
                st.apply(Command::LoadSelectedToDeck(deck));
                let track_opt = match deck {
                    DeckId::Deck1 => st.deck1.loaded_track_idx,
                    DeckId::Deck2 => st.deck2.loaded_track_idx,
                };
                if let Some(t_idx) = track_opt {
                    load_audio_async(&ctrl_load, &state_ref, &ui_weak, &st, deck, t_idx);
                }
                if let Some(ui) = ui_weak.upgrade() {
                    presentation::render(&st, &ui);
                }
            }
        });
    }

    // 3. Carregar faixa específica em um deck
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let ctrl_load = audio_controller.clone();
        ui.on_load_deck(move |deck_num, filtered_idx| {
            let deck = match deck_num {
                1 => DeckId::Deck1,
                2 => DeckId::Deck2,
                _ => return,
            };
            if let Ok(filtered_idx) = usize::try_from(filtered_idx) {
                if let Ok(mut st) = state_ref.lock() {
                    st.apply(Command::LoadToDeck {
                        deck,
                        filtered_index: filtered_idx,
                    });
                    let track_opt = match deck {
                        DeckId::Deck1 => st.deck1.loaded_track_idx,
                        DeckId::Deck2 => st.deck2.loaded_track_idx,
                    };
                    if let Some(t_idx) = track_opt {
                        load_audio_async(&ctrl_load, &state_ref, &ui_weak, &st, deck, t_idx);
                    }
                    if let Some(ui) = ui_weak.upgrade() {
                        presentation::render(&st, &ui);
                    }
                }
            }
        });
    }

    // 4. Controles de transporte dos Decks (Play/Pause, CUE, Seek)
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let ctrl = audio_controller.clone();
        ui.on_deck_play(move |deck_num| {
            let Some(i) = deck_idx(deck_num) else { return };
            let deck = if i == 0 { DeckId::Deck1 } else { DeckId::Deck2 };
            if let Ok(mut st) = state_ref.lock() {
                let starting = if i == 0 {
                    st.deck1.audio_ready && !st.deck1.is_playing
                } else {
                    st.deck2.audio_ready && !st.deck2.is_playing
                };
                if starting && st.beat_sync[i] && i != st.master {
                    send_phase_alignment(&ctrl, &st, i);
                }
                ctrl.send(AudioCommand::PlayPause {
                    deck: deck_num as u8,
                });
                st.apply(Command::PlayPause(deck));
                if let Some(ui) = ui_weak.upgrade() {
                    presentation::render(&st, &ui);
                }
            }
        });
    }

    {
        // CUE: pressionar e soltar vão ao motor (spec/rekordbox/30-deck.md); a
        // posição e o estado voltam pelo timer de sincronização.
        let ctrl = audio_controller.clone();
        ui.on_deck_cue(move |deck_num, pressionado| {
            if !(1..=2).contains(&deck_num) {
                return;
            }
            let deck = deck_num as u8;
            ctrl.send(if pressionado {
                AudioCommand::CueDown { deck }
            } else {
                AudioCommand::CueUp { deck }
            });
        });
    }

    // Mixer (p7-0): fader, TRIM e crossfader vão ao motor; a tela atualiza só o mixer.
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let ctrl = audio_controller.clone();
        ui.on_mixer_fader(move |c, v| {
            let Ok(mut st) = state_ref.lock() else { return };
            let Some(f) = st.mix_fader.get_mut(c as usize) else {
                return;
            };
            *f = v;
            ctrl.send(AudioCommand::SetFader {
                deck: c as u8 + 1,
                valor: v,
            });
            if let Some(ui) = ui_weak.upgrade() {
                presentation::render_mixer(&st, &ui);
            }
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let ctrl = audio_controller.clone();
        ui.on_mixer_trim(move |c, v| {
            let Ok(mut st) = state_ref.lock() else { return };
            let Some(t) = st.mix_trim.get_mut(c as usize) else {
                return;
            };
            *t = v;
            ctrl.send(AudioCommand::SetTrim {
                deck: c as u8 + 1,
                valor: v,
            });
            if let Some(ui) = ui_weak.upgrade() {
                presentation::render_mixer(&st, &ui);
            }
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let ctrl = audio_controller.clone();
        ui.on_mixer_cross(move |v| {
            let Ok(mut st) = state_ref.lock() else { return };
            st.mix_cross = v;
            ctrl.send(AudioCommand::SetCrossfader { valor: v });
            if let Some(ui) = ui_weak.upgrade() {
                presentation::render_mixer(&st, &ui);
            }
        });
    }

    // Loops, quantize e linhas de cue (spec/rekordbox/32-loops-cues.md).
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let ctrl = audio_controller.clone();
        ui.on_deck_loop_alternar(move |n| {
            let Some(i) = deck_idx(n) else { return };
            let Ok(mut st) = state_ref.lock() else { return };
            if let Some(cmd) = acoes::loop_alternar(&mut st, i, pos_s(&ctrl, i)) {
                ctrl.send(cmd);
            }
            render_ui(&ui_weak, &st);
        });
    }
    for dobro in [false, true] {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let ctrl = audio_controller.clone();
        let f = move |n: i32| {
            let Some(i) = deck_idx(n) else { return };
            let Ok(mut st) = state_ref.lock() else { return };
            let Some(cmd) = acoes::loop_cut(&mut st, i, dobro) else {
                return;
            };
            ctrl.send(cmd);
            render_ui(&ui_weak, &st);
        };
        if dobro {
            ui.on_deck_loop_dobro(f);
        } else {
            ui.on_deck_loop_metade(f);
        }
    }
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let ctrl = audio_controller.clone();
        ui.on_deck_quantize(move |n| {
            let Some(i) = deck_idx(n) else { return };
            let Ok(mut st) = state_ref.lock() else { return };
            ctrl.send(acoes::quantize(&mut st, i));
            render_ui(&ui_weak, &st);
        });
    }
    // Demais botões do deck (spec/rekordbox/30-deck.md, 32-loops-cues.md;
    // dossiê evidence/p6-2-sync-slip-quantize.md).
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let ctrl = audio_controller.clone();
        ui.on_deck_botao(move |n, nome| {
            let Some(i) = deck_idx(n) else { return };
            let Ok(mut st) = state_ref.lock() else { return };
            let deck = n as u8;
            match nome.as_str() {
                // MT (unit#15): alterna.
                "mt" => {
                    st.master_tempo[i] = !st.master_tempo[i];
                    ctrl.send(AudioCommand::SetMasterTempo {
                        deck,
                        on: st.master_tempo[i],
                    });
                }
                // SLIP (unit#14): só o estado; a sombra é DESCONHECIDA.
                "slip" => {
                    st.slip[i] = !st.slip[i];
                    ctrl.send(AudioCommand::SetSlip {
                        deck,
                        on: st.slip[i],
                    });
                }
                // BEAT SYNC (unit#22): o não master assume o andamento do master.
                "beat_sync" => {
                    st.beat_sync[i] = !st.beat_sync[i];
                    let master_bpm = sync::bpm_do_master(&st, i);
                    ctrl.send(AudioCommand::BeatSync {
                        deck,
                        on: st.beat_sync[i],
                        own_bpm: sync::bpm_base(&st, i),
                        master_bpm,
                    });
                    sync::aplicar_sync(&mut st, i);
                    if st.beat_sync[i] && i == st.master {
                        st.status_text =
                            format!("BEAT SYNC: deck {n} é MASTER; ative SYNC no outro deck");
                    } else if st.beat_sync[i] && !send_phase_alignment(&ctrl, &st, i) {
                        st.status_text = "BEAT SYNC: grade insuficiente para alinhar a fase".into();
                    }
                }
                // MASTER: estado do app (nenhuma chamada na vtable do deck);
                // decks em sync passam a seguir o novo master.
                "master" => {
                    st.master = i;
                    let j = 1 - i;
                    if st.beat_sync[j] {
                        ctrl.send(AudioCommand::BeatSync {
                            deck: (j + 1) as u8,
                            on: true,
                            own_bpm: sync::bpm_base(&st, j),
                            master_bpm: sync::bpm_do_master(&st, j),
                        });
                        sync::aplicar_sync(&mut st, j);
                        send_phase_alignment(&ctrl, &st, j);
                    }
                }
                // KEY SYNC (unit#45): desloca o tom para o do outro deck
                // (na captura, o deck 1 casou com o deck 2). Desligar só apaga
                // o indicador: o que o rekordbox faz ao desligar não foi observado.
                "key_sync" => {
                    st.key_sync[i] = !st.key_sync[i];
                    if st.key_sync[i] {
                        let (own, refe) = if i == 0 {
                            (&st.deck1.key, &st.deck2.key)
                        } else {
                            (&st.deck2.key, &st.deck1.key)
                        };
                        match sync::key_sync_shift(own, refe) {
                            Some(s) => {
                                st.key_shift[i] = s;
                                ctrl.send(AudioCommand::KeyShift { deck, semitones: s });
                            }
                            None => st.status_text = "KEY SYNC: tons sem regra observada".into(),
                        }
                    }
                }
                "loop_modo" => acoes::loop_modo(&mut st, i),
                "loop_in" => {
                    if let Some(c) = acoes::loop_in(&mut st, i, pos_s(&ctrl, i)) {
                        ctrl.send(c);
                    }
                }
                "loop_out" => {
                    if let Some(c) = acoes::loop_out(&mut st, i, pos_s(&ctrl, i)) {
                        ctrl.send(c);
                    }
                }
                "loop_exit" => {
                    for c in acoes::loop_exit_reloop(&mut st, i) {
                        ctrl.send(c);
                    }
                }
                "memory" => acoes::memory(&mut st, i, pos_s(&ctrl, i)),
                _ => return,
            }
            render_ui(&ui_weak, &st);
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let ctrl = audio_controller.clone();
        ui.on_deck_cue_linha(move |n, linha| {
            let (Some(i), Ok(linha)) = (deck_idx(n), usize::try_from(linha)) else {
                return;
            };
            let Ok(mut st) = state_ref.lock() else { return };
            if let Some(cmd) = acoes::cue_linha(&mut st, i, linha, pos_s(&ctrl, i)) {
                ctrl.send(cmd);
            }
            render_ui(&ui_weak, &st);
        });
    }

    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let ctrl = audio_controller.clone();
        ui.on_deck_seek(move |deck_num, pos_rel| {
            let deck = match deck_num {
                1 => DeckId::Deck1,
                2 => DeckId::Deck2,
                _ => return,
            };
            if let Ok(mut st) = state_ref.lock() {
                st.apply(Command::Seek(deck, f64::from(pos_rel)));
                let dur = match deck {
                    DeckId::Deck1 => st.deck1.dur_s,
                    DeckId::Deck2 => st.deck2.dur_s,
                };
                let sec = f64::from(pos_rel).clamp(0.0, 1.0) * dur;
                ctrl.send(AudioCommand::SeekSeconds {
                    deck: deck_num as u8,
                    seconds: sec,
                });
                if let Some(ui) = ui_weak.upgrade() {
                    presentation::render(&st, &ui);
                }
            }
        });
    }

    // Clique e arrasto na grade/forma de onda ampliada usam segundos
    // absolutos. O estado continua recebendo a posição relativa para manter
    // em um só lugar a atualização dos tempos exibidos.
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let ctrl = audio_controller.clone();
        ui.on_deck_wave_seek(move |deck_num, seconds| {
            let deck = match deck_num {
                1 => DeckId::Deck1,
                2 => DeckId::Deck2,
                _ => return,
            };
            if let Ok(mut st) = state_ref.lock() {
                let dur = match deck {
                    DeckId::Deck1 => st.deck1.dur_s,
                    DeckId::Deck2 => st.deck2.dur_s,
                };
                if dur <= 0.0 {
                    return;
                }
                let sec = f64::from(seconds).clamp(0.0, dur);
                st.apply(Command::Seek(deck, sec / dur));
                ctrl.send(AudioCommand::SeekSeconds {
                    deck: deck_num as u8,
                    seconds: sec,
                });
                if let Some(ui) = ui_weak.upgrade() {
                    presentation::render(&st, &ui);
                }
            }
        });
    }

    // Scratch da waveform: durante o contato, o motor lê o buffer seguindo o
    // mouse. Diferente de seek, deslocamentos negativos geram backspin audível.
    {
        let ctrl = audio_controller.clone();
        ui.on_deck_wave_scratch_begin(move |deck| {
            if (1..=2).contains(&deck) {
                ctrl.send(AudioCommand::ScratchBegin { deck: deck as u8 });
            }
        });
    }
    {
        let ctrl = audio_controller.clone();
        ui.on_deck_wave_scratch_to(move |deck, seconds| {
            if (1..=2).contains(&deck) {
                ctrl.send(AudioCommand::ScratchTo {
                    deck: deck as u8,
                    seconds: f64::from(seconds),
                });
            }
        });
    }
    {
        let ctrl = audio_controller.clone();
        ui.on_deck_wave_scratch_end(move |deck| {
            if (1..=2).contains(&deck) {
                ctrl.send(AudioCommand::ScratchEnd { deck: deck as u8 });
            }
        });
    }

    // GRID/PHRASE EDIT (rekordbox 7.2.8, manual pp. 135–136).
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        let ctrl = audio_controller.clone();
        let tap_clock = std::time::Instant::now();
        ui.on_grid_command(move |deck_num, name, value| {
            let Some(i) = deck_idx(deck_num) else { return };
            let Ok(mut st) = state_ref.lock() else { return };
            let position = pos_s(&ctrl, i);
            let step = st.grid_edit[i].step_ms();
            let mut changed = false;
            match name.as_str() {
                "downbeat" => {
                    changed = st.edit_beatgrid(i, GridAction::SetDownbeat, position);
                }
                "goto_downbeat" => {
                    let beats = if i == 0 {
                        &st.deck1.beats
                    } else {
                        &st.deck2.beats
                    };
                    if let Some(target) = beatgrid::nearest_downbeat_s(beats, position) {
                        let id = if i == 0 { DeckId::Deck1 } else { DeckId::Deck2 };
                        let dur = if i == 0 {
                            st.deck1.dur_s
                        } else {
                            st.deck2.dur_s
                        };
                        if dur > 0.0 {
                            st.apply(Command::Seek(id, target / dur));
                            ctrl.send(AudioCommand::SeekSeconds {
                                deck: deck_num as u8,
                                seconds: target,
                            });
                        }
                    }
                }
                "shift_left" => changed = st.edit_beatgrid(i, GridAction::ShiftMs(-step), position),
                "shift_right" => changed = st.edit_beatgrid(i, GridAction::ShiftMs(step), position),
                "shift_left_3" => changed = st.edit_beatgrid(i, GridAction::ShiftMs(-3), position),
                "shift_left_1" => changed = st.edit_beatgrid(i, GridAction::ShiftMs(-1), position),
                "shift_right_1" => changed = st.edit_beatgrid(i, GridAction::ShiftMs(1), position),
                "shift_right_3" => changed = st.edit_beatgrid(i, GridAction::ShiftMs(3), position),
                "undo" => changed = st.undo_beatgrid(i),
                "redo" => changed = st.redo_beatgrid(i),
                "set_bpm" if value > 0.0 => {
                    changed = st.edit_beatgrid(i, GridAction::SetBpm(f64::from(value)), position);
                }
                "tap" => {
                    if let Some(bpm) = st.grid_edit[i].tap(tap_clock.elapsed().as_secs_f64()) {
                        changed = st.edit_beatgrid(i, GridAction::SetBpm(bpm), position);
                    }
                }
                "narrow" => changed = st.edit_beatgrid(i, GridAction::SpacingMs(-step), position),
                "widen" => changed = st.edit_beatgrid(i, GridAction::SpacingMs(step), position),
                "narrow_3" => changed = st.edit_beatgrid(i, GridAction::SpacingMs(-3), position),
                "narrow_1" => changed = st.edit_beatgrid(i, GridAction::SpacingMs(-1), position),
                "widen_1" => changed = st.edit_beatgrid(i, GridAction::SpacingMs(1), position),
                "widen_3" => changed = st.edit_beatgrid(i, GridAction::SpacingMs(3), position),
                "half" => changed = st.edit_beatgrid(i, GridAction::ScaleBpm(0.5), position),
                "double" => changed = st.edit_beatgrid(i, GridAction::ScaleBpm(2.0), position),
                "range" => {
                    st.grid_edit[i].range = if st.grid_edit[i].range == EditRange::Whole {
                        EditRange::FromPosition
                    } else {
                        EditRange::Whole
                    };
                }
                "reset" if value > 0.0 => {
                    changed = st.edit_beatgrid(
                        i,
                        GridAction::ResetAt {
                            bpm: f64::from(value),
                        },
                        position,
                    );
                }
                "fine" => st.grid_edit[i].fine = !st.grid_edit[i].fine,
                "metronome" => {
                    st.grid_edit[i].toggle_metronome();
                    send_metronome(&ctrl, &st, i);
                }
                "play" => {
                    let id = if i == 0 { DeckId::Deck1 } else { DeckId::Deck2 };
                    let starting = if i == 0 {
                        st.deck1.audio_ready && !st.deck1.is_playing
                    } else {
                        st.deck2.audio_ready && !st.deck2.is_playing
                    };
                    if starting && st.beat_sync[i] && i != st.master {
                        send_phase_alignment(&ctrl, &st, i);
                    }
                    ctrl.send(AudioCommand::PlayPause {
                        deck: deck_num as u8,
                    });
                    st.apply(Command::PlayPause(id));
                }
                "lock" => {
                    st.grid_edit[i].locked = !st.grid_edit[i].locked;
                    st.save_beatgrid(i);
                }
                _ => {}
            }
            if changed {
                st.status_text = format!("Beatgrid do deck {deck_num} alterada");
                if st.grid_edit[i].metronome > 0 {
                    send_metronome(&ctrl, &st, i);
                }
            }
            render_ui(&ui_weak, &st);
        });
    }

    // 4. Busca por texto (U1.1)
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        ui.on_search_edited(move |query| {
            if let Ok(mut st) = state_ref.lock() {
                st.apply(Command::Search(query.to_string()));
                if let Some(ui) = ui_weak.upgrade() {
                    presentation::render(&st, &ui);
                }
            }
        });
    }

    // 6. Lista de reprodução na árvore (-1 = coleção inteira)
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        ui.on_playlist_selected(move |idx| {
            if let Ok(mut st) = state_ref.lock() {
                st.apply(Command::SelectPlaylist(usize::try_from(idx).ok()));
                if let Some(ui) = ui_weak.upgrade() {
                    presentation::render(&st, &ui);
                }
            }
        });
    }

    // 5. Ordenação por coluna (U1.2)
    {
        let ui_weak = ui.as_weak();
        let state_ref = state.clone();
        ui.on_sort_clicked(move |col_idx| {
            let col = match col_idx {
                0 => SortColumn::Id,
                1 => SortColumn::Title,
                2 => SortColumn::Artist,
                3 => SortColumn::Bpm,
                4 => SortColumn::Duration,
                5 => SortColumn::Bars,
                _ => return,
            };
            if let Ok(mut st) = state_ref.lock() {
                st.apply(Command::SortBy(col));
                if let Some(ui) = ui_weak.upgrade() {
                    presentation::render(&st, &ui);
                }
            }
        });
    }

    // VLBX_CAPTURA=<arquivo.ppm>: abre no cliente de referência (1920x1051, o
    // mesmo do rekordbox maximizado), espera o scan e grava a tela em PPM.
    // Ferramenta de comparação de layout; não afeta o uso normal.
    let capture_timer = slint::Timer::default();
    if let Some(out) = std::env::var_os("VLBX_CAPTURA") {
        ui.window().set_size(slint::PhysicalSize::new(1920, 1051));
        let ui_weak = ui.as_weak();
        // VLBX_CAPTURA_BUSCA=<texto>: busca e carrega o primeiro resultado no deck 1.
        let busca = std::env::var("VLBX_CAPTURA_BUSCA").ok();
        let captura_grid = std::env::var("VLBX_CAPTURA_GRID")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .filter(|v| (1..=2).contains(v));
        let state_ref = state.clone();
        capture_timer.start(
            slint::TimerMode::SingleShot,
            std::time::Duration::from_secs(8),
            move || {
                let Some(ui) = ui_weak.upgrade() else { return };
                if let Some(deck) = captura_grid {
                    ui.set_grid_edit_deck(deck);
                }
                if let (Some(b), Ok(mut st)) = (busca.clone(), state_ref.lock()) {
                    st.apply(Command::Search(b));
                    st.apply(Command::LoadToDeck {
                        deck: DeckId::Deck1,
                        filtered_index: 0,
                    });
                    presentation::render(&st, &ui);
                }
                match ui.window().take_snapshot() {
                    Ok(buf) => {
                        let mut ppm =
                            format!("P6\n{} {}\n255\n", buf.width(), buf.height()).into_bytes();
                        ppm.extend(buf.as_slice().iter().flat_map(|p| [p.r, p.g, p.b]));
                        if let Err(e) = std::fs::write(&out, ppm) {
                            eprintln!("captura: {e}");
                        }
                    }
                    Err(e) => eprintln!("captura: {e}"),
                }
                let _ = slint::quit_event_loop();
            },
        );
    }

    ui.run()
}

#[cfg(test)]
mod busca_tests {
    use super::nome_normalizado;

    #[test]
    fn ignora_numeracao_e_extensao() {
        let a = nome_normalizado("01 - FAIXA PRETA ((FUNKHALL)).mp3");
        assert_eq!(a, nome_normalizado("FAIXA PRETA ((FUNKHALL)).wav"));
        assert_eq!(
            nome_normalizado("01. Brutalismus 3000 - Horime.flac").as_deref(),
            Some("brutalismus 3000 - horime")
        );
        assert_eq!(nome_normalizado("capa.jpg"), None);
    }
}
