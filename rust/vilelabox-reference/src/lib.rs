pub mod comparator;
pub mod error;
pub mod schema;

pub use comparator::{ComparisonReport, DifferentialComparator, MetricResult, Tolerances};
pub use error::ReferenceError;
pub use schema::{ReferenceCue, ReferencePhrase, ReferenceTrack, ReferenceVocalSegment};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_real_synthetic_fixture() {
        let mut fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        fixture_path.pop(); // rust/
        fixture_path.pop(); // raiz repo/
        fixture_path.push("tests");
        fixture_path.push("reference");
        fixture_path.push("synthetic");
        fixture_path.push("fixed_120bpm.json");

        if fixture_path.exists() {
            let ref_track = ReferenceTrack::from_file(&fixture_path).unwrap();
            assert_eq!(ref_track.bpm, 120.0);
            assert_eq!(ref_track.sample_rate, 44100);
            assert_eq!(ref_track.phrases.len(), 2);
            assert_eq!(ref_track.cues.len(), 2);
            assert_eq!(ref_track.downbeat_index, Some(0));

            // Comparador consigo mesmo deve passar com 100%
            let comparator = DifferentialComparator::new();
            let report = comparator.compare(&ref_track, &ref_track);
            assert!(report.passed, "Relatório próprio deve passar integralmente");
        }
    }

    #[test]
    fn test_differential_comparator_tolerances_and_failures() {
        let expected = ReferenceTrack {
            name: Some("test_track".into()),
            sha256: "abc123".into(),
            sample_rate: 44100,
            duration: 60.0,
            bpm: 128.00,
            tempo_type: "fixed".into(),
            first_beat_time_s: Some(0.5000),
            first_beat_frame: Some(22050),
            downbeat_index: Some(0),
            key: Some("8A".into()),
            phrases: vec![ReferencePhrase {
                start: 0.0,
                end: 15.0,
                bars: Some(8),
                type_name: "INTRO".into(),
                confidence: Some(1.0),
            }],
            cues: vec![ReferenceCue {
                type_name: "hotcue".into(),
                position_frame: 22050,
                length_frame: None,
                hotcue: Some(0),
                label: Some("Drop".into()),
                color: None,
            }],
            has_vocal: false,
            vocal_segments: None,
        };

        let comparator = DifferentialComparator::new();

        // Caso 1: Dentro da tolerância de BPM (±0.005 BPM <= 0.01)
        let mut actual_close = expected.clone();
        actual_close.bpm = 128.005;
        let report_close = comparator.compare(&expected, &actual_close);
        assert!(report_close.passed);

        // Caso 2: Fora da tolerância de BPM (±0.05 BPM > 0.01)
        let mut actual_far = expected.clone();
        actual_far.bpm = 128.05;
        let report_far = comparator.compare(&expected, &actual_far);
        assert!(!report_far.passed);
        assert!(report_far
            .metrics
            .iter()
            .any(|m| m.metric == "bpm" && !m.passed));

        // Caso 3: Divergência de Downbeat (fase de compasso diferente)
        let mut actual_downbeat = expected.clone();
        actual_downbeat.downbeat_index = Some(1);
        let report_db = comparator.compare(&expected, &actual_downbeat);
        assert!(!report_db.passed);
        assert!(report_db
            .metrics
            .iter()
            .any(|m| m.metric == "downbeat_index" && !m.passed));
    }
}
