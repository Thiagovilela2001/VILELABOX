use crate::error::MixxxCompatError;
use vilelabox_types::{Beat, BeatGrid, Bpm, Frame, SampleRate, TempoChange};

/// Decodifica o blob binário do beats.proto do Mixxx.
/// Retorna tuplas de (posição_em_frames, bpm_opcional).
pub fn decode_beats_proto(data: &[u8]) -> Result<Vec<(i32, Option<f32>)>, MixxxCompatError> {
    let mut beats = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let (tag_wire, n) = read_varint(&data[i..])
            .ok_or_else(|| MixxxCompatError::InvalidBeatsProto("Varint truncado".into()))?;
        i += n;
        let field_num = tag_wire >> 3;
        let wire_type = tag_wire & 7;

        if field_num == 1 && wire_type == 2 {
            // Submensagem Beat
            let (sub_len, n) = read_varint(&data[i..]).ok_or_else(|| {
                MixxxCompatError::InvalidBeatsProto("Tamanho submensagem truncado".into())
            })?;
            i += n;
            let sub_len = sub_len as usize;
            if i + sub_len > data.len() {
                return Err(MixxxCompatError::InvalidBeatsProto(
                    "Payload da submensagem truncado".into(),
                ));
            }
            let sub_bytes = &data[i..i + sub_len];
            i += sub_len;

            let mut frame_pos: Option<i32> = None;
            let mut bpm: Option<f32> = None;
            let mut si = 0;
            while si < sub_bytes.len() {
                let (sub_tw, sn) = read_varint(&sub_bytes[si..]).ok_or_else(|| {
                    MixxxCompatError::InvalidBeatsProto("Varint interno truncado".into())
                })?;
                si += sn;
                let sf_num = sub_tw >> 3;
                let sf_wire = sub_tw & 7;

                if sf_num == 1 && sf_wire == 0 {
                    let (pos, pn) = read_varint(&sub_bytes[si..]).ok_or_else(|| {
                        MixxxCompatError::InvalidBeatsProto("Posição truncada".into())
                    })?;
                    si += pn;
                    frame_pos = Some(pos as i32);
                } else if sf_num == 2 && sf_wire == 5 {
                    if si + 4 > sub_bytes.len() {
                        return Err(MixxxCompatError::InvalidBeatsProto("Float truncado".into()));
                    }
                    let bytes = [
                        sub_bytes[si],
                        sub_bytes[si + 1],
                        sub_bytes[si + 2],
                        sub_bytes[si + 3],
                    ];
                    bpm = Some(f32::from_le_bytes(bytes));
                    si += 4;
                } else {
                    let skipped = skip_field(sf_wire, &sub_bytes[si..]).ok_or_else(|| {
                        MixxxCompatError::InvalidBeatsProto("Campo desconhecido não pulado".into())
                    })?;
                    si += skipped;
                }
            }
            if let Some(pos) = frame_pos {
                beats.push((pos, bpm));
            }
        } else {
            let skipped = skip_field(wire_type, &data[i..]).ok_or_else(|| {
                MixxxCompatError::InvalidBeatsProto("Campo externo não pulado".into())
            })?;
            i += skipped;
        }
    }
    Ok(beats)
}

/// Converte o blob beats.proto em um BeatGrid completo do vilelabox-types.
pub fn beats_proto_to_grid(
    data: &[u8],
    sample_rate: SampleRate,
    default_bpm: Option<Bpm>,
) -> Result<BeatGrid, MixxxCompatError> {
    let raw_beats = decode_beats_proto(data)?;
    let mut beats = Vec::new();
    let mut tempo_changes = Vec::new();
    for (i, (pos, maybe_bpm)) in raw_beats.into_iter().enumerate() {
        let frame = Frame::new(pos as i64);
        let beat_num = (i + 1) as u32;
        beats.push(Beat {
            position: frame,
            beat_num,
        });

        if let Some(b) = maybe_bpm {
            if let Ok(bpm) = Bpm::new(b as f64) {
                tempo_changes.push(TempoChange {
                    position: frame,
                    bpm,
                });
            }
        }
    }

    if tempo_changes.is_empty() {
        if let Some(b) = default_bpm {
            tempo_changes.push(TempoChange {
                position: Frame::ZERO,
                bpm: b,
            });
        }
    }

    let downbeat = beats.first().map(|b| b.position);
    let grid = BeatGrid::new(sample_rate, beats, tempo_changes, downbeat)?;
    Ok(grid)
}

fn read_varint(buf: &[u8]) -> Option<(u64, usize)> {
    let mut value: u64 = 0;
    let mut shift = 0;
    for (i, &b) in buf.iter().enumerate() {
        value |= ((b & 0x7F) as u64) << shift;
        if (b & 0x80) == 0 {
            return Some((value, i + 1));
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }
    None
}

fn skip_field(wire_type: u64, buf: &[u8]) -> Option<usize> {
    match wire_type {
        0 => read_varint(buf).map(|(_, n)| n),
        1 => (buf.len() >= 8).then_some(8),
        2 => {
            let (len, n) = read_varint(buf)?;
            let total = n + (len as usize);
            (buf.len() >= total).then_some(total)
        }
        5 => (buf.len() >= 4).then_some(4),
        _ => None,
    }
}
