//! Módulo de estado e comandos da interface (`AppState`, `Command`).
//!
//! Implementa os requisitos U0.1, U0.2 e U1 do `PLANO-UI-FUNCOES.md` seguindo o
//! Princípio 1: a interface só emite comandos e recebe snapshots.
//! Toda lógica de seleção, ordenação, filtro e decks é puramente testável aqui
//! sem instanciar janelas Slint.

use crate::beatgrid::{BeatGridEditor, GridAction};
use crate::library::{self, Playlist, Track};
use std::cmp::Ordering;
use std::sync::Arc;
use vilelabox_anlz::{
    all_sections, cue_sources, waveform_from_ext, Beat, Grid, RawCue, WaveformDetail,
    WaveformPreview,
};
use vilelabox_audio::AudioBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeckId {
    Deck1,
    Deck2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Id,
    Title,
    Artist,
    Bpm,
    Duration,
    Bars,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    #[must_use]
    pub fn toggle(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

/// Artista exibido quando o nome do arquivo não traz um; não é dado da faixa.
pub const SEM_ARTISTA: &str = "Vilelabox / Coleção";

pub fn split_artist_title(raw: &str) -> (String, String) {
    let name = raw
        .strip_suffix(".mp3")
        .or_else(|| raw.strip_suffix(".flac"))
        .or_else(|| raw.strip_suffix(".wav"))
        .or_else(|| raw.strip_suffix(".aiff"))
        .or_else(|| raw.strip_suffix(".m4a"))
        .or_else(|| raw.strip_suffix(".MP3"))
        .or_else(|| raw.strip_suffix(".FLAC"))
        .unwrap_or(raw);

    let clean = if let Some(rest) = name.strip_prefix(|c: char| c.is_ascii_digit()) {
        let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
        let rest = rest.trim_start_matches(['.', '-', ' ']);
        rest.trim()
    } else {
        name
    };

    if let Some((artist, title)) = clean.split_once(" - ") {
        (artist.trim().to_string(), title.trim().to_string())
    } else {
        (SEM_ARTISTA.to_string(), clean.trim().to_string())
    }
}

pub fn duration_ms(grid: Option<&Grid>) -> u64 {
    let Some(grid) = grid else {
        return 0;
    };
    let Some(last) = grid.beats.last() else {
        return 0;
    };
    let beat_ms = if grid.bpm > 0.0 {
        60_000.0 / grid.bpm
    } else {
        0.0
    };
    (last.time_ms as f64 + beat_ms).max(0.0) as u64
}

/// BPM com duas casas; "—" sem valor.
pub fn fmt_bpm(bpm: f64) -> String {
    if bpm > 0.0 {
        format!("{bpm:.2}")
    } else {
        "—".into()
    }
}

/// Duração `MM:SS`; "—" sem valor.
pub fn fmt_dur(ms: u64) -> String {
    if ms == 0 {
        return "—".into();
    }
    format!("{:02}:{:02}", ms / 60_000, ms % 60_000 / 1000)
}

pub fn count_bars(grid: Option<&Grid>) -> i32 {
    grid.map_or(0, |g| {
        g.beats.iter().filter(|b| b.beat_number == 1).count() as i32
    })
}

/// Formata segundos em MM:SS.d (ex: 01:23.4).
pub fn fmt_tenths(s: f64) -> String {
    let s = s.max(0.0);
    let min = (s / 60.0) as u32;
    let sec = (s % 60.0) as u32;
    let tenths = ((s.fract() * 10.0) as u32).min(9);
    format!("{min:02}:{sec:02}.{tenths}")
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhraseSegment {
    pub start: f32,
    pub end: f32,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeckState {
    pub loaded_track_idx: Option<usize>,
    pub title: String,
    pub artist: String,
    pub duration: String,
    pub bpm: String,
    pub key: String,
    pub info: String,
    pub bars: Vec<f32>,
    pub phrases: Vec<PhraseSegment>,
    pub audio_ready: bool,
    /// Formas de onda do `.EXT` (PWV4 e PWV5/PWV3), quando houver.
    pub preview: Option<WaveformPreview>,
    pub detail: Option<WaveformDetail>,
    /// Cues e loops (PCO2/PCOB) e batidas da grade (PQTZ).
    pub cues: Vec<RawCue>,
    pub beats: Vec<Beat>,
    /// Duração da análise em segundos (PWV5 a 150/s; na falta, a grade).
    pub dur_s: f64,
    /// Posição atual de reprodução em segundos.
    pub pos_s: f64,
    pub is_playing: bool,
    pub is_cue_standby: bool,
    pub tempo_decorrido: String,
    pub tempo_restante: String,
    pub audio_buf: Option<Arc<AudioBuffer>>,
}

impl DeckState {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            loaded_track_idx: None,
            title: String::new(),
            artist: String::new(),
            duration: "—".into(),
            bpm: "—".into(),
            key: "—".into(),
            info: String::new(),
            bars: Vec::new(),
            phrases: Vec::new(),
            audio_ready: false,
            preview: None,
            detail: None,
            cues: Vec::new(),
            beats: Vec::new(),
            dur_s: 0.0,
            pos_s: 0.0,
            is_playing: false,
            is_cue_standby: false,
            tempo_decorrido: "00:00.0".into(),
            tempo_restante: "-00:00.0".into(),
            audio_buf: None,
        }
    }

    #[must_use]
    pub fn load_track(track_idx: usize, track: &Track) -> Self {
        let (artist, title) = track.artist_title();
        let bpm = fmt_bpm(track.bpm());
        let duration = fmt_dur(track.dur_ms());
        let key = track
            .meta
            .as_ref()
            .map(|m| m.tonality.clone())
            .filter(|k| !k.is_empty())
            .unwrap_or_else(|| "—".into());
        let view = library::timeline(track);
        let phrases = view
            .phrases
            .into_iter()
            .map(|p| PhraseSegment {
                start: p.start,
                end: p.end,
                kind: p.kind,
            })
            .collect();
        let ler = |p: &Option<std::path::PathBuf>| p.as_ref().and_then(|p| std::fs::read(p).ok());
        let (dat, ext) = (ler(&track.dat), ler(&track.ext));
        let (mut preview, mut detail) = ext.as_deref().map_or((None, None), waveform_from_ext);
        // WAV com tamanho declarado 0xFFFFFFFF: o rekordbox grava as ondas para
        // ~22 370 s. Com a duração medida no arquivo, o detalhe é cortado nela e a
        // visão geral é refeita a partir dele (a do PWV4 teria ~20 colunas úteis).
        if let (Some(ms), Some(d)) = (track.audio_ms, detail.as_mut()) {
            let n = (ms as usize * crate::waveform::DETALHE_POR_SEGUNDO) / 1000;
            if n + crate::waveform::DETALHE_POR_SEGUNDO < d.height.len() {
                for v in [&mut d.height, &mut d.red, &mut d.green, &mut d.blue] {
                    v.truncate(n);
                }
                preview = Some(crate::waveform::visao_do_detalhe(d, 1200));
            }
        }
        // Cues: do XML quando houver (o ANLZ local não os guarda); senão, do ANLZ.
        let cues = match track.meta.as_ref().filter(|m| !m.cues.is_empty()) {
            Some(m) => m.cues.clone(),
            None => cue_sources(
                &dat.as_deref().map(all_sections).unwrap_or_default(),
                &ext.as_deref().map(all_sections).unwrap_or_default(),
            ),
        };
        let beats = track
            .grid
            .as_ref()
            .map(|g| g.beats.clone())
            .unwrap_or_default();
        let dur_s = detail.as_ref().map_or_else(
            || duration_ms(track.grid.as_ref()) as f64 / 1000.0,
            |d| d.height.len() as f64 / crate::waveform::DETALHE_POR_SEGUNDO as f64,
        );

        let tempo_restante = if dur_s > 0.0 {
            format!("-{}", fmt_tenths(dur_s))
        } else {
            "-00:00.0".into()
        };

        Self {
            loaded_track_idx: Some(track_idx),
            preview,
            detail,
            cues,
            beats,
            dur_s,
            pos_s: 0.0,
            is_playing: false,
            is_cue_standby: true,
            tempo_decorrido: "00:00.0".into(),
            tempo_restante,
            audio_buf: None,
            title,
            artist,
            duration,
            bpm,
            key,
            info: view.info,
            bars: view.bars,
            phrases,
            audio_ready: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub tracks: Vec<Track>,
    pub filtered_indices: Vec<usize>,
    pub selected_filtered_index: Option<usize>,
    pub filter_text: String,
    pub sort_column: SortColumn,
    pub sort_direction: SortDirection,
    pub deck1: DeckState,
    pub deck2: DeckState,
    pub status_text: String,
    /// Por deck [1, 2]: tamanho do loop em batidas (vale entre faixas), loop
    /// ativo e quantize. Tamanho inicial: o da tela medida (8 no deck 1, 4 no 2).
    pub loop_batidas: [f64; 2],
    pub loop_ativo: [bool; 2],
    pub quantize: [bool; 2],
    /// Botões de `evidence/p6-2-sync-slip-quantize.md`: deck master (0/1; o
    /// deck 1 era o master na captura), BEAT SYNC e velocidade resultante,
    /// KEY SYNC (tom de referência e semitons), SLIP, MT e modo de loop
    /// manual (MA; falso = AU).
    pub master: usize,
    pub beat_sync: [bool; 2],
    pub velocidade: [f64; 2],
    pub key_sync: [bool; 2],
    pub key_shift: [i8; 2],
    pub slip: [bool; 2],
    pub master_tempo: [bool; 2],
    pub loop_manual: [bool; 2],
    /// Estado do painel GRID EDIT por deck: histórico, faixa de ajuste,
    /// precisão, bloqueio, TAP e metrônomo.
    pub grid_edit: [BeatGridEditor; 2],
    /// Loop manual por deck (p6-5): IN pendente e o par IN/OUT que o RELOOP reabre, em segundos.
    pub loop_in: [Option<f64>; 2],
    /// Mixer (p7-0), unidades do rekordbox: fader 0..100, TRIM -10..10, crossfader -10..10.
    pub mix_fader: [f32; 2],
    pub mix_trim: [f32; 2],
    pub mix_cross: f32,
    pub loop_pontos: [Option<(f64, f64)>; 2],
    pub scan_in_progress: bool,
    pub clock_text: String,
    /// Listas do XML e a lista que filtra a tabela (None = coleção inteira).
    pub playlists: Vec<Playlist>,
    pub playlist: Option<usize>,
    /// Pasta dos cues gravados (`cues_store`); None = não persiste.
    pub cues_dir: Option<std::path::PathBuf>,
    /// Canal para o serviço do núcleo (P5.1): a tela só envia pedidos; as
    /// respostas voltam como `Command::RespostaNucleo`. Sem canal, cues e
    /// grade vão direto para os arquivos de `cues_dir`.
    pub nucleo: Option<std::sync::mpsc::Sender<crate::nucleo::Pedido>>,
    /// Edições locais desde a carga: respostas de leitura não podem desfazê-las.
    cues_editados: [bool; 2],
    grade_editada: [bool; 2],
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            filtered_indices: Vec::new(),
            selected_filtered_index: None,
            filter_text: String::new(),
            sort_column: SortColumn::Id,
            sort_direction: SortDirection::Ascending,
            deck1: DeckState::empty(),
            deck2: DeckState::empty(),
            status_text: "Iniciando VILELABOX...".into(),
            loop_batidas: [8.0, 4.0],
            loop_ativo: [false, false],
            quantize: [false, false],
            master: 0,
            beat_sync: [false, false],
            velocidade: [1.0, 1.0],
            key_sync: [false, false],
            key_shift: [0, 0],
            slip: [false, false],
            master_tempo: [true, true],
            loop_manual: [false, false],
            grid_edit: [BeatGridEditor::default(), BeatGridEditor::default()],
            loop_in: [None, None],
            mix_fader: [100.0, 100.0],
            mix_trim: [0.0, 0.0],
            mix_cross: 0.0,
            loop_pontos: [None, None],
            scan_in_progress: false,
            clock_text: "--:--".into(),
            playlists: Vec::new(),
            playlist: None,
            cues_dir: None,
            nucleo: None,
            cues_editados: [false; 2],
            grade_editada: [false; 2],
        }
    }

    fn deck_ref(&self, i: usize) -> &DeckState {
        if i == 0 {
            &self.deck1
        } else {
            &self.deck2
        }
    }

    /// Caminho, título e artista da faixa no deck `i`, com o índice dela.
    fn faixa_do_deck(&self, i: usize) -> Option<(usize, String, String, String)> {
        let t = self.deck_ref(i).loaded_track_idx?;
        let track = self.tracks.get(t)?;
        let (artist, title) = track.artist_title();
        Some((t, track.path.clone()?, title, artist))
    }

    /// Envia um pedido ao serviço do núcleo; falha vira aviso.
    fn pedir(&mut self, pedido: crate::nucleo::Pedido) -> bool {
        let Some(tx) = &self.nucleo else {
            return false;
        };
        if tx.send(pedido).is_err() {
            self.status_text = "Banco: serviço do núcleo parado".into();
            self.nucleo = None;
            return false;
        }
        true
    }

    /// Áudio do deck `i` decodificado: pede ao núcleo para gravar taxa,
    /// canais e duração e reler a preparação; a resposta migra os JSON.
    pub fn audio_pronto(&mut self, i: usize, taxa: u32, canais: u16, frames: u64) {
        let Some((faixa, caminho, titulo, artista)) = self.faixa_do_deck(i) else {
            return;
        };
        if self.pedir(crate::nucleo::Pedido::RegistrarAudio {
            caminho: caminho.clone(),
            taxa,
            canais,
            frames,
        }) {
            self.pedir(crate::nucleo::Pedido::Carregar {
                deck: i,
                faixa,
                caminho,
                titulo,
                artista,
                apos_audio: true,
            });
        }
    }

    /// Persiste os cues do deck `i` (0/1): pelo núcleo quando há canal (o
    /// JSON só se o banco recusar), senão direto no JSON.
    pub fn salvar_cues(&mut self, i: usize) {
        self.cues_editados[i] = true;
        if let Some((faixa, ..)) = self.faixa_do_deck(i) {
            let cues = self.deck_ref(i).cues.clone();
            if self.pedir(crate::nucleo::Pedido::GravarCues {
                deck: i,
                faixa,
                cues,
            }) {
                return;
            }
        }
        self.gravar_cues_json(i);
    }

    fn gravar_cues_json(&mut self, i: usize) {
        let d = self.deck_ref(i);
        let path = d
            .loaded_track_idx
            .and_then(|t| self.tracks.get(t)?.path.clone());
        if let (Some(dir), Some(p)) = (&self.cues_dir, path) {
            if let Err(e) = crate::cues_store::gravar(dir, &p, &d.cues) {
                self.status_text = format!("Cues não gravados: {e}");
            }
        }
    }

    /// Resposta do serviço do núcleo; as de faixa que já saiu do deck são
    /// descartadas.
    fn aplicar_resposta(&mut self, r: crate::nucleo::Resposta) {
        use crate::nucleo::Resposta;
        let atual = |st: &Self, deck: usize, faixa: usize| {
            st.deck_ref(deck).loaded_track_idx == Some(faixa)
        };
        match r {
            Resposta::Status(texto) => self.status_text = texto,
            Resposta::CuesRecusados { deck, faixa } if atual(self, deck, faixa) => {
                self.gravar_cues_json(deck);
            }
            Resposta::GradeRecusada { deck, faixa } if atual(self, deck, faixa) => {
                self.gravar_grade_json(deck);
            }
            Resposta::Preparacao {
                deck,
                faixa,
                prep,
                apos_audio,
            } if atual(self, deck, faixa) => {
                if apos_audio {
                    // Migração: JSON existente e banco ainda vazio.
                    let Some((_, caminho, ..)) = self.faixa_do_deck(deck) else {
                        return;
                    };
                    let Some(dir) = self.cues_dir.clone() else {
                        return;
                    };
                    if prep.cues.is_empty() && !crate::cues_store::ler(&dir, &caminho).is_empty() {
                        self.salvar_cues(deck);
                    }
                    if prep.grade.is_none()
                        && crate::grid_store::read(&dir, std::path::Path::new(&caminho)).is_some()
                    {
                        self.save_beatgrid(deck);
                    }
                    return;
                }
                // Uma leitura anterior à edição não deve desfazer o trabalho local.
                let d = if deck == 0 {
                    &mut self.deck1
                } else {
                    &mut self.deck2
                };
                if !self.cues_editados[deck] && !prep.cues.is_empty() {
                    d.cues.retain(|c| c.is_loop);
                    d.cues.extend(prep.cues);
                }
                if let Some((beats, locked)) = prep.grade.filter(|_| !self.grade_editada[deck]) {
                    d.beats = beats;
                    self.grid_edit[deck].locked = locked;
                    self.sync_beatgrid(deck);
                }
            }
            _ => {}
        }
    }

    pub fn apply(&mut self, cmd: Command) {
        match cmd {
            Command::ScanStarted { dir_display } => {
                self.scan_in_progress = true;
                self.status_text = format!("Varrendo análises em {dir_display}...");
            }
            Command::ScanFinished { tracks, summary } => {
                self.scan_in_progress = false;
                self.tracks = tracks;
                if self.nucleo.is_some() {
                    let pedido = crate::nucleo::Pedido::Sincronizar {
                        faixas: self.tracks.clone(),
                        listas: self.playlists.clone(),
                    };
                    self.pedir(pedido);
                }
                self.status_text = summary;
                self.recompute_filtered_indices();

                if !self.filtered_indices.is_empty() && self.selected_filtered_index.is_none() {
                    self.selected_filtered_index = Some(0);
                }
            }
            Command::SelectTrack(filtered_index) => {
                if filtered_index < self.filtered_indices.len() {
                    self.selected_filtered_index = Some(filtered_index);
                }
            }
            Command::LoadToDeck {
                deck,
                filtered_index,
            } => {
                if let Some(&track_idx) = self.filtered_indices.get(filtered_index) {
                    if let Some(track) = self.tracks.get(track_idx) {
                        let mut deck_state = DeckState::load_track(track_idx, track);
                        // Arquivo próprio da faixa substitui os cues (loops da origem ficam).
                        if let (Some(dir), Some(p)) = (&self.cues_dir, &track.path) {
                            let salvos = crate::cues_store::ler(dir, p);
                            if !salvos.is_empty() {
                                deck_state.cues.retain(|c| c.is_loop);
                                deck_state.cues.extend(salvos);
                            }
                            if let Some((beats, locked)) =
                                crate::grid_store::read(dir, std::path::Path::new(p))
                            {
                                deck_state.beats = beats;
                                let i = if deck == DeckId::Deck1 { 0 } else { 1 };
                                self.grid_edit[i].locked = locked;
                            }
                        }
                        match deck {
                            DeckId::Deck1 => {
                                self.deck1 = deck_state;
                                self.grid_edit[0].clear_history();
                            }
                            DeckId::Deck2 => {
                                self.deck2 = deck_state;
                                self.grid_edit[1].clear_history();
                            }
                        }
                        let i = if deck == DeckId::Deck1 { 0 } else { 1 };
                        self.cues_editados[i] = false;
                        self.grade_editada[i] = false;
                        self.sync_beatgrid(i);
                        // Banco próprio: a preparação gravada lá chega como
                        // resposta e vale sobre o JSON.
                        if let Some((faixa, caminho, titulo, artista)) = self.faixa_do_deck(i) {
                            self.pedir(crate::nucleo::Pedido::Carregar {
                                deck: i,
                                faixa,
                                caminho,
                                titulo,
                                artista,
                                apos_audio: false,
                            });
                        }
                    }
                }
            }
            Command::LoadSelectedToDeck(deck) => {
                if let Some(sel_idx) = self.selected_filtered_index {
                    self.apply(Command::LoadToDeck {
                        deck,
                        filtered_index: sel_idx,
                    });
                }
            }
            Command::Search(query) => {
                self.filter_text = query;
                self.recompute_filtered_indices();
                if self.filter_text.is_empty() {
                    self.status_text = format!("{} faixas carregadas", self.tracks.len());
                } else {
                    self.status_text = format!(
                        "{} faixas encontradas para \"{}\"",
                        self.filtered_indices.len(),
                        self.filter_text
                    );
                }
            }
            Command::SortBy(col) => {
                if self.sort_column == col {
                    self.sort_direction = self.sort_direction.toggle();
                } else {
                    self.sort_column = col;
                    self.sort_direction = SortDirection::Ascending;
                }
                self.recompute_filtered_indices();
            }
            Command::TickClock(clock) => {
                self.clock_text = clock;
            }
            Command::PlaylistsLoaded(playlists) => {
                self.playlists = playlists;
                self.playlist = None;
            }
            Command::SelectPlaylist(idx) => {
                self.playlist = idx.filter(|&i| i < self.playlists.len());
                self.selected_filtered_index = None;
                self.recompute_filtered_indices();
            }
            Command::PlayPause(deck) => {
                let d = match deck {
                    DeckId::Deck1 => &mut self.deck1,
                    DeckId::Deck2 => &mut self.deck2,
                };
                if d.audio_ready {
                    d.is_playing = !d.is_playing;
                    if d.is_playing {
                        d.is_cue_standby = false;
                    }
                }
            }
            Command::Seek(deck, pos_rel) => {
                let d = match deck {
                    DeckId::Deck1 => &mut self.deck1,
                    DeckId::Deck2 => &mut self.deck2,
                };
                if d.audio_ready && d.dur_s > 0.0 {
                    let rel = pos_rel.clamp(0.0, 1.0);
                    d.pos_s = rel * d.dur_s;
                    d.tempo_decorrido = fmt_tenths(d.pos_s);
                    d.tempo_restante = format!("-{}", fmt_tenths(d.dur_s - d.pos_s));
                }
            }
            Command::SetDeckAudioReady { deck, ready } => {
                let d = match deck {
                    DeckId::Deck1 => &mut self.deck1,
                    DeckId::Deck2 => &mut self.deck2,
                };
                d.audio_ready = ready;
            }
            Command::RespostaNucleo(r) => self.aplicar_resposta(r),
            Command::SyncAudioSnapshots {
                deck1_pos_s,
                deck1_playing,
                deck1_cue,
                deck2_pos_s,
                deck2_playing,
                deck2_cue,
            } => {
                if self.deck1.audio_ready {
                    self.deck1.pos_s = deck1_pos_s;
                    self.deck1.is_playing = deck1_playing;
                    self.deck1.is_cue_standby = deck1_cue;
                    self.deck1.tempo_decorrido = fmt_tenths(deck1_pos_s);
                    let rem1 = (self.deck1.dur_s - deck1_pos_s).max(0.0);
                    self.deck1.tempo_restante = format!("-{}", fmt_tenths(rem1));
                }
                if self.deck2.audio_ready {
                    self.deck2.pos_s = deck2_pos_s;
                    self.deck2.is_playing = deck2_playing;
                    self.deck2.is_cue_standby = deck2_cue;
                    self.deck2.tempo_decorrido = fmt_tenths(deck2_pos_s);
                    let rem2 = (self.deck2.dur_s - deck2_pos_s).max(0.0);
                    self.deck2.tempo_restante = format!("-{}", fmt_tenths(rem2));
                }
            }
        }
    }

    /// Aplica uma operação do GRID EDIT e mantém deck, coleção e visão geral
    /// coerentes. A persistência no formato ANLZ é tratada separadamente.
    pub fn edit_beatgrid(&mut self, i: usize, action: GridAction, position_s: f64) -> bool {
        let changed = match i {
            0 => {
                self.grid_edit[0].apply(&mut self.deck1.beats, action, position_s, self.deck1.dur_s)
            }
            1 => {
                self.grid_edit[1].apply(&mut self.deck2.beats, action, position_s, self.deck2.dur_s)
            }
            _ => false,
        };
        if changed {
            self.sync_beatgrid(i);
            self.save_beatgrid(i);
        }
        changed
    }

    pub fn undo_beatgrid(&mut self, i: usize) -> bool {
        let changed = match i {
            0 => self.grid_edit[0].undo(&mut self.deck1.beats),
            1 => self.grid_edit[1].undo(&mut self.deck2.beats),
            _ => false,
        };
        if changed {
            self.sync_beatgrid(i);
            self.save_beatgrid(i);
        }
        changed
    }

    pub fn redo_beatgrid(&mut self, i: usize) -> bool {
        let changed = match i {
            0 => self.grid_edit[0].redo(&mut self.deck1.beats),
            1 => self.grid_edit[1].redo(&mut self.deck2.beats),
            _ => false,
        };
        if changed {
            self.sync_beatgrid(i);
            self.save_beatgrid(i);
        }
        changed
    }

    fn sync_beatgrid(&mut self, i: usize) {
        let (track_idx, beats, dur_s) = if i == 0 {
            (
                self.deck1.loaded_track_idx,
                self.deck1.beats.clone(),
                self.deck1.dur_s,
            )
        } else {
            (
                self.deck2.loaded_track_idx,
                self.deck2.beats.clone(),
                self.deck2.dur_s,
            )
        };
        let bars: Vec<f32> = beats
            .iter()
            .filter(|b| b.beat_number == 1)
            .map(|b| (f64::from(b.time_ms) / 1000.0 / dur_s.max(0.001)).clamp(0.0, 1.0) as f32)
            .collect();
        if i == 0 {
            self.deck1.bars = bars;
        } else {
            self.deck2.bars = bars;
        }
        let Some(track) = track_idx.and_then(|idx| self.tracks.get_mut(idx)) else {
            return;
        };
        let Some(grid) = track.grid.as_mut() else {
            return;
        };
        grid.beats = beats;
        if let Some(first) = grid.beats.first() {
            grid.bpm = first.bpm;
            grid.first_beat_time_s = f64::from(first.time_ms) / 1000.0;
        }
        grid.variable = grid
            .beats
            .first()
            .is_some_and(|first| grid.beats.iter().any(|b| (b.bpm - first.bpm).abs() > 1e-9));
        grid.downbeat_index = grid
            .beats
            .iter()
            .position(|b| b.beat_number == 1)
            .unwrap_or(0) as u32;
    }

    pub fn save_beatgrid(&mut self, i: usize) {
        self.grade_editada[i] = true;
        if let Some((faixa, ..)) = self.faixa_do_deck(i) {
            let beats = self.deck_ref(i).beats.clone();
            let bloqueada = self.grid_edit[i].locked;
            if self.pedir(crate::nucleo::Pedido::GravarGrade {
                deck: i,
                faixa,
                beats,
                bloqueada,
            }) {
                return;
            }
        }
        self.gravar_grade_json(i);
    }

    fn gravar_grade_json(&mut self, i: usize) {
        let deck = self.deck_ref(i);
        let path = deck
            .loaded_track_idx
            .and_then(|idx| self.tracks.get(idx)?.path.clone());
        if let (Some(base), Some(path)) = (&self.cues_dir, path) {
            if let Err(error) = crate::grid_store::write(
                base,
                std::path::Path::new(&path),
                &deck.beats,
                self.grid_edit[i].locked,
            ) {
                self.status_text = format!("Beatgrid não gravada: {error}");
            }
        }
    }

    fn recompute_filtered_indices(&mut self) {
        let currently_selected_raw_idx = self
            .selected_filtered_index
            .and_then(|idx| self.filtered_indices.get(idx).copied());

        let term = self.filter_text.trim().to_lowercase();
        // Universo: a lista selecionada (na ordem dela) ou a coleção inteira.
        let universo: Vec<usize> = match self.playlist.and_then(|i| self.playlists.get(i)) {
            Some(p) => p.tracks.clone(),
            None => (0..self.tracks.len()).collect(),
        };
        let ordem: std::collections::HashMap<usize, usize> = universo
            .iter()
            .enumerate()
            .map(|(pos, &i)| (i, pos))
            .collect();
        let mut matches: Vec<usize> = universo
            .into_iter()
            .filter(|&idx| {
                let Some(track) = self.tracks.get(idx) else {
                    return false;
                };
                if term.is_empty() {
                    return true;
                }
                let (artist, title) = track.artist_title();
                track.title.to_lowercase().contains(&term)
                    || title.to_lowercase().contains(&term)
                    || artist.to_lowercase().contains(&term)
            })
            .collect();

        let tracks = &self.tracks;
        let col = self.sort_column;
        let asc = self.sort_direction == SortDirection::Ascending;

        matches.sort_by(|&a_idx, &b_idx| {
            let tr_a = &tracks[a_idx];
            let tr_b = &tracks[b_idx];

            let ord = match col {
                SortColumn::Id => ordem[&a_idx].cmp(&ordem[&b_idx]),
                SortColumn::Title => {
                    let (_, title_a) = tr_a.artist_title();
                    let (_, title_b) = tr_b.artist_title();
                    title_a
                        .to_lowercase()
                        .cmp(&title_b.to_lowercase())
                        .then_with(|| a_idx.cmp(&b_idx))
                }
                SortColumn::Artist => {
                    let (artist_a, _) = tr_a.artist_title();
                    let (artist_b, _) = tr_b.artist_title();
                    artist_a
                        .to_lowercase()
                        .cmp(&artist_b.to_lowercase())
                        .then_with(|| a_idx.cmp(&b_idx))
                }
                SortColumn::Bpm => {
                    let bpm_a = tr_a.bpm();
                    let bpm_b = tr_b.bpm();
                    bpm_a
                        .partial_cmp(&bpm_b)
                        .unwrap_or(Ordering::Equal)
                        .then_with(|| a_idx.cmp(&b_idx))
                }
                SortColumn::Duration => {
                    let dur_a = tr_a.dur_ms();
                    let dur_b = tr_b.dur_ms();
                    dur_a.cmp(&dur_b).then_with(|| a_idx.cmp(&b_idx))
                }
                SortColumn::Bars => {
                    let bars_a = count_bars(tr_a.grid.as_ref());
                    let bars_b = count_bars(tr_b.grid.as_ref());
                    bars_a.cmp(&bars_b).then_with(|| a_idx.cmp(&b_idx))
                }
            };

            if asc {
                ord
            } else {
                ord.reverse()
            }
        });

        self.filtered_indices = matches;

        // Tenta preservar a seleção do item
        if let Some(target) = currently_selected_raw_idx {
            self.selected_filtered_index =
                self.filtered_indices.iter().position(|&idx| idx == target);
        } else if !self.filtered_indices.is_empty() {
            self.selected_filtered_index = Some(0);
        } else {
            self.selected_filtered_index = None;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    ScanStarted {
        dir_display: String,
    },
    ScanFinished {
        tracks: Vec<Track>,
        summary: String,
    },
    SelectTrack(usize),
    LoadToDeck {
        deck: DeckId,
        filtered_index: usize,
    },
    LoadSelectedToDeck(DeckId),
    Search(String),
    PlaylistsLoaded(Vec<Playlist>),
    SelectPlaylist(Option<usize>),
    SortBy(SortColumn),
    TickClock(String),
    PlayPause(DeckId),
    Seek(DeckId, f64),
    SetDeckAudioReady {
        deck: DeckId,
        ready: bool,
    },
    /// Resposta do serviço do núcleo (P5.1).
    RespostaNucleo(crate::nucleo::Resposta),
    SyncAudioSnapshots {
        deck1_pos_s: f64,
        deck1_playing: bool,
        deck1_cue: bool,
        deck2_pos_s: f64,
        deck2_playing: bool,
        deck2_cue: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use vilelabox_anlz::Beat;

    fn sample_track(title: &str, bpm: f64, beats_count: u16) -> Track {
        let beats: Vec<Beat> = (0..beats_count)
            .map(|i| Beat {
                beat_number: (i % 4) + 1,
                bpm,
                time_ms: u32::from(i) * 500,
            })
            .collect();
        Track {
            title: title.to_string(),
            dat: None,
            path: None,
            audio_ms: None,
            meta: None,
            ext: None,
            grid: Some(Grid {
                bpm,
                variable: false,
                first_beat_time_s: 0.0,
                downbeat_index: 0,
                beats,
            }),
        }
    }

    #[test]
    fn app_state_inicializa_vazio() {
        let state = AppState::new();
        assert!(state.tracks.is_empty());
        assert!(state.filtered_indices.is_empty());
        assert_eq!(state.selected_filtered_index, None);
        assert_eq!(state.deck1, DeckState::empty());
        assert_eq!(state.deck2, DeckState::empty());
        assert!(!state.deck1.audio_ready);
        assert!(!state.deck2.audio_ready);
    }

    #[test]
    fn carga_de_faixas_mantem_ambos_decks_vazios_ate_carga_explicita() {
        let mut state = AppState::new();
        let tracks = vec![
            sample_track("Artista A - Track 1.mp3", 124.0, 16),
            sample_track("Artista B - Track 2.mp3", 128.0, 32),
        ];

        state.apply(Command::ScanFinished {
            tracks,
            summary: "2 faixas".into(),
        });

        assert_eq!(state.tracks.len(), 2);
        assert_eq!(state.filtered_indices.len(), 2);
        assert_eq!(state.selected_filtered_index, Some(0));

        // Ambos os decks permanecem vazios (Not Loaded.)
        assert_eq!(state.deck1, DeckState::empty());
        assert_eq!(state.deck2, DeckState::empty());

        // Carrega faixa 0 no Deck 1
        state.apply(Command::LoadToDeck {
            deck: DeckId::Deck1,
            filtered_index: 0,
        });
        assert_eq!(state.deck1.loaded_track_idx, Some(0));
        assert_eq!(state.deck1.title, "Track 1");
        assert_eq!(state.deck1.artist, "Artista A");
        assert_eq!(state.deck1.bpm, "124.00");
        assert_eq!(state.deck2, DeckState::empty());
    }

    #[test]
    fn selecao_de_faixa_nao_altera_decks_ate_comando_load() {
        let mut state = AppState::new();
        state.apply(Command::ScanFinished {
            tracks: vec![
                sample_track("Artista A - Track 1.mp3", 124.0, 16),
                sample_track("Artista B - Track 2.mp3", 128.0, 32),
            ],
            summary: "2 faixas".into(),
        });

        // Carrega Track 1 no Deck 1 explicitamente
        state.apply(Command::LoadToDeck {
            deck: DeckId::Deck1,
            filtered_index: 0,
        });
        assert_eq!(state.deck1.title, "Track 1");

        // Seleciona a faixa 1 (Track 2)
        state.apply(Command::SelectTrack(1));
        assert_eq!(state.selected_filtered_index, Some(1));

        // Deck 1 ainda deve estar com a Track 1
        assert_eq!(state.deck1.title, "Track 1");
        // Deck 2 continua vazio
        assert_eq!(state.deck2.loaded_track_idx, None);

        // Agora carrega a faixa selecionada no Deck 2
        state.apply(Command::LoadSelectedToDeck(DeckId::Deck2));
        assert_eq!(state.deck2.loaded_track_idx, Some(1));
        assert_eq!(state.deck2.title, "Track 2");
        assert_eq!(state.deck2.artist, "Artista B");
        assert_eq!(state.deck2.bpm, "128.00");

        // Os dois decks agora mostram faixas diferentes!
        assert_ne!(state.deck1.title, state.deck2.title);
    }

    #[test]
    fn busca_filtra_por_titulo_e_artista() {
        let mut state = AppState::new();
        state.apply(Command::ScanFinished {
            tracks: vec![
                sample_track("Daft Punk - One More Time.mp3", 123.0, 16),
                sample_track("Deadmau5 - Strobe.mp3", 128.0, 32),
                sample_track("Daft Punk - Around The World.mp3", 121.0, 16),
            ],
            summary: "3 faixas".into(),
        });

        assert_eq!(state.filtered_indices.len(), 3);

        // Busca por artista
        state.apply(Command::Search("daft".into()));
        assert_eq!(state.filtered_indices.len(), 2);
        assert_eq!(state.filtered_indices, vec![0, 2]);

        // Busca por título
        state.apply(Command::Search("strobe".into()));
        assert_eq!(state.filtered_indices.len(), 1);
        assert_eq!(state.filtered_indices, vec![1]);

        // Busca sem resultado
        state.apply(Command::Search("inexistente".into()));
        assert_eq!(state.filtered_indices.len(), 0);
        assert_eq!(state.selected_filtered_index, None);

        // Limpeza da busca restaura lista completa
        state.apply(Command::Search(String::new()));
        assert_eq!(state.filtered_indices.len(), 3);
    }

    #[test]
    fn ordenacao_por_coluna_e_inversao_de_direcao() {
        let mut state = AppState::new();
        state.apply(Command::ScanFinished {
            tracks: vec![
                sample_track("Zeta - B Track.mp3", 130.0, 16),
                sample_track("Alpha - A Track.mp3", 120.0, 16),
                sample_track("Beta - C Track.mp3", 125.0, 16),
            ],
            summary: "3 faixas".into(),
        });

        // Ordena por BPM crescente
        state.apply(Command::SortBy(SortColumn::Bpm));
        assert_eq!(state.sort_column, SortColumn::Bpm);
        assert_eq!(state.sort_direction, SortDirection::Ascending);
        assert_eq!(state.filtered_indices, vec![1, 2, 0]); // 120, 125, 130

        // Clica na mesma coluna: inverte para decrescente
        state.apply(Command::SortBy(SortColumn::Bpm));
        assert_eq!(state.sort_direction, SortDirection::Descending);
        assert_eq!(state.filtered_indices, vec![0, 2, 1]); // 130, 125, 120

        // Clica em Artista: ordena por Artista crescente
        state.apply(Command::SortBy(SortColumn::Artist));
        assert_eq!(state.sort_column, SortColumn::Artist);
        assert_eq!(state.sort_direction, SortDirection::Ascending);
        assert_eq!(state.filtered_indices, vec![1, 2, 0]); // Alpha, Beta, Zeta
    }

    #[test]
    fn atualizacao_do_relogio() {
        let mut state = AppState::new();
        assert_eq!(state.clock_text, "--:--");
        state.apply(Command::TickClock("14:35".into()));
        assert_eq!(state.clock_text, "14:35");
    }

    #[test]
    fn play_pause_cue_e_seek_no_deck() {
        let mut state = AppState::new();
        state.apply(Command::ScanFinished {
            tracks: vec![sample_track("Artista A - Track 1.mp3", 124.0, 16)],
            summary: "1 faixa".into(),
        });
        state.apply(Command::SetDeckAudioReady {
            deck: DeckId::Deck1,
            ready: true,
        });

        assert_eq!(state.deck1.pos_s, 0.0);
        assert!(!state.deck1.is_playing);

        // Play
        state.apply(Command::PlayPause(DeckId::Deck1));
        assert!(state.deck1.is_playing);

        // Seek para 50%
        state.apply(Command::Seek(DeckId::Deck1, 0.5));
        assert!((state.deck1.pos_s - state.deck1.dur_s * 0.5).abs() < 0.01);

        // Sincronização de snapshot
        state.apply(Command::SyncAudioSnapshots {
            deck1_pos_s: 1.25,
            deck1_playing: true,
            deck1_cue: false,
            deck2_pos_s: 0.0,
            deck2_playing: false,
            deck2_cue: true,
        });
        assert_eq!(state.deck1.pos_s, 1.25);
        assert!(state.deck1.is_playing);
        assert_eq!(state.deck1.tempo_decorrido, "00:01.2");
    }

    #[test]
    fn cues_gravados_voltam_ao_recarregar_a_faixa() {
        let dir = std::env::temp_dir().join(format!("vilelabox-cues-state-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut st = AppState::new();
        st.cues_dir = Some(dir.clone());
        let mut t = sample_track("A", 120.0, 8);
        t.path = Some("/musica/a.mp3".into());
        st.tracks = vec![t];
        st.filtered_indices = vec![0];
        let carregar = |st: &mut AppState| {
            st.apply(Command::LoadToDeck {
                deck: DeckId::Deck1,
                filtered_index: 0,
            });
        };
        carregar(&mut st);
        crate::sync::gravar_memory_cue(&mut st.deck1.cues, 3.25);
        st.salvar_cues(0);
        crate::sync::gravar_hot_cue(&mut st.deck1.cues, 2, 1.5);
        st.salvar_cues(0);
        st.deck1 = DeckState::empty();
        carregar(&mut st);
        let v: Vec<_> = st
            .deck1
            .cues
            .iter()
            .map(|c| (c.hot_cue, c.start_time_s))
            .collect();
        assert_eq!(v, [(0, 3.25), (2, 1.5)]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Atende todos os pedidos pendentes na ponte e devolve as respostas ao
    /// estado, como o serviço e o event loop fazem no app.
    fn processar(
        st: &mut AppState,
        rx: &std::sync::mpsc::Receiver<crate::nucleo::Pedido>,
        ponte: &mut crate::nucleo::Ponte,
    ) {
        while let Ok(p) = rx.try_recv() {
            for r in ponte.atender(p) {
                st.apply(Command::RespostaNucleo(r));
            }
        }
    }

    #[test]
    fn leitura_atrasada_preserva_edicoes_e_nova_carga_restaura_banco() {
        let mut st = AppState::new();
        st.tracks = vec![sample_track("A", 120.0, 8)];
        st.filtered_indices = vec![0];
        let carregar = |st: &mut AppState| {
            st.apply(Command::LoadToDeck {
                deck: DeckId::Deck1,
                filtered_index: 0,
            })
        };
        carregar(&mut st);
        let mut cues_antigos = Vec::new();
        crate::sync::gravar_memory_cue(&mut cues_antigos, 1.0);
        let grade_antiga = st.deck1.beats.clone();
        let resposta = crate::nucleo::Resposta::Preparacao {
            deck: 0,
            faixa: 0,
            prep: crate::nucleo::Preparacao {
                cues: cues_antigos.clone(),
                grade: Some((grade_antiga.clone(), true)),
            },
            apos_audio: false,
        };
        crate::sync::gravar_memory_cue(&mut st.deck1.cues, 2.0);
        st.salvar_cues(0);
        assert!(st.edit_beatgrid(0, GridAction::ShiftMs(3), 0.0));
        let cues_editados = st.deck1.cues.clone();
        let grade_editada = st.deck1.beats.clone();
        st.apply(Command::RespostaNucleo(resposta.clone()));
        assert_eq!(st.deck1.cues, cues_editados);
        assert_eq!(st.deck1.beats, grade_editada);
        assert!(!st.grid_edit[0].locked);
        carregar(&mut st);
        st.apply(Command::RespostaNucleo(resposta));
        assert_eq!(st.deck1.cues, cues_antigos);
        assert_eq!(st.deck1.beats, grade_antiga);
        assert!(st.grid_edit[0].locked);
    }

    #[test]
    fn banco_proprio_migra_o_json_e_restaura_sem_ele() {
        let dir =
            std::env::temp_dir().join(format!("vilelabox-nucleo-state-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut ponte = crate::nucleo::Ponte::em_memoria();
        let (tx, rx) = std::sync::mpsc::channel();
        let novo = |dir: Option<std::path::PathBuf>, ponte: &mut crate::nucleo::Ponte| {
            let mut st = AppState::new();
            st.cues_dir = dir;
            st.nucleo = Some(tx.clone());
            let mut t = sample_track("A", 120.0, 8);
            t.path = Some("/musica/a.mp3".into());
            st.tracks = vec![t];
            st.filtered_indices = vec![0];
            st.apply(Command::LoadToDeck {
                deck: DeckId::Deck1,
                filtered_index: 0,
            });
            processar(&mut st, &rx, ponte);
            st
        };
        // Sem taxa conhecida, o banco recusa e o cue vai para o JSON.
        let mut st = novo(Some(dir.clone()), &mut ponte);
        crate::sync::gravar_hot_cue(&mut st.deck1.cues, 2, 1.5);
        crate::sync::gravar_memory_cue(&mut st.deck1.cues, 3.25);
        st.salvar_cues(0);
        processar(&mut st, &rx, &mut ponte);
        assert!(!crate::cues_store::ler(&dir, "/musica/a.mp3").is_empty());
        // Áudio decodificado: identidade gravada e o JSON migra para o banco.
        st.audio_pronto(0, 44100, 2, 44100 * 60);
        processar(&mut st, &rx, &mut ponte);
        // Sem a pasta de JSON, o banco devolve os cues (ordenados por posição).
        let st2 = novo(None, &mut ponte);
        let v: Vec<_> = st2
            .deck1
            .cues
            .iter()
            .filter(|c| !c.is_loop)
            .map(|c| (c.hot_cue, c.start_time_s))
            .collect();
        assert_eq!(v, [(2, 1.5), (0, 3.25)]);
        // Resposta de faixa que já saiu do deck é descartada.
        let mut st3 = novo(None, &mut ponte);
        st3.deck1 = DeckState::empty();
        st3.apply(Command::RespostaNucleo(
            crate::nucleo::Resposta::CuesRecusados { deck: 0, faixa: 0 },
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
