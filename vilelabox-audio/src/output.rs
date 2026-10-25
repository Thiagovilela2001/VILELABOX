use crate::deck::{Deck, DeckSnapshot};
use crate::decode::AudioBuffer;
use crate::error::AudioError;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Comandos em tempo real despachados da interface ou do núcleo para o motor de áudio.
#[derive(Debug)]
pub enum AudioCommand {
    LoadTrack {
        deck: u8,
        audio: Arc<AudioBuffer>,
        cue_frame: Option<u64>,
    },
    PlayPause {
        deck: u8,
    },
    Play {
        deck: u8,
    },
    Pause {
        deck: u8,
    },
    CueDown {
        deck: u8,
    },
    CueUp {
        deck: u8,
    },
    CuePress {
        deck: u8,
    },
    SeekSeconds {
        deck: u8,
        seconds: f64,
    },
    SeekFrame {
        deck: u8,
        frame: u64,
    },
    SetLoop {
        deck: u8,
        start_frame: u64,
        end_frame: u64,
    },
    ToggleLoop {
        deck: u8,
    },
    ClearLoop {
        deck: u8,
    },
    /// Hot cue: posiciona e toca.
    JumpAndPlay {
        deck: u8,
        frame: u64,
    },
    /// Ponto de cue sem mexer no transporte (IN do loop manual, p6-5).
    SetCuePoint {
        deck: u8,
        frame: u64,
    },
    /// Memory cue: define o cue e para nele.
    SetCueAndStandby {
        deck: u8,
        frame: u64,
    },
    /// Loop de `len_frames` a partir de `start_frame` (quantizado pelo app com
    /// Q ligado) ou, com `None`, da posição atual.
    AutoLoop {
        deck: u8,
        len_frames: u64,
        start_frame: Option<u64>,
    },
    /// Tamanho do loop × `num/den`.
    LoopCut {
        deck: u8,
        num: u64,
        den: u64,
    },
    /// Botões de p6-2 (spec/rekordbox/30-deck.md, 32-loops-cues.md).
    SetMasterTempo { deck: u8, on: bool },
    SetSlip { deck: u8, on: bool },
    SetQuantize { deck: u8, on: bool },
    /// `master_bpm = None` quando o próprio deck é o master.
    BeatSync { deck: u8, on: bool, own_bpm: f64, master_bpm: Option<f64> },
    KeyShift { deck: u8, semitones: i8 },
}

/// Instantâneo consolidado de ambos os decks para consulta da UI.
#[derive(Debug, Clone, Default)]
pub struct EngineSnapshot {
    pub deck1: DeckSnapshot,
    pub deck2: DeckSnapshot,
    pub underruns: u64,
}

/// Estado compartilhado entre a thread de processamento de áudio e a UI.
pub struct SharedEngineState {
    pub snapshot: Mutex<EngineSnapshot>,
    pub underruns: AtomicU64,
    pub is_running: AtomicBool,
}

/// Handle para emissão de comandos e leitura de snapshots pelo aplicativo.
#[derive(Clone)]
pub struct AudioController {
    producer: Arc<Mutex<Producer<AudioCommand>>>,
    shared: Arc<SharedEngineState>,
}

impl AudioController {
    /// Envia um comando para a fila em tempo real (não bloqueante).
    pub fn send(&self, cmd: AudioCommand) -> bool {
        if let Ok(mut prod) = self.producer.lock() {
            prod.push(cmd).is_ok()
        } else {
            false
        }
    }

    /// Obtém o snapshot mais recente publicado pelo motor.
    #[must_use]
    pub fn get_snapshot(&self) -> EngineSnapshot {
        self.shared
            .snapshot
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default()
    }
}

/// Motor de mixagem e transporte executado dentro do callback de áudio.
pub struct AudioMixer {
    pub deck1: Deck,
    pub deck2: Deck,
    consumer: Consumer<AudioCommand>,
    shared: Arc<SharedEngineState>,
    buf_l: Vec<f32>,
    buf_r: Vec<f32>,
}

impl AudioMixer {
    #[must_use]
    pub fn new(consumer: Consumer<AudioCommand>, shared: Arc<SharedEngineState>) -> Self {
        const MAX_BLOCK: usize = 2048;
        Self {
            deck1: Deck::new(),
            deck2: Deck::new(),
            consumer,
            shared,
            buf_l: vec![0.0f32; MAX_BLOCK],
            buf_r: vec![0.0f32; MAX_BLOCK],
        }
    }

    fn deck_mut(&mut self, n: u8) -> Option<&mut Deck> {
        match n {
            1 => Some(&mut self.deck1),
            2 => Some(&mut self.deck2),
            _ => None,
        }
    }

    /// Mixer sem fila nem saída de hardware: comandos entram por `apply`
    /// (testes do app de ponta a ponta).
    #[must_use]
    pub fn offline() -> Self {
        let shared = Arc::new(SharedEngineState {
            snapshot: Mutex::new(EngineSnapshot::default()),
            underruns: AtomicU64::new(0),
            is_running: AtomicBool::new(false),
        });
        Self::new(RingBuffer::new(1).1, shared)
    }

    /// Processa comandos pendentes da fila sem travas nem alocações.
    pub fn drain_commands(&mut self) {
        while let Ok(cmd) = self.consumer.pop() {
            self.apply(cmd);
        }
    }

    /// Executa um comando no deck indicado.
    pub fn apply(&mut self, cmd: AudioCommand) {
        {
            match cmd {
                AudioCommand::LoadTrack {
                    deck,
                    audio,
                    cue_frame,
                } => match deck {
                    1 => self.deck1.load_audio(audio, cue_frame),
                    2 => self.deck2.load_audio(audio, cue_frame),
                    _ => {}
                },
                AudioCommand::PlayPause { deck } => match deck {
                    1 => self.deck1.play_pause(),
                    2 => self.deck2.play_pause(),
                    _ => {}
                },
                AudioCommand::Play { deck } => match deck {
                    1 => self.deck1.play(),
                    2 => self.deck2.play(),
                    _ => {}
                },
                AudioCommand::Pause { deck } => match deck {
                    1 => self.deck1.pause(),
                    2 => self.deck2.pause(),
                    _ => {}
                },
                AudioCommand::CueDown { deck } => match deck {
                    1 => self.deck1.cue_down(),
                    2 => self.deck2.cue_down(),
                    _ => {}
                },
                AudioCommand::CueUp { deck } => match deck {
                    1 => self.deck1.cue_up(),
                    2 => self.deck2.cue_up(),
                    _ => {}
                },
                AudioCommand::CuePress { deck } => match deck {
                    1 => self.deck1.cue_press(),
                    2 => self.deck2.cue_press(),
                    _ => {}
                },
                AudioCommand::SeekSeconds { deck, seconds } => match deck {
                    1 => self.deck1.seek_seconds(seconds),
                    2 => self.deck2.seek_seconds(seconds),
                    _ => {}
                },
                AudioCommand::SeekFrame { deck, frame } => match deck {
                    1 => self.deck1.seek_frame(frame),
                    2 => self.deck2.seek_frame(frame),
                    _ => {}
                },
                AudioCommand::SetLoop {
                    deck,
                    start_frame,
                    end_frame,
                } => match deck {
                    1 => self.deck1.set_loop(start_frame, end_frame),
                    2 => self.deck2.set_loop(start_frame, end_frame),
                    _ => {}
                },
                AudioCommand::ToggleLoop { deck } => match deck {
                    1 => self.deck1.toggle_loop(),
                    2 => self.deck2.toggle_loop(),
                    _ => {}
                },
                AudioCommand::ClearLoop { deck } => match deck {
                    1 => self.deck1.clear_loop(),
                    2 => self.deck2.clear_loop(),
                    _ => {}
                },
                AudioCommand::JumpAndPlay { deck, frame } => match deck {
                    1 => self.deck1.jump_and_play(frame),
                    2 => self.deck2.jump_and_play(frame),
                    _ => {}
                },
                AudioCommand::SetCuePoint { deck, frame } => match deck {
                    1 => self.deck1.set_cue_point(frame),
                    2 => self.deck2.set_cue_point(frame),
                    _ => {}
                },
                AudioCommand::SetCueAndStandby { deck, frame } => match deck {
                    1 => self.deck1.set_cue_and_standby(frame),
                    2 => self.deck2.set_cue_and_standby(frame),
                    _ => {}
                },
                AudioCommand::AutoLoop { deck, len_frames, start_frame } => {
                    if let Some(d) = self.deck_mut(deck) {
                        match start_frame {
                            Some(s) => d.auto_loop_at(s, len_frames),
                            None => d.auto_loop(len_frames),
                        }
                    }
                }
                AudioCommand::LoopCut { deck, num, den } => match deck {
                    1 => self.deck1.loop_cut(num, den),
                    2 => self.deck2.loop_cut(num, den),
                    _ => {}
                },
                AudioCommand::SetMasterTempo { deck, on } => {
                    if let Some(d) = self.deck_mut(deck) {
                        d.set_master_tempo(on);
                    }
                }
                AudioCommand::SetSlip { deck, on } => {
                    if let Some(d) = self.deck_mut(deck) {
                        d.set_slip(on);
                    }
                }
                AudioCommand::SetQuantize { deck, on } => {
                    if let Some(d) = self.deck_mut(deck) {
                        d.set_quantize(on);
                    }
                }
                AudioCommand::BeatSync { deck, on, own_bpm, master_bpm } => {
                    if let Some(d) = self.deck_mut(deck) {
                        d.set_beat_sync(on, own_bpm, master_bpm);
                    }
                }
                AudioCommand::KeyShift { deck, semitones } => {
                    if let Some(d) = self.deck_mut(deck) {
                        d.set_key_shift(semitones);
                    }
                }
            }
        }
    }

    /// Processa um bloco de saída estéreo intercalado `[L0, R0, L1, R1, ...]`.
    pub fn process_interleaved(&mut self, output: &mut [f32]) {
        self.drain_commands();

        let n_frames = output.len() / 2;
        if n_frames == 0 {
            return;
        }

        // Reajustar buffers auxiliares se o hardware pedir bloco maior que o padrão
        if self.buf_l.len() < n_frames {
            self.buf_l.resize(n_frames, 0.0);
            self.buf_r.resize(n_frames, 0.0);
        }

        // Processar Deck 1
        self.deck1
            .process_block(&mut self.buf_l[..n_frames], &mut self.buf_r[..n_frames]);
        for i in 0..n_frames {
            output[i * 2] = self.buf_l[i];
            output[i * 2 + 1] = self.buf_r[i];
        }

        // Somar Deck 2
        self.deck2
            .process_block(&mut self.buf_l[..n_frames], &mut self.buf_r[..n_frames]);
        for i in 0..n_frames {
            output[i * 2] += self.buf_l[i];
            output[i * 2 + 1] += self.buf_r[i];
        }

        // Atualizar snapshot compartilhado se o lock estiver livre
        if let Ok(mut snap) = self.shared.snapshot.try_lock() {
            snap.deck1 = self.deck1.snapshot();
            snap.deck2 = self.deck2.snapshot();
            snap.underruns = self.shared.underruns.load(Ordering::Relaxed);
        }
    }
}

/// Cria o sistema de áudio e inicializa a saída de hardware (WASAPI no Windows) ou fallback de emulação.
pub fn start_audio_engine() -> Result<(AudioController, Option<cpal::Stream>), AudioError> {
    const QUEUE_CAPACITY: usize = 256;
    let (producer, consumer) = RingBuffer::new(QUEUE_CAPACITY);

    let shared = Arc::new(SharedEngineState {
        snapshot: Mutex::new(EngineSnapshot::default()),
        underruns: AtomicU64::new(0),
        is_running: AtomicBool::new(true),
    });

    let controller = AudioController {
        producer: Arc::new(Mutex::new(producer)),
        shared: Arc::clone(&shared),
    };

    let mixer = Arc::new(Mutex::new(AudioMixer::new(consumer, Arc::clone(&shared))));

    // Tentar inicializar o dispositivo de hardware padrão via CPAL / WASAPI
    let host = cpal::default_host();
    let maybe_device = host.default_output_device();

    let stream = if let Some(device) = maybe_device {
        let config_res = device.default_output_config();
        match config_res {
            Ok(config) => {
                let err_fn = {
                    let shared = Arc::clone(&shared);
                    move |_err| {
                        shared.underruns.fetch_add(1, Ordering::Relaxed);
                    }
                };

                let stream_res = match config.sample_format() {
                    cpal::SampleFormat::F32 => {
                        let stream_config: cpal::StreamConfig = config.into();
                        let channels = stream_config.channels as usize;
                        // Sem isso, faixa a 44,1 kHz numa saída a 48 kHz toca rápida e aguda.
                        if let Ok(mut m) = mixer.lock() {
                            m.deck1.out_rate = stream_config.sample_rate.0;
                            m.deck2.out_rate = stream_config.sample_rate.0;
                        }
                        let mixer_cb = Arc::clone(&mixer);
                        let mut estereo = vec![0.0f32; 8192];
                        device.build_output_stream(
                            &stream_config,
                            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                                let Ok(mut m) = mixer_cb.try_lock() else {
                                    data.fill(0.0);
                                    return;
                                };
                                if channels == 2 {
                                    m.process_interleaved(data);
                                    return;
                                }
                                // Mono ou multicanal: mixa em estéreo e distribui
                                // (mono = média; canais além de L/R ficam mudos).
                                let frames = data.len() / channels.max(1);
                                if estereo.len() < frames * 2 {
                                    estereo.resize(frames * 2, 0.0);
                                }
                                let st = &mut estereo[..frames * 2];
                                m.process_interleaved(st);
                                for (f, out) in data.chunks_mut(channels.max(1)).enumerate() {
                                    let (l, r) = (st[f * 2], st[f * 2 + 1]);
                                    if out.len() == 1 {
                                        out[0] = 0.5 * (l + r);
                                    } else {
                                        out.fill(0.0);
                                        out[0] = l;
                                        out[1] = r;
                                    }
                                }
                            },
                            err_fn,
                            None,
                        )
                    }
                    _ => Err(cpal::BuildStreamError::DeviceNotAvailable),
                };

                match stream_res {
                    Ok(s) => {
                        let _ = s.play();
                        Some(s)
                    }
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };

    // Caso não haja dispositivo de áudio ou falhe a negociação, inicializa thread virtual
    // de bombeamento a 44.1 kHz em blocos de 512 amostras para manter relógio e transporte ativos.
    if stream.is_none() {
        let shared_pump = Arc::clone(&shared);
        let mixer_pump = Arc::clone(&mixer);
        std::thread::spawn(move || {
            let mut pump_buf = vec![0.0f32; 1024]; // 512 frames estéreo
            while shared_pump.is_running.load(Ordering::Relaxed) {
                if let Ok(mut m) = mixer_pump.lock() {
                    m.process_interleaved(&mut pump_buf);
                }
                // 512 amostras a 44100 Hz = aprox. 11.6 ms
                std::thread::sleep(Duration::from_millis(11));
            }
        });
    }

    Ok((controller, stream))
}
