//! Decodificação da coluna `library.beats` do Mixxx (`src/proto/beats.proto`),
//! seguindo `Beats::fromBeatGridByteArray` e `Beats::fromBeatMapByteArray` em
//! `src/track/beats.cpp`. A mensagem é escolhida por `library.beats_version`.

use crate::error::MixxxCompatError;
use vilelabox_types::{Beat, BeatGrid, Bpm, Frame, SampleRate, TempoChange};

/// Prefixos de `library.beats_version` (`src/track/beats.h`).
const BEAT_GRID_PREFIX: &str = "BeatGrid";
const BEAT_MAP_PREFIX: &str = "BeatMap";

/// Conteúdo do blob antes de virar `BeatGrid`.
#[derive(Debug, Clone, PartialEq)]
pub enum MixxxBeats {
    /// Andamento constante a partir de `first_beat` (frames).
    Grid {
        bpm: f64,
        first_beat: i64,
        first_downbeat_index: Option<i32>,
    },
    /// Batidas explícitas (frames), em ordem.
    Map {
        beats: Vec<i64>,
        first_downbeat_index: Option<i32>,
    },
}

fn invalid(message: &str) -> MixxxCompatError {
    MixxxCompatError::InvalidBeatsProto(message.to_string())
}

/// Campo protobuf já lido: número e valor bruto.
enum Value<'a> {
    Varint(u64),
    Fixed64([u8; 8]),
    Bytes(&'a [u8]),
}

/// Percorre os campos de uma mensagem. Erro em qualquer byte truncado ou wire
/// type desconhecido, como `ParseFromArray`.
fn fields(mut buf: &[u8]) -> Result<Vec<(u64, Value<'_>)>, MixxxCompatError> {
    let mut out = Vec::new();
    while !buf.is_empty() {
        let (tag, n) = read_varint(buf).ok_or_else(|| invalid("tag truncada"))?;
        buf = &buf[n..];
        let value = match tag & 7 {
            0 => {
                let (v, n) = read_varint(buf).ok_or_else(|| invalid("varint truncado"))?;
                buf = &buf[n..];
                Value::Varint(v)
            }
            1 => {
                let bytes: [u8; 8] = buf
                    .get(..8)
                    .and_then(|b| b.try_into().ok())
                    .ok_or_else(|| invalid("fixed64 truncado"))?;
                buf = &buf[8..];
                Value::Fixed64(bytes)
            }
            2 => {
                let (len, n) = read_varint(buf).ok_or_else(|| invalid("tamanho truncado"))?;
                let end = n
                    .checked_add(usize::try_from(len).map_err(|_| invalid("tamanho inválido"))?)
                    .filter(|&end| end <= buf.len())
                    .ok_or_else(|| invalid("submensagem truncada"))?;
                let bytes = &buf[n..end];
                buf = &buf[end..];
                Value::Bytes(bytes)
            }
            5 => {
                if buf.len() < 4 {
                    return Err(invalid("fixed32 truncado"));
                }
                buf = &buf[4..];
                continue;
            }
            _ => return Err(invalid("wire type desconhecido")),
        };
        if tag >> 3 == 0 {
            return Err(invalid("campo 0"));
        }
        out.push((tag >> 3, value));
    }
    Ok(out)
}

/// `Beat.frame_position` (int32): negativos chegam como varint de 64 bits.
fn beat_frame(msg: &[u8]) -> Result<Option<i64>, MixxxCompatError> {
    let mut frame = None;
    for (field, value) in fields(msg)? {
        if let (1, Value::Varint(v)) = (field, value) {
            frame = Some(i64::from(v as i32));
        }
    }
    Ok(frame)
}

fn zigzag(v: u64) -> i32 {
    ((v >> 1) as i64 ^ -((v & 1) as i64)) as i32
}

fn decode_grid(data: &[u8]) -> Result<MixxxBeats, MixxxCompatError> {
    let mut bpm = None;
    let mut first_beat = None;
    let mut first_downbeat_index = None;
    for (field, value) in fields(data)? {
        match (field, value) {
            (1, Value::Bytes(msg)) => {
                for (f, v) in fields(msg)? {
                    if let (1, Value::Fixed64(b)) = (f, v) {
                        bpm = Some(f64::from_le_bytes(b));
                    }
                }
            }
            (2, Value::Bytes(msg)) => first_beat = beat_frame(msg)?,
            (3, Value::Varint(v)) => first_downbeat_index = Some(zigzag(v)),
            _ => {}
        }
    }
    Ok(MixxxBeats::Grid {
        bpm: bpm.ok_or_else(|| invalid("BeatGrid sem bpm"))?,
        // proto2: first_beat ausente vale frame 0.
        first_beat: first_beat.unwrap_or(0),
        first_downbeat_index,
    })
}

fn decode_map(data: &[u8]) -> Result<MixxxBeats, MixxxCompatError> {
    let mut beats = Vec::new();
    let mut first_downbeat_index = None;
    for (field, value) in fields(data)? {
        match (field, value) {
            (1, Value::Bytes(msg)) => beats.push(beat_frame(msg)?.unwrap_or(0)),
            (2, Value::Varint(v)) => first_downbeat_index = Some(zigzag(v)),
            _ => {}
        }
    }
    if beats.len() < 2 {
        return Err(invalid("BeatMap precisa de pelo menos 2 batidas"));
    }
    Ok(MixxxBeats::Map {
        beats,
        first_downbeat_index,
    })
}

/// Decodifica o blob conforme `beats_version` (`BeatGrid-1.0`, `BeatGrid-2.0`,
/// `BeatMap-1.0`). `BeatGrid-1.0` antigo é a struct `{double bpm; double
/// firstBeat}` sem protobuf, usada quando o parse falha.
pub fn decode_beats(data: &[u8], beats_version: &str) -> Result<MixxxBeats, MixxxCompatError> {
    if beats_version.starts_with(BEAT_MAP_PREFIX) {
        return decode_map(data);
    }
    if !beats_version.starts_with(BEAT_GRID_PREFIX) {
        return Err(invalid(&format!(
            "beats_version desconhecida: {}",
            beats_version
        )));
    }
    match decode_grid(data) {
        Ok(grid) => Ok(grid),
        Err(_) if data.len() == 16 => {
            let bpm = f64::from_ne_bytes(data[..8].try_into().unwrap());
            let first = f64::from_ne_bytes(data[8..].try_into().unwrap());
            Ok(MixxxBeats::Grid {
                bpm,
                first_beat: first.round() as i64,
                first_downbeat_index: None,
            })
        }
        Err(e) => Err(e),
    }
}

/// Converte o blob em `BeatGrid`. Grades de andamento constante são expandidas
/// da primeira batida até `duration`. O downbeat é a batida da fase
/// `first_downbeat_index mod 4` (o Mixxx guarda o índice absoluto).
pub fn beats_proto_to_grid(
    data: &[u8],
    beats_version: &str,
    sample_rate: SampleRate,
    duration: Frame,
) -> Result<BeatGrid, MixxxCompatError> {
    let (positions, tempo_map, downbeat_index) = match decode_beats(data, beats_version)? {
        MixxxBeats::Grid {
            bpm,
            first_beat,
            first_downbeat_index,
        } => {
            let bpm = Bpm::new(bpm)?;
            let beat_length = 60.0 * f64::from(sample_rate.get()) / bpm.get();
            if beat_length < 1.0 {
                return Err(invalid("bpm maior que a taxa de amostragem"));
            }
            let positions: Vec<i64> = (0..)
                .map(|n| first_beat + (n as f64 * beat_length).round() as i64)
                // A primeira batida entra mesmo além da duração (faixa curta).
                .take_while(|&p| p == first_beat || p < duration.get())
                .collect();
            let tempo = vec![TempoChange {
                position: Frame::new(first_beat),
                bpm,
            }];
            (positions, tempo, first_downbeat_index)
        }
        MixxxBeats::Map {
            beats,
            first_downbeat_index,
        } => {
            // Andamento de cada trecho entre batidas; nova entrada só quando muda.
            let mut tempo: Vec<TempoChange> = Vec::new();
            for pair in beats.windows(2) {
                let frames = (pair[1] - pair[0]) as f64;
                if frames <= 0.0 {
                    continue; // BeatGrid::new rejeita a ordem abaixo
                }
                let bpm = Bpm::new(60.0 * f64::from(sample_rate.get()) / frames)?;
                let rounded = (bpm.get() * 100.0).round() / 100.0;
                if tempo.last().map(|t| t.bpm.get()) != Some(rounded) {
                    tempo.push(TempoChange {
                        position: Frame::new(pair[0]),
                        bpm: Bpm::new(rounded)?,
                    });
                }
            }
            (beats, tempo, first_downbeat_index)
        }
    };

    let beats: Vec<Beat> = positions
        .iter()
        .enumerate()
        .map(|(i, &p)| Beat {
            position: Frame::new(p),
            beat_num: (i + 1) as u32,
        })
        .collect();
    let downbeat = downbeat_index
        .and_then(|i| beats.get(i.rem_euclid(4) as usize))
        .map(|b| b.position);
    Ok(BeatGrid::new(sample_rate, beats, tempo_map, downbeat)?)
}

fn read_varint(buf: &[u8]) -> Option<(u64, usize)> {
    let mut value: u64 = 0;
    for (i, &b) in buf.iter().enumerate().take(10) {
        value |= u64::from(b & 0x7F) << (7 * i);
        if b & 0x80 == 0 {
            return Some((value, i + 1));
        }
    }
    None
}
