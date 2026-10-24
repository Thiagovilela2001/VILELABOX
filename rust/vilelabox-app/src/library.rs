use std::fs;
use std::path::{Path, PathBuf};
use vilelabox_anlz::{parse_ppth, parse_pqtz, parse_pssi, sections, Grid, RawPhrase};

pub struct Track {
    pub title: String,
    pub ext: Option<PathBuf>,
    pub grid: Option<Grid>,
}

pub struct PhraseSpan {
    pub start: f32,
    pub end: f32,
    pub kind: String,
}

pub struct Timeline {
    pub info: String,
    /// Posição relativa (0..1) de cada compasso.
    pub bars: Vec<f32>,
    pub phrases: Vec<PhraseSpan>,
}

fn dat_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            dat_files(&path, out);
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("dat"))
        {
            out.push(path);
        }
    }
}

fn title_from_path(raw: &str) -> String {
    raw.rsplit(['/', '\\']).next().unwrap_or(raw).to_string()
}

/// Lê todas as análises `.DAT` (e o `.EXT` irmão, se houver) sob `dir`.
/// Arquivos ilegíveis ou sem caminho de áudio entram com o nome do arquivo.
pub fn scan(dir: &Path) -> Vec<Track> {
    let mut files = Vec::new();
    dat_files(dir, &mut files);
    let mut tracks: Vec<Track> = files
        .into_iter()
        .filter_map(|dat_path| {
            let dat = fs::read(&dat_path).ok()?;
            let ext_path = dat_path.with_extension("EXT");
            let ext = fs::read(&ext_path).ok();
            let dat_secs = sections(&dat);
            let ext_secs = ext.as_deref().map(sections).unwrap_or_default();
            let ppth = ext_secs
                .get(b"PPTH")
                .or(dat_secs.get(b"PPTH"))
                .and_then(|p| parse_ppth(p));
            let title = ppth
                .as_deref()
                .map_or_else(|| dat_path.display().to_string(), title_from_path);
            let grid = dat_secs.get(b"PQTZ").and_then(|p| parse_pqtz(p));
            Some(Track {
                title,
                ext: ext.is_some().then_some(ext_path),
                grid,
            })
        })
        .collect();
    tracks.sort_by_key(|t| t.title.to_lowercase());
    tracks
}

fn canonical_kind(kind: &str) -> String {
    if kind.starts_with("verse") {
        "VERSE".into()
    } else {
        kind.to_uppercase()
    }
}

/// Monta a linha do tempo: compassos e frases em posição relativa à duração
/// coberta pela grade (última batida + uma batida).
pub fn build_timeline(grid: &Grid, phrases: &[RawPhrase]) -> (Vec<f32>, Vec<PhraseSpan>) {
    let Some(last) = grid.beats.last() else {
        return (Vec::new(), Vec::new());
    };
    let beat_ms = if grid.bpm > 0.0 {
        60_000.0 / grid.bpm
    } else {
        0.0
    };
    let total_ms = last.time_ms as f64 + beat_ms;
    if total_ms <= 0.0 {
        return (Vec::new(), Vec::new());
    }
    let rel = |ms: f64| (ms / total_ms).clamp(0.0, 1.0) as f32;

    let bars = grid
        .beats
        .iter()
        .filter(|b| b.beat_number == 1)
        .map(|b| rel(b.time_ms as f64))
        .collect();

    let starts: Vec<(f32, String)> = phrases
        .iter()
        .filter_map(|p| Some((rel(p.start_time_s? * 1000.0), canonical_kind(&p.kind))))
        .collect();
    let spans = starts
        .iter()
        .enumerate()
        .map(|(i, (start, kind))| PhraseSpan {
            start: *start,
            end: starts.get(i + 1).map_or(1.0, |n| n.0),
            kind: kind.clone(),
        })
        .collect();
    (bars, spans)
}

pub fn timeline(track: &Track) -> Timeline {
    let Some(grid) = &track.grid else {
        return Timeline {
            info: "Sem grade de batidas nesta análise.".into(),
            bars: Vec::new(),
            phrases: Vec::new(),
        };
    };
    let beat_times: Vec<u32> = grid.beats.iter().map(|b| b.time_ms).collect();
    let raw = track
        .ext
        .as_ref()
        .and_then(|p| fs::read(p).ok())
        .map(|ext| {
            sections(&ext)
                .get(b"PSSI")
                .map(|p| parse_pssi(p, &beat_times))
                .unwrap_or_default()
        })
        .unwrap_or_default();
    let (bars, phrases) = build_timeline(grid, &raw);
    let tipo = if grid.variable { "variável" } else { "fixo" };
    Timeline {
        info: format!(
            "{:.2} BPM ({tipo})  ·  {} batidas  ·  {} compassos  ·  {} frases  ·  1ª batida {:.3} s",
            grid.bpm,
            grid.beats.len(),
            bars.len(),
            phrases.len(),
            grid.first_beat_time_s
        ),
        bars,
        phrases,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vilelabox_anlz::Beat;

    fn grid(beats: &[(u16, u32)], bpm: f64) -> Grid {
        Grid {
            bpm,
            variable: false,
            first_beat_time_s: beats[0].1 as f64 / 1000.0,
            downbeat_index: 0,
            beats: beats
                .iter()
                .map(|&(n, t)| Beat {
                    beat_number: n,
                    bpm,
                    time_ms: t,
                })
                .collect(),
        }
    }

    fn phrase(beat: u16, kind: &str, start: Option<f64>) -> RawPhrase {
        RawPhrase {
            index: 0,
            beat,
            kind: kind.into(),
            kind_id: 0,
            start_time_s: start,
        }
    }

    #[test]
    fn compassos_e_frases_relativos() {
        // 120 BPM, 8 batidas a cada 500 ms: duração 3500 + 500 = 4000 ms
        let g = grid(
            &[
                (1, 0),
                (2, 500),
                (3, 1000),
                (4, 1500),
                (1, 2000),
                (2, 2500),
                (3, 3000),
                (4, 3500),
            ],
            120.0,
        );
        let p = [
            phrase(1, "intro", Some(0.0)),
            phrase(5, "verse_2", Some(2.0)),
            phrase(99, "outro", None),
        ];
        let (bars, spans) = build_timeline(&g, &p);
        assert_eq!(bars, vec![0.0, 0.5]);
        assert_eq!(spans.len(), 2);
        assert_eq!(
            (spans[0].start, spans[0].end, spans[0].kind.as_str()),
            (0.0, 0.5, "INTRO")
        );
        assert_eq!(
            (spans[1].start, spans[1].end, spans[1].kind.as_str()),
            (0.5, 1.0, "VERSE")
        );
    }

    #[test]
    fn titulo_pelo_nome_do_arquivo() {
        assert_eq!(title_from_path("C:\\Music\\Faixa A.mp3"), "Faixa A.mp3");
        assert_eq!(title_from_path("/music/b.flac"), "b.flac");
    }
}
