pub mod container;
pub mod error;
pub mod grid;
pub mod phrases;

pub use container::{all_sections, parse_ppth, sections, FourCc};
pub use error::AnlzError;
pub use grid::{parse_pqtz, Beat, Grid};
pub use phrases::{parse_pssi, RawPhrase};

pub(crate) fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

pub(crate) fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}
