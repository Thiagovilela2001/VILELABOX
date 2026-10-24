use crate::error::TypeError;
use crate::units::{Bpm, Frame, SampleRate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Beat {
    pub position: Frame,
    pub beat_num: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TempoChange {
    pub position: Frame,
    pub bpm: Bpm,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeatGrid {
    sample_rate: SampleRate,
    beats: Vec<Beat>,
    tempo_map: Vec<TempoChange>,
    downbeat: Option<Frame>,
}

impl BeatGrid {
    pub fn new(
        sample_rate: SampleRate,
        beats: Vec<Beat>,
        tempo_map: Vec<TempoChange>,
        downbeat: Option<Frame>,
    ) -> Result<Self, TypeError> {
        // Invariante 1: se houver batidas, elas devem estar ordenadas estritamente crescente
        for window in beats.windows(2) {
            if window[0].position >= window[1].position {
                return Err(TypeError::GridInvariantViolation(format!(
                    "Batidas fora de ordem estritamente crescente: {} >= {}",
                    window[0].position, window[1].position
                )));
            }
        }

        // Invariante 2: tempo_map deve estar ordenado
        for window in tempo_map.windows(2) {
            if window[0].position > window[1].position {
                return Err(TypeError::GridInvariantViolation(format!(
                    "Mudanças de tempo fora de ordem: {} > {}",
                    window[0].position, window[1].position
                )));
            }
        }

        Ok(Self {
            sample_rate,
            beats,
            tempo_map,
            downbeat,
        })
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub fn beats(&self) -> &[Beat] {
        &self.beats
    }

    pub fn tempo_map(&self) -> &[TempoChange] {
        &self.tempo_map
    }

    pub fn downbeat(&self) -> Option<Frame> {
        self.downbeat
    }

    pub fn primary_bpm(&self) -> Option<Bpm> {
        self.tempo_map.first().map(|tc| tc.bpm)
    }

    pub fn bpm_at(&self, frame: Frame) -> Option<Bpm> {
        if self.tempo_map.is_empty() {
            return None;
        }
        let mut current_bpm = self.tempo_map[0].bpm;
        for tc in &self.tempo_map {
            if tc.position <= frame {
                current_bpm = tc.bpm;
            } else {
                break;
            }
        }
        Some(current_bpm)
    }

    pub fn closest_beat(&self, frame: Frame) -> Option<&Beat> {
        if self.beats.is_empty() {
            return None;
        }
        let idx = match self.beats.binary_search_by_key(&frame, |b| b.position) {
            Ok(i) => i,
            Err(i) => {
                if i == 0 {
                    0
                } else if i >= self.beats.len() {
                    self.beats.len() - 1
                } else {
                    let diff_before = (frame.get() - self.beats[i - 1].position.get()).abs();
                    let diff_after = (self.beats[i].position.get() - frame.get()).abs();
                    if diff_before <= diff_after {
                        i - 1
                    } else {
                        i
                    }
                }
            }
        };
        Some(&self.beats[idx])
    }
}
