use crate::decode::AudioBuffer;
use std::sync::Arc;

/// Estados discretos de transporte de um deck, mapeados a partir de `spec/rekordbox/30-deck.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportState {
    /// Pausado ou parado na posição atual.
    Stopped,
    /// Reproduzindo continuamente.
    Playing,
    /// Pausado exatamente no ponto de CUE ativo, pronto para disparo.
    CueStandby,
    /// Reproduzindo temporariamente enquanto o botão CUE é mantido pressionado.
    CuePreview,
}

/// Instantâneo imutável do transporte de um deck para apresentação na interface.
#[derive(Debug, Clone, PartialEq)]
pub struct DeckSnapshot {
    pub track_generation: u64,
    pub position_frames: u64,
    pub total_frames: u64,
    pub sample_rate: u32,
    pub state: TransportState,
    pub cue_frame: u64,
    pub loop_range: Option<(u64, u64)>,
    pub loop_active: bool,
    pub master_tempo: bool,
    pub beat_sync: bool,
    pub slip: bool,
    pub quantize: bool,
    pub key_shift: i8,
    pub speed_ratio: f64,
    pub position_seconds: f64,
    pub duration_seconds: f64,
    pub is_playing: bool,
    pub is_cue_standby: bool,
}

impl Default for DeckSnapshot {
    fn default() -> Self {
        Self {
            track_generation: 0,
            position_frames: 0,
            total_frames: 0,
            sample_rate: 44100,
            state: TransportState::Stopped,
            cue_frame: 0,
            loop_range: None,
            loop_active: false,
            master_tempo: true,
            beat_sync: false,
            slip: false,
            quantize: false,
            key_shift: 0,
            speed_ratio: 1.0,
            position_seconds: 0.0,
            duration_seconds: 0.0,
            is_playing: false,
            is_cue_standby: false,
        }
    }
}

/// Motor de transporte de um deck individual.
#[derive(Debug, Clone)]
pub struct Deck {
    pub audio: Option<Arc<AudioBuffer>>,
    pub state: TransportState,
    pub position_frames: u64,
    pub cue_frame: u64,
    pub loop_range: Option<(u64, u64)>,
    pub loop_active: bool,
    pub speed_ratio: f64,
    /// Botões do deck observados em `evidence/p6-2-sync-slip-quantize.md`
    /// (spec/rekordbox/30-deck.md e 32-loops-cues.md). Só estado: o efeito
    /// no som de MT, SLIP, Q e do deslocamento de tom não foi observado.
    pub master_tempo: bool,
    pub beat_sync: bool,
    pub slip: bool,
    pub quantize: bool,
    /// Deslocamento de tom em semitons pedido pelo KEY SYNC.
    pub key_shift: i8,
    pub generation: u64,
    /// Taxa da saída de áudio; 0 = igual à do arquivo (sem conversão).
    pub out_rate: u32,
    /// Fração de frame entre `position_frames` e o seguinte (interpolação).
    frac: f64,
    /// SLIP (p6-6): posição em sombra, em frames, que segue andando durante
    /// um loop automático aberto com SLIP ligado; a saída do loop vai para ela.
    sombra: Option<f64>,
    metronome_level: u8,
    metronome_beats: Vec<(u64, u16)>,
    metronome_next: usize,
    metronome_click_left: u32,
    metronome_click_len: u32,
    metronome_accent: bool,
    scratch_active: bool,
    scratch_target_frame: f64,
    scratch_resume_state: TransportState,
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}

impl Deck {
    #[must_use]
    pub fn new() -> Self {
        Self {
            audio: None,
            state: TransportState::Stopped,
            position_frames: 0,
            cue_frame: 0,
            loop_range: None,
            loop_active: false,
            speed_ratio: 1.0,
            // MT aceso na tela medida (deck.slint); p6-2 só observou a alternância.
            master_tempo: true,
            beat_sync: false,
            slip: false,
            quantize: false,
            key_shift: 0,
            generation: 0,
            out_rate: 0,
            frac: 0.0,
            sombra: None,
            metronome_level: 0,
            metronome_beats: Vec::new(),
            metronome_next: 0,
            metronome_click_left: 0,
            metronome_click_len: 0,
            metronome_accent: false,
            scratch_active: false,
            scratch_target_frame: 0.0,
            scratch_resume_state: TransportState::Stopped,
        }
    }

    /// Carrega um buffer de áudio no deck, posicionando no início (ou no CUE inicial).
    pub fn load_audio(&mut self, audio: Arc<AudioBuffer>, initial_cue: Option<u64>) {
        self.generation = self.generation.wrapping_add(1);
        let cue = initial_cue.unwrap_or(0).min(audio.total_frames);
        self.audio = Some(audio);
        self.cue_frame = cue;
        self.position_frames = cue;
        self.state = TransportState::CueStandby;
        self.loop_range = None;
        self.loop_active = false;
        self.sombra = None;
        self.scratch_active = false;
        self.scratch_target_frame = cue as f64;
        self.reset_metronome_cursor();
    }

    /// Descarrega o deck e redefine os parâmetros.
    pub fn unload(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.audio = None;
        self.position_frames = 0;
        self.cue_frame = 0;
        self.state = TransportState::Stopped;
        self.loop_range = None;
        self.loop_active = false;
        self.scratch_active = false;
    }

    /// Inicia a reprodução (`Playing`).
    pub fn play(&mut self) {
        if self.audio.is_some() {
            self.state = TransportState::Playing;
        }
    }

    /// Pausa a reprodução no frame atual (`Stopped`).
    pub fn pause(&mut self) {
        if self.state == TransportState::Playing || self.state == TransportState::CuePreview {
            self.state = TransportState::Stopped;
        }
    }

    /// Alterna entre reprodução e pausa.
    pub fn play_pause(&mut self) {
        match self.state {
            TransportState::Playing => self.pause(),
            TransportState::Stopped | TransportState::CueStandby | TransportState::CuePreview => {
                self.play();
            }
        }
    }

    /// Lógica do botão CUE pressionado (down):
    /// - Se estiver tocando (`Playing`): para imediatamente, salta para o `cue_frame` e entra em `CueStandby`.
    /// - Se estiver em `Stopped` fora do CUE: define `cue_frame` na posição atual e entra em `CueStandby`.
    /// - Se já estiver em `CueStandby`: entra em `CuePreview` e toca enquanto mantido.
    pub fn cue_down(&mut self) {
        if self.audio.is_none() {
            return;
        }
        match self.state {
            TransportState::Playing => {
                self.position_frames = self.cue_frame;
                self.state = TransportState::CueStandby;
            }
            TransportState::Stopped => {
                self.cue_frame = self.position_frames;
                self.state = TransportState::CueStandby;
            }
            TransportState::CueStandby => {
                self.state = TransportState::CuePreview;
            }
            TransportState::CuePreview => {}
        }
    }

    /// Lógica do botão CUE liberado (up):
    /// - Se estiver em `CuePreview`: retorna a agulha para o `cue_frame` e entra em `CueStandby`.
    pub fn cue_up(&mut self) {
        if self.state == TransportState::CuePreview {
            self.position_frames = self.cue_frame;
            self.state = TransportState::CueStandby;
        }
    }

    /// Disparo único de CUE (clique instantâneo): se tocando volta ao CUE; se parado define CUE.
    pub fn cue_press(&mut self) {
        if self.state == TransportState::Playing {
            self.position_frames = self.cue_frame;
            self.state = TransportState::CueStandby;
        } else {
            self.cue_frame = self.position_frames;
            self.state = TransportState::CueStandby;
        }
    }

    /// Salto direto para um frame específico com limitação aos limites da faixa.
    pub fn seek_frame(&mut self, frame: u64) {
        let total = self.audio.as_ref().map_or(0, |a| a.total_frames);
        let clamped = frame.min(total);
        self.position_frames = clamped;
        self.frac = 0.0;
        self.reset_metronome_cursor();
        if self.state == TransportState::CueStandby && clamped != self.cue_frame {
            self.state = TransportState::Stopped;
        }
    }

    /// Salto para tempo em segundos.
    pub fn seek_seconds(&mut self, seconds: f64) {
        if let Some(audio) = &self.audio {
            let frame = (seconds.max(0.0) * f64::from(audio.sample_rate)).round() as u64;
            self.seek_frame(frame);
        }
    }

    /// Segura virtualmente a mídia. O transporte passa a responder à mão,
    /// inclusive produzindo áudio reverso quando o alvo recua.
    pub fn scratch_begin(&mut self) {
        if self.audio.is_none() || self.scratch_active {
            return;
        }
        self.scratch_resume_state = self.state;
        self.scratch_target_frame = self.position_frames as f64 + self.frac;
        self.scratch_active = true;
        self.metronome_click_left = 0;
    }

    pub fn scratch_to(&mut self, seconds: f64) {
        let Some(audio) = &self.audio else { return };
        if !self.scratch_active || !seconds.is_finite() {
            return;
        }
        self.scratch_target_frame = (seconds.max(0.0) * f64::from(audio.sample_rate))
            .clamp(0.0, audio.total_frames.saturating_sub(1) as f64);
    }

    pub fn scratch_end(&mut self) {
        if !self.scratch_active {
            return;
        }
        self.scratch_active = false;
        self.state = self.scratch_resume_state;
        self.reset_metronome_cursor();
    }

    /// Define e ativa um intervalo de loop em frames `[start, end)`.
    pub fn set_loop(&mut self, start: u64, end: u64) {
        if end > start {
            self.loop_range = Some((start, end));
            self.loop_active = true;
        }
    }

    /// Alterna a ativação do loop ativo.
    pub fn toggle_loop(&mut self) {
        if self.loop_range.is_some() {
            self.loop_active = !self.loop_active;
        }
    }

    /// Lançamento de hot cue: posiciona no ponto e toca, parado ou tocando
    /// (spec/rekordbox/32-loops-cues.md).
    pub fn jump_and_play(&mut self, frame: u64) {
        if self.audio.is_none() {
            return;
        }
        let total = self.audio.as_ref().map_or(0, |a| a.total_frames);
        self.position_frames = frame.min(total);
        self.frac = 0.0;
        self.reset_metronome_cursor();
        self.state = TransportState::Playing;
    }

    /// IN do loop manual (p6-5): o ponto de cue passa a ser `frame`; o
    /// transporte segue como está.
    pub fn set_cue_point(&mut self, frame: u64) {
        let total = self.audio.as_ref().map_or(0, |a| a.total_frames);
        self.cue_frame = frame.min(total);
    }

    /// Define o ponto de cue e fica parado nele (chamada de memory cue).
    pub fn set_cue_and_standby(&mut self, frame: u64) {
        if self.audio.is_none() {
            return;
        }
        let total = self.audio.as_ref().map_or(0, |a| a.total_frames);
        self.cue_frame = frame.min(total);
        self.position_frames = self.cue_frame;
        self.frac = 0.0;
        self.state = TransportState::CueStandby;
    }

    /// MT (unit#15, p6-2): alterna o estado. Efeito no tom sob ajuste de
    /// andamento é DESCONHECIDO (o fader não foi movido).
    pub fn set_master_tempo(&mut self, on: bool) {
        self.master_tempo = on;
    }

    /// SLIP (unit#14, p6-2; sombra em p6-6). Desligar no meio de um loop:
    /// DESCONHECIDO; descarta a sombra (a saída do loop não salta).
    pub fn set_slip(&mut self, on: bool) {
        self.slip = on;
        if !on {
            self.sombra = None;
        }
    }

    /// Q (unit#9, p6-2): troca o objeto de comportamento; os argumentos de
    /// `reqAutoBeatLoop` não mudam. O arredondamento (para baixo) é feito no app.
    pub fn set_quantize(&mut self, on: bool) {
        self.quantize = on;
    }

    /// Metrônomo audível sincronizado aos nós da beatgrid. O compasso 1 recebe
    /// acento; os níveis 1..3 seguem o seletor de volume do painel.
    pub fn set_metronome(&mut self, level: u8, beats: Vec<(u32, u16)>) {
        self.metronome_level = level.min(3);
        let sample_rate = self.audio.as_ref().map_or(44_100, |a| a.sample_rate);
        self.metronome_beats = beats
            .into_iter()
            .map(|(ms, number)| {
                (
                    ((u64::from(ms) * u64::from(sample_rate)) + 500) / 1000,
                    number,
                )
            })
            .collect();
        self.metronome_click_left = 0;
        self.reset_metronome_cursor();
    }

    fn reset_metronome_cursor(&mut self) {
        self.metronome_next = self
            .metronome_beats
            .partition_point(|(frame, _)| *frame < self.position_frames);
    }

    /// BEAT SYNC (unit#22, p6-2). `master_bpm = None`: este deck é o master e
    /// nada muda além do estado (observado). Não master: assume o andamento do
    /// master (observado 130,00 -> 130,14 BPM). Desligar mantém o andamento atual
    /// (o que o rekordbox faz ao desligar não foi observado).
    pub fn set_beat_sync(&mut self, on: bool, own_bpm: f64, master_bpm: Option<f64>) {
        self.beat_sync = on;
        if let (true, Some(m)) = (on, master_bpm) {
            if own_bpm > 0.0 && m > 0.0 {
                self.speed_ratio = m / own_bpm;
            }
        }
    }

    /// KEY SYNC (unit#45, p6-2): guarda o deslocamento de tom. Sem DSP de tom
    /// no motor ainda.
    // ponytail: só estado; aplicar no som quando houver pitch shifter.
    pub fn set_key_shift(&mut self, semitones: i8) {
        self.key_shift = semitones;
    }

    /// Loop automático de `len` frames a partir da posição atual. Com Q ligado
    /// o tamanho chega igual (p6-2).
    // Quantização (regra para baixo, `0x142ab7130`, p6-2): o motor não conhece
    // a grade de batidas. Quem quantiza é o app (`AppState.beats`), que passa o
    // início já ajustado por `auto_loop_at`; aqui a posição entra intacta.
    pub fn auto_loop(&mut self, len: u64) {
        self.auto_loop_at(self.position_frames, len);
    }

    /// Loop automático de `len` frames a partir de `start` (já quantizado pelo app).
    pub fn auto_loop_at(&mut self, start: u64, len: u64) {
        self.set_loop(start, start + len.max(1));
        // SLIP ligado (p6-6): a reprodução segue em sombra a partir daqui.
        if self.slip && self.loop_active {
            self.sombra = Some(self.position_frames as f64 + self.frac);
        }
    }

    /// Multiplica o tamanho do loop por `num/den` mantendo o início (½ e ×2).
    pub fn loop_cut(&mut self, num: u64, den: u64) {
        if let Some((start, end)) = self.loop_range {
            if let Some(len) = ((end - start) * num).checked_div(den) {
                self.loop_range = Some((start, start + len.max(1)));
            }
        }
    }

    /// Remove o loop. Com sombra do SLIP, a reprodução vai para onde estaria
    /// sem o loop (p6-6: saiu em 20,17 s após 7 s de loop iniciado em 13,31 s).
    pub fn clear_loop(&mut self) {
        self.loop_range = None;
        self.loop_active = false;
        if let Some(s) = self.sombra.take() {
            let total = self.audio.as_ref().map_or(0, |a| a.total_frames);
            self.position_frames = (s.floor() as u64).min(total);
            self.frac = s.fract();
        }
    }

    /// Processa um bloco de áudio estéreo, preenchendo os buffers de saída `(L, R)`.
    ///
    /// Garante execução em tempo real determinística sem alocações ou chamadas bloqueantes.
    pub fn process_block(&mut self, out_l: &mut [f32], out_r: &mut [f32]) {
        let n_frames = out_l.len().min(out_r.len());
        let Some(audio) = &self.audio else {
            out_l[..n_frames].fill(0.0);
            out_r[..n_frames].fill(0.0);
            return;
        };

        let is_running = self.scratch_active
            || self.state == TransportState::Playing
            || self.state == TransportState::CuePreview;

        if !is_running {
            out_l[..n_frames].fill(0.0);
            out_r[..n_frames].fill(0.0);
            return;
        }

        let total = audio.total_frames;
        // Arquivo a 44,1 kHz numa saída a 48 kHz: avança 0,919 frame por amostra.
        let step = if self.out_rate > 0 {
            self.speed_ratio * f64::from(audio.sample_rate) / f64::from(self.out_rate)
        } else {
            self.speed_ratio
        };
        // Percorrer o caminho durante o bloco produz o gesto audível, em vez
        // de uma sequência de saltos mudos. O sinal negativo é o backspin.
        let scratch_step = if self.scratch_active {
            let here = self.position_frames as f64 + self.frac;
            ((self.scratch_target_frame - here) / n_frames.max(1) as f64).clamp(-64.0, 64.0)
        } else {
            0.0
        };
        for i in 0..n_frames {
            // Verificar loop
            if !self.scratch_active && self.loop_active {
                if let Some((loop_start, loop_end)) = self.loop_range {
                    if self.position_frames >= loop_end {
                        self.position_frames = loop_start;
                        self.metronome_next = self
                            .metronome_beats
                            .partition_point(|(frame, _)| *frame < self.position_frames);
                    }
                }
            }

            // Verificar fim da faixa
            if self.position_frames >= total {
                if self.scratch_active && total > 0 {
                    self.position_frames = total - 1;
                    self.frac = 0.0;
                } else {
                    self.position_frames = total;
                    self.state = TransportState::Stopped;
                    out_l[i..n_frames].fill(0.0);
                    out_r[i..n_frames].fill(0.0);
                    break;
                }
            }

            // Um vinil segurado não repete uma amostra DC enquanto o mouse
            // está imóvel; ele fica silencioso até surgir novo movimento.
            if self.scratch_active && scratch_step.abs() < 1e-6 {
                out_l[i] = 0.0;
                out_r[i] = 0.0;
                continue;
            }

            // ponytail: interpolação linear; trocar por sinc/polifásico se o aliasing incomodar.
            let (l0, r0) = audio.get_frame(self.position_frames).unwrap_or((0.0, 0.0));
            let (l1, r1) = audio
                .get_frame(self.position_frames + 1)
                .unwrap_or((l0, r0));
            let f = self.frac as f32;
            out_l[i] = l0 + (l1 - l0) * f;
            out_r[i] = r0 + (r1 - r0) * f;
            if !self.scratch_active && self.metronome_level > 0 {
                while self
                    .metronome_beats
                    .get(self.metronome_next)
                    .is_some_and(|(frame, _)| *frame <= self.position_frames)
                {
                    self.metronome_accent = self.metronome_beats[self.metronome_next].1 == 1;
                    let click_rate = if self.out_rate > 0 {
                        self.out_rate
                    } else {
                        audio.sample_rate
                    };
                    self.metronome_click_len = (click_rate / 35).max(1);
                    self.metronome_click_left = self.metronome_click_len;
                    self.metronome_next += 1;
                }
                if self.metronome_click_left > 0 {
                    let elapsed = self.metronome_click_len - self.metronome_click_left;
                    let frequency = if self.metronome_accent {
                        1760.0
                    } else {
                        1100.0
                    };
                    let click_rate = if self.out_rate > 0 {
                        self.out_rate
                    } else {
                        audio.sample_rate
                    };
                    let phase =
                        std::f32::consts::TAU * frequency * elapsed as f32 / click_rate as f32;
                    let envelope =
                        self.metronome_click_left as f32 / self.metronome_click_len as f32;
                    let gain = [0.0, 0.08, 0.16, 0.26][usize::from(self.metronome_level)];
                    let accent = if self.metronome_accent { 1.25 } else { 1.0 };
                    let click = phase.sin() * envelope * gain * accent;
                    out_l[i] = (out_l[i] + click).clamp(-1.0, 1.0);
                    out_r[i] = (out_r[i] + click).clamp(-1.0, 1.0);
                    self.metronome_click_left -= 1;
                }
            }
            if self.scratch_active {
                let next = (self.position_frames as f64 + self.frac + scratch_step)
                    .clamp(0.0, total.saturating_sub(1) as f64);
                self.position_frames = next.floor() as u64;
                self.frac = next.fract();
            } else {
                self.frac += step;
                let avanco = self.frac.floor();
                self.frac -= avanco;
                self.position_frames += avanco as u64;
                if let Some(s) = self.sombra.as_mut() {
                    *s = (*s + step).min(total as f64);
                }
            }
        }
    }

    /// Extrai o snapshot imutável para a interface.
    #[must_use]
    pub fn snapshot(&self) -> DeckSnapshot {
        let (total_frames, sample_rate, duration_seconds) = match &self.audio {
            Some(a) => (a.total_frames, a.sample_rate, a.duration_seconds()),
            None => (0, 44100, 0.0),
        };

        let position_seconds = if sample_rate > 0 {
            self.position_frames as f64 / f64::from(sample_rate)
        } else {
            0.0
        };

        DeckSnapshot {
            track_generation: self.generation,
            position_frames: self.position_frames,
            total_frames,
            sample_rate,
            state: self.state,
            cue_frame: self.cue_frame,
            loop_range: self.loop_range,
            loop_active: self.loop_active,
            master_tempo: self.master_tempo,
            beat_sync: self.beat_sync,
            slip: self.slip,
            quantize: self.quantize,
            key_shift: self.key_shift,
            speed_ratio: self.speed_ratio,
            position_seconds,
            duration_seconds,
            is_playing: self.state == TransportState::Playing,
            is_cue_standby: self.state == TransportState::CueStandby,
        }
    }
}
