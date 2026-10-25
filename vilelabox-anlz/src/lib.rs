pub mod canonical;
pub mod container;
pub mod cues;
pub mod error;
pub mod export;
pub mod grid;
pub mod phrases;
pub mod waveform;

pub use canonical::{
    canonical_cues, canonical_phrases, cue_payloads, cue_sources, track_from_anlz,
};
pub use container::{all_sections, parse_ppth, sections, FourCc};
pub use cues::{parse_cues, parse_cues_extended, RawCue};
pub use error::AnlzError;
pub use export::{compute_file_sha256, export_anlz, ExportIndex, ExportIndexTrack, ExportOptions};
pub use grid::{parse_pqtz, Beat, Grid};
pub use phrases::{parse_pssi, RawPhrase};
pub use waveform::{
    parse_pwv3, parse_pwv4, parse_pwv5, waveform_from_ext, WaveformDetail, WaveformPreview,
};

pub(crate) fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

pub(crate) fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}
