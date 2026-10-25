//! Persistência atômica das beatgrids editadas pelo usuário.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use vilelabox_anlz::Beat;

#[derive(Serialize, Deserialize)]
struct SavedGrid {
    version: u8,
    locked: bool,
    beats: Vec<SavedBeat>,
}

#[derive(Serialize, Deserialize)]
struct SavedBeat {
    number: u16,
    bpm: f64,
    time_ms: u32,
}

fn fnv1a(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

fn file(base: &Path, track: &Path) -> PathBuf {
    let key = track.to_string_lossy();
    base.join(format!("grid-{:016x}.json", fnv1a(&key)))
}

pub fn read(base: &Path, track: &Path) -> Option<(Vec<Beat>, bool)> {
    let bytes = std::fs::read(file(base, track)).ok()?;
    let saved: SavedGrid = serde_json::from_slice(&bytes).ok()?;
    if saved.version != 1 || saved.beats.is_empty() {
        return None;
    }
    let beats: Vec<Beat> = saved
        .beats
        .into_iter()
        .map(|b| Beat {
            beat_number: b.number,
            bpm: b.bpm,
            time_ms: b.time_ms,
        })
        .collect();
    beats
        .windows(2)
        .all(|w| w[0].time_ms < w[1].time_ms)
        .then_some((beats, saved.locked))
}

pub fn write(base: &Path, track: &Path, beats: &[Beat], locked: bool) -> std::io::Result<()> {
    std::fs::create_dir_all(base)?;
    let destination = file(base, track);
    let temporary = destination.with_extension("json.tmp");
    let value = SavedGrid {
        version: 1,
        locked,
        beats: beats
            .iter()
            .map(|b| SavedBeat {
                number: b.beat_number,
                bpm: b.bpm,
                time_ms: b.time_ms,
            })
            .collect(),
    };
    let bytes = serde_json::to_vec_pretty(&value).map_err(std::io::Error::other)?;
    {
        use std::io::Write;
        let mut out = std::fs::File::create(&temporary)?;
        out.write_all(&bytes)?;
        out.sync_all()?;
    }
    std::fs::rename(temporary, destination)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_preserva_grade_e_lock() {
        let dir = std::env::temp_dir().join(format!("vilelabox-grid-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let track = Path::new("C:/Music/faixa.wav");
        let beats = vec![
            Beat {
                beat_number: 1,
                bpm: 120.0,
                time_ms: 1000,
            },
            Beat {
                beat_number: 2,
                bpm: 120.0,
                time_ms: 1500,
            },
        ];
        write(&dir, track, &beats, true).unwrap();
        assert_eq!(read(&dir, track), Some((beats, true)));
        let _ = std::fs::remove_dir_all(dir);
    }
}
