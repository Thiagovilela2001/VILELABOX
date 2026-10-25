use crate::error::AudioError;
use std::fs::File;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Buffer de áudio estéreo em memória com amostras `f32` intercaladas no intervalo `[-1.0, 1.0]`.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioBuffer {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
    pub total_frames: u64,
}

impl AudioBuffer {
    #[must_use]
    pub fn new(sample_rate: u32, channels: u16, samples: Vec<f32>) -> Self {
        let total_frames = if channels > 0 {
            (samples.len() / usize::from(channels)) as u64
        } else {
            0
        };
        Self {
            sample_rate,
            channels,
            samples,
            total_frames,
        }
    }

    #[must_use]
    pub fn duration_seconds(&self) -> f64 {
        if self.sample_rate > 0 {
            self.total_frames as f64 / f64::from(self.sample_rate)
        } else {
            0.0
        }
    }

    /// Retorna as amostras (L, R) de um frame específico.
    #[must_use]
    pub fn get_frame(&self, frame: u64) -> Option<(f32, f32)> {
        if frame >= self.total_frames {
            return None;
        }
        let ch = usize::from(self.channels);
        let idx = (frame as usize) * ch;
        if ch == 2 && idx + 1 < self.samples.len() {
            Some((self.samples[idx], self.samples[idx + 1]))
        } else if ch == 1 && idx < self.samples.len() {
            Some((self.samples[idx], self.samples[idx]))
        } else {
            None
        }
    }

    /// Cria um buffer de silêncio estéreo com a duração em frames informada.
    #[must_use]
    pub fn silence(sample_rate: u32, frames: u64) -> Self {
        let samples = vec![0.0f32; (frames as usize) * 2];
        Self {
            sample_rate,
            channels: 2,
            samples,
            total_frames: frames,
        }
    }

    /// Cria um buffer sintético de senoide estéreo para testes determinísticos.
    #[must_use]
    pub fn synthetic_sine(sample_rate: u32, freq: f32, duration_s: f64) -> Self {
        let frames = (duration_s * f64::from(sample_rate)).round() as u64;
        let mut samples = Vec::with_capacity((frames as usize) * 2);
        for f in 0..frames {
            let t = f as f32 / sample_rate as f32;
            let val = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.5;
            samples.push(val);
            samples.push(val);
        }
        Self {
            sample_rate,
            channels: 2,
            samples,
            total_frames: frames,
        }
    }
}

/// Decodifica um arquivo `.wav` para `AudioBuffer` estéreo em `f32`.
pub fn decode_wav(path: &Path) -> Result<AudioBuffer, AudioError> {
    if !path.exists() {
        return Err(AudioError::FileNotFound(path.to_path_buf()));
    }

    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let sample_rate = spec.sample_rate;
    let channels = spec.channels;

    if channels == 0 || channels > 2 {
        return Err(AudioError::UnsupportedFormat(format!(
            "apenas mono (1) ou estéreo (2) são suportados, encontrado {channels}"
        )));
    }

    let mut samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<Vec<f32>, _>>()?,
        hound::SampleFormat::Int => {
            let bits = spec.bits_per_sample;
            match bits {
                16 => {
                    let max = i16::MAX as f32;
                    reader
                        .samples::<i16>()
                        .map(|s| s.map(|v| (v as f32) / max))
                        .collect::<Result<Vec<f32>, _>>()?
                }
                24 | 32 => {
                    let max = (1i64 << (bits - 1)) as f32;
                    reader
                        .samples::<i32>()
                        .map(|s| s.map(|v| (v as f32) / max))
                        .collect::<Result<Vec<f32>, _>>()?
                }
                _ => {
                    return Err(AudioError::UnsupportedFormat(format!(
                        "resolução de {bits} bits não suportada"
                    )));
                }
            }
        }
    };

    // Converter mono para estéreo intercalado
    if channels == 1 {
        let mut stereo = Vec::with_capacity(samples.len() * 2);
        for s in samples {
            stereo.push(s);
            stereo.push(s);
        }
        samples = stereo;
    }

    let total_frames = (samples.len() / 2) as u64;
    Ok(AudioBuffer {
        sample_rate,
        channels: 2,
        samples,
        total_frames,
    })
}

/// Decodifica qualquer formato suportado pelo Symphonia (MP3, FLAC, AAC, M4A, OGG, WAV) para `AudioBuffer` estéreo em `f32`.
pub fn decode_with_symphonia(path: &Path) -> Result<AudioBuffer, AudioError> {
    if !path.exists() {
        return Err(AudioError::FileNotFound(path.to_path_buf()));
    }

    let src = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .map_err(|e| AudioError::UnsupportedFormat(format!("falha ao identificar formato: {e}")))?;

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| {
            AudioError::UnsupportedFormat("nenhuma trilha de áudio encontrada".into())
        })?;

    let dec_opts: DecoderOptions = Default::default();
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .map_err(|e| AudioError::UnsupportedFormat(format!("codec não suportado: {e}")))?;

    let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);

    let mut sample_buf: Option<SampleBuffer<f32>> = None;
    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(SymphoniaError::ResetRequired) => {
                continue;
            }
            Err(SymphoniaError::IoError(_)) => {
                break;
            }
            Err(e) => {
                if !all_samples.is_empty() {
                    break;
                }
                return Err(AudioError::Corrupted(format!(
                    "erro durante leitura do pacote: {e}"
                )));
            }
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                let ch_count = spec.channels.count();
                if sample_buf.is_none() {
                    let duration = decoded.capacity() as u64;
                    sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                }

                if let Some(buf) = sample_buf.as_mut() {
                    buf.copy_interleaved_ref(decoded);
                    let samples = buf.samples();
                    if ch_count == 1 {
                        for &s in samples {
                            all_samples.push(s);
                            all_samples.push(s);
                        }
                    } else if ch_count == 2 {
                        all_samples.extend_from_slice(samples);
                    } else {
                        for frame in samples.chunks_exact(ch_count) {
                            all_samples.push(frame[0]);
                            all_samples.push(frame[1]);
                        }
                    }
                }
            }
            Err(SymphoniaError::DecodeError(_)) => {
                continue;
            }
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => {
                if !all_samples.is_empty() {
                    break;
                }
                return Err(AudioError::Corrupted(format!(
                    "erro ao decodificar pacote: {e}"
                )));
            }
        }
    }

    if all_samples.is_empty() {
        return Err(AudioError::UnsupportedFormat(
            "nenhuma amostra de áudio decodificada".into(),
        ));
    }

    let total_frames = (all_samples.len() / 2) as u64;

    Ok(AudioBuffer {
        sample_rate,
        channels: 2,
        samples: all_samples,
        total_frames,
    })
}

/// Decodifica um arquivo de áudio para PCM em memória.
pub fn decode_audio(path: &Path) -> Result<AudioBuffer, AudioError> {
    if !path.exists() {
        return Err(AudioError::FileNotFound(path.to_path_buf()));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "wav" => decode_wav(path).or_else(|_| decode_with_symphonia(path)),
        "mp3" | "flac" | "aac" | "m4a" | "ogg" | "aiff" => decode_with_symphonia(path),
        _ => decode_with_symphonia(path),
    }
}
