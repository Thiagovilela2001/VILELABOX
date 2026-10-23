use crate::{round2, round3};

#[derive(Debug, Clone, PartialEq)]
pub struct Beat {
    pub beat_number: u16,
    pub bpm: f64,
    pub time_ms: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    pub bpm: f64,
    pub variable: bool,
    pub first_beat_time_s: f64,
    pub downbeat_index: u32,
    pub beats: Vec<Beat>,
}

pub fn parse_pqtz(payload: &[u8]) -> Option<Grid> {
    if payload.len() < 12 {
        return None;
    }

    let num_beats_bytes = payload.get(8..12)?.try_into().ok()?;
    let num_beats = u32::from_be_bytes(num_beats_bytes) as usize;
    if num_beats == 0 {
        return None;
    }

    let needed_bytes = num_beats.checked_mul(8)?;
    let available_bytes = payload.len().checked_sub(12)?;
    if available_bytes < needed_bytes {
        return None;
    }

    let mut beats = Vec::with_capacity(num_beats);
    let mut downbeat_index = None;

    for i in 0..num_beats {
        let offset = 12 + i * 8;
        let beat_num_bytes = payload.get(offset..offset + 2)?.try_into().ok()?;
        let tempo_raw_bytes = payload.get(offset + 2..offset + 4)?.try_into().ok()?;
        let time_ms_bytes = payload.get(offset + 4..offset + 8)?.try_into().ok()?;

        let beat_number = u16::from_be_bytes(beat_num_bytes);
        let tempo_raw = u16::from_be_bytes(tempo_raw_bytes);
        let time_ms = u32::from_be_bytes(time_ms_bytes);

        let bpm = round2(tempo_raw as f64 / 100.0);

        if beat_number == 1 && downbeat_index.is_none() {
            downbeat_index = Some(i as u32);
        }

        beats.push(Beat {
            beat_number,
            bpm,
            time_ms,
        });
    }

    if beats.is_empty() {
        return None;
    }

    let first_beat_time_s = round3(beats[0].time_ms as f64 / 1000.0);
    let first_bpm = beats[0].bpm;
    let variable = beats
        .iter()
        .any(|b| (b.bpm - first_bpm).abs() > f64::EPSILON);
    let downbeat_index = downbeat_index.unwrap_or(0);

    Some(Grid {
        bpm: first_bpm,
        variable,
        first_beat_time_s,
        downbeat_index,
        beats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pqtz(beats: &[(u16, u16, u32)]) -> Vec<u8> {
        let mut payload = vec![0u8; 8];
        payload.extend((beats.len() as u32).to_be_bytes());
        for &(beat_num, tempo_raw, time_ms) in beats {
            payload.extend(beat_num.to_be_bytes());
            payload.extend(tempo_raw.to_be_bytes());
            payload.extend(time_ms.to_be_bytes());
        }
        payload
    }

    #[test]
    fn pqtz_duas_batidas_120() {
        let g = parse_pqtz(&pqtz(&[(1, 12000, 500), (2, 12000, 1000)])).unwrap();
        assert_eq!(
            (g.bpm, g.variable, g.first_beat_time_s, g.downbeat_index),
            (120.0, false, 0.5, 0)
        );
        assert_eq!(g.beats.len(), 2);
    }

    #[test]
    fn pqtz_variavel_e_downbeat() {
        let g = parse_pqtz(&pqtz(&[(3, 12800, 100), (4, 12800, 569), (1, 13000, 1038)])).unwrap();
        assert_eq!((g.bpm, g.variable, g.downbeat_index), (128.0, true, 2));
    }

    #[test]
    fn pqtz_contagem_absurda() {
        let mut p = vec![0u8; 8];
        p.extend(u32::MAX.to_be_bytes());
        assert_eq!(parse_pqtz(&p), None);
        assert_eq!(parse_pqtz(&[0u8; 11]), None);
    }
}
