//! Edição não destrutiva da beatgrid usada pelo painel GRID/PHRASE EDIT.
//!
//! O contrato de comandos segue o manual do rekordbox 7.2.8, pp. 135–136:
//! deslocamento de 1/3 ms, BPM/TAP, largura do intervalo, ×2/÷2, faixa
//! inteira ou posição atual em diante, reset, 1.1, undo/redo e lock.

use std::collections::VecDeque;
use vilelabox_anlz::Beat;

const HISTORY_LIMIT: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditRange {
    Whole,
    FromPosition,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridAction {
    ShiftMs(i32),
    SpacingMs(i32),
    SetBpm(f64),
    ScaleBpm(f64),
    ResetAt { bpm: f64 },
    SetDownbeat,
}

#[derive(Debug, Clone)]
pub struct BeatGridEditor {
    undo: VecDeque<Vec<Beat>>,
    redo: Vec<Vec<Beat>>,
    taps_s: VecDeque<f64>,
    pub range: EditRange,
    pub fine: bool,
    pub locked: bool,
    /// 0 = desligado; 1..=3 = níveis do metrônomo.
    pub metronome: u8,
}

impl Default for BeatGridEditor {
    fn default() -> Self {
        Self {
            undo: VecDeque::new(),
            redo: Vec::new(),
            taps_s: VecDeque::new(),
            range: EditRange::Whole,
            fine: false,
            locked: false,
            metronome: 0,
        }
    }
}

impl BeatGridEditor {
    #[must_use]
    pub fn step_ms(&self) -> i32 {
        if self.fine {
            1
        } else {
            3
        }
    }

    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn clear_history(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.taps_s.clear();
        self.range = EditRange::Whole;
    }

    pub fn toggle_metronome(&mut self) {
        self.metronome = (self.metronome + 1) % 4;
    }

    pub fn undo(&mut self, beats: &mut Vec<Beat>) -> bool {
        if self.locked {
            return false;
        }
        let Some(previous) = self.undo.pop_back() else {
            return false;
        };
        self.redo.push(std::mem::replace(beats, previous));
        true
    }

    pub fn redo(&mut self, beats: &mut Vec<Beat>) -> bool {
        if self.locked {
            return false;
        }
        let Some(next) = self.redo.pop() else {
            return false;
        };
        self.push_undo(std::mem::replace(beats, next));
        true
    }

    /// Registra um TAP e, a partir do segundo, devolve o BPM médio. Pausas
    /// maiores que 2,5 s iniciam uma nova sequência; são usadas no máximo as
    /// oito batidas mais recentes para que o controle continue responsivo.
    pub fn tap(&mut self, at_s: f64) -> Option<f64> {
        if self
            .taps_s
            .back()
            .is_some_and(|last| at_s <= *last || at_s - *last > 2.5)
        {
            self.taps_s.clear();
        }
        self.taps_s.push_back(at_s);
        while self.taps_s.len() > 8 {
            self.taps_s.pop_front();
        }
        let first = *self.taps_s.front()?;
        let last = *self.taps_s.back()?;
        let intervals = self.taps_s.len().checked_sub(1)?;
        (intervals > 0 && last > first).then_some(60.0 * intervals as f64 / (last - first))
    }

    pub fn apply(
        &mut self,
        beats: &mut Vec<Beat>,
        action: GridAction,
        position_s: f64,
        duration_s: f64,
    ) -> bool {
        if self.locked || beats.is_empty() {
            return false;
        }
        let before = beats.clone();
        let anchor = nearest(beats, position_s);
        match action {
            GridAction::ShiftMs(delta) => shift(beats, self.start(anchor), delta),
            GridAction::SpacingMs(delta) => {
                let start = self.start(anchor);
                change_spacing(beats, start, delta, duration_s);
            }
            GridAction::SetBpm(bpm) if valid_bpm(bpm) => {
                let start = self.start(anchor);
                rebuild(beats, start, 60_000.0 / bpm, bpm, duration_s, None);
            }
            GridAction::ScaleBpm(factor) if factor > 0.0 => {
                let bpm = bpm_at(beats, position_s) * factor;
                if valid_bpm(bpm) {
                    let start = self.start(anchor);
                    rebuild(beats, start, 60_000.0 / bpm, bpm, duration_s, None);
                }
            }
            GridAction::ResetAt { bpm } if valid_bpm(bpm) => {
                let at_ms = (position_s.max(0.0) * 1000.0).round();
                rebuild(beats, anchor, 60_000.0 / bpm, bpm, duration_s, Some(at_ms));
            }
            GridAction::SetDownbeat => set_downbeat(beats, anchor),
            _ => {}
        }
        if *beats == before {
            return false;
        }
        self.push_undo(before);
        self.redo.clear();
        true
    }

    fn start(&self, anchor: usize) -> usize {
        if self.range == EditRange::Whole {
            0
        } else {
            anchor
        }
    }

    fn push_undo(&mut self, beats: Vec<Beat>) {
        if self.undo.len() == HISTORY_LIMIT {
            self.undo.pop_front();
        }
        self.undo.push_back(beats);
    }
}

#[must_use]
pub fn bpm_at(beats: &[Beat], position_s: f64) -> f64 {
    let ms = position_s.max(0.0) * 1000.0;
    beats
        .iter()
        .rfind(|b| f64::from(b.time_ms) <= ms)
        .or_else(|| beats.first())
        .map_or(0.0, |b| b.bpm)
}

#[must_use]
pub fn nearest_downbeat_s(beats: &[Beat], position_s: f64) -> Option<f64> {
    beats
        .iter()
        .filter(|b| b.beat_number == 1)
        .min_by(|a, b| {
            (f64::from(a.time_ms) / 1000.0 - position_s)
                .abs()
                .total_cmp(&(f64::from(b.time_ms) / 1000.0 - position_s).abs())
        })
        .map(|b| f64::from(b.time_ms) / 1000.0)
}

fn valid_bpm(bpm: f64) -> bool {
    bpm.is_finite() && (20.0..=999.0).contains(&bpm)
}

fn nearest(beats: &[Beat], position_s: f64) -> usize {
    let ms = position_s.max(0.0) * 1000.0;
    beats
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (f64::from(a.time_ms) - ms)
                .abs()
                .total_cmp(&(f64::from(b.time_ms) - ms).abs())
        })
        .map_or(0, |(i, _)| i)
}

fn shift(beats: &mut [Beat], start: usize, delta: i32) {
    for b in beats.iter_mut().skip(start) {
        b.time_ms = if delta >= 0 {
            b.time_ms.saturating_add(delta as u32)
        } else {
            b.time_ms.saturating_sub(delta.unsigned_abs())
        };
    }
}

fn change_spacing(beats: &mut Vec<Beat>, start: usize, delta: i32, duration_s: f64) {
    let interval = beats
        .get(start + 1)
        .map(|n| f64::from(n.time_ms.saturating_sub(beats[start].time_ms)))
        .or_else(|| (beats[start].bpm > 0.0).then_some(60_000.0 / beats[start].bpm));
    let Some(interval) = interval else { return };
    let interval = (interval + f64::from(delta)).max(1.0);
    let bpm = 60_000.0 / interval;
    rebuild(beats, start, interval, bpm, duration_s, None);
}

fn rebuild(
    beats: &mut Vec<Beat>,
    start: usize,
    interval_ms: f64,
    bpm: f64,
    duration_s: f64,
    anchor_ms: Option<f64>,
) {
    if start >= beats.len() || interval_ms <= 0.0 {
        return;
    }
    let anchor = anchor_ms.unwrap_or(f64::from(beats[start].time_ms));
    let first_number = beats[start].beat_number.max(1);
    beats.truncate(start);
    let end_ms = (duration_s.max(anchor / 1000.0) * 1000.0).round();
    let mut i = 0usize;
    loop {
        let time = anchor + interval_ms * i as f64;
        if time > end_ms + interval_ms || time > f64::from(u32::MAX) {
            break;
        }
        beats.push(Beat {
            beat_number: ((usize::from(first_number - 1) + i) % 4 + 1) as u16,
            bpm,
            time_ms: time.max(0.0).round() as u32,
        });
        i += 1;
        if i > 1_000_000 {
            break;
        }
    }
}

fn set_downbeat(beats: &mut [Beat], anchor: usize) {
    for (i, b) in beats.iter_mut().enumerate() {
        b.beat_number = ((i + 4 - anchor % 4) % 4 + 1) as u16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> Vec<Beat> {
        (0..9)
            .map(|i| Beat {
                beat_number: (i % 4 + 1) as u16,
                bpm: 120.0,
                time_ms: 1000 + i * 500,
            })
            .collect()
    }

    #[test]
    fn desloca_com_passos_normal_e_fino_e_desfaz() {
        let mut e = BeatGridEditor::default();
        let mut g = grid();
        assert_eq!(e.step_ms(), 3);
        e.apply(&mut g, GridAction::ShiftMs(-e.step_ms()), 2.0, 5.5);
        assert_eq!(g[0].time_ms, 997);
        e.fine = true;
        assert_eq!(e.step_ms(), 1);
        assert!(e.undo(&mut g));
        assert_eq!(g, grid());
        assert!(e.redo(&mut g));
        assert_eq!(g[0].time_ms, 997);
    }

    #[test]
    fn bpm_inteiro_e_parcial_preservam_ancora() {
        let mut e = BeatGridEditor::default();
        let mut g = grid();
        e.apply(&mut g, GridAction::SetBpm(150.0), 3.0, 5.0);
        assert_eq!((g[0].time_ms, g[1].time_ms), (1000, 1400));

        let mut g = grid();
        e.clear_history();
        e.range = EditRange::FromPosition;
        e.apply(&mut g, GridAction::SetBpm(100.0), 3.0, 5.0);
        assert_eq!(g[3].time_ms, 2500);
        assert_eq!(g[4].time_ms, 3000);
        assert_eq!(g[5].time_ms, 3600);

        e.apply(&mut g, GridAction::ShiftMs(3), 2.5, 5.0);
        assert_eq!(g[2].time_ms, 2000);
        assert_eq!(g[3].time_ms, 2503);
    }

    #[test]
    fn reset_downbeat_tap_lock_e_metrônomo() {
        let mut e = BeatGridEditor::default();
        let mut g = grid();
        e.apply(&mut g, GridAction::ResetAt { bpm: 120.0 }, 2.26, 5.0);
        assert_eq!(g[3].time_ms, 2260);
        e.apply(&mut g, GridAction::SetDownbeat, 2.26, 5.0);
        assert_eq!(g[3].beat_number, 1);
        assert_eq!(e.tap(10.0), None);
        assert_eq!(e.tap(10.5), Some(120.0));
        e.toggle_metronome();
        assert_eq!(e.metronome, 1);
        e.locked = true;
        let frozen = g.clone();
        assert!(!e.apply(&mut g, GridAction::ShiftMs(3), 0.0, 5.0));
        assert_eq!(g, frozen);
    }

    #[test]
    fn localiza_downbeat_mais_proximo() {
        assert_eq!(nearest_downbeat_s(&grid(), 4.8), Some(5.0));
    }
}
