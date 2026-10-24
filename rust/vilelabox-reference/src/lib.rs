pub mod comparator;
pub mod error;
pub mod schema;

pub use comparator::{ComparisonReport, DifferentialComparator, MetricResult, Tolerances};
pub use error::ReferenceError;
pub use schema::{
    ReferenceCue, ReferencePhrase, ReferencePlaylist, ReferencePlaylistSet, ReferenceTempoSection,
    ReferenceTrack, ReferenceVocalSegment, SCHEMA_VERSION,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn track() -> ReferenceTrack {
        ReferenceTrack {
            schema_version: SCHEMA_VERSION,
            name: Some("test_track".into()),
            sha256: "ab".repeat(32),
            source: Some("synthetic".into()),
            source_ref: None,
            sample_rate: Some(44100),
            duration: Some(60.0),
            bpm: Some(128.0),
            tempo_type: "fixed".into(),
            sections: None,
            first_beat_time_s: Some(0.5),
            downbeat_index: Some(0),
            key: Some("8A".into()),
            energy: Some(6.5),
            phrases: vec![ReferencePhrase {
                start: 0.0,
                end: Some(15.0),
                bars: Some(8),
                type_name: "INTRO".into(),
                confidence: Some(1.0),
            }],
            cues: vec![ReferenceCue {
                type_name: "loop".into(),
                position_s: 0.5,
                length_s: Some(1.875),
                hotcue: Some(0),
                label: Some("Drop".into()),
                color: None,
            }],
            has_vocal: Some(true),
            vocal_segments: Some(vec![ReferenceVocalSegment {
                start: 10.0,
                end: 20.0,
                confidence: None,
            }]),
        }
    }

    fn failed(report: &ComparisonReport, name: &str) -> bool {
        !report.passed
            && report
                .metrics
                .iter()
                .any(|m| m.metric.starts_with(name) && !m.passed)
    }

    #[test]
    fn bpm_downbeat_and_first_beat_tolerances() {
        let expected = track();
        let comparator = DifferentialComparator::new();
        assert!(comparator.compare(&expected, &expected).passed);

        let mut close = expected.clone();
        close.bpm = Some(128.005);
        assert!(comparator.compare(&expected, &close).passed);

        let mut far = expected.clone();
        far.bpm = Some(128.05);
        assert!(failed(&comparator.compare(&expected, &far), "bpm"));

        let mut downbeat = expected.clone();
        downbeat.downbeat_index = Some(1);
        assert!(failed(
            &comparator.compare(&expected, &downbeat),
            "downbeat_index"
        ));

        // 5 ms passa só na tolerância de áudio real.
        let mut late = expected.clone();
        late.first_beat_time_s = Some(0.505);
        assert!(failed(
            &comparator.compare(&expected, &late),
            "first_beat_time_s"
        ));
        let real = DifferentialComparator::with_tolerances(Tolerances::for_source(None));
        assert!(real.compare(&expected, &late).passed);
    }

    #[test]
    fn missing_fields_fail_only_when_reference_defines_them() {
        let expected = track();
        let comparator = DifferentialComparator::new();

        let mut energy = expected.clone();
        energy.energy = Some(6.6);
        assert!(failed(&comparator.compare(&expected, &energy), "energy"));
        energy.energy = Some(11.0);
        assert!(energy.validate().is_err());

        let mut no_bpm = expected.clone();
        no_bpm.bpm = None;
        assert!(failed(&comparator.compare(&expected, &no_bpm), "bpm"));

        // Referência sem vocal (rekordbox) não cobra o resultado.
        let mut reference = expected.clone();
        reference.has_vocal = None;
        reference.vocal_segments = None;
        let mut actual = expected.clone();
        actual.has_vocal = Some(false);
        assert!(comparator.compare(&reference, &actual).passed);
    }

    #[test]
    fn cue_type_slot_and_loop_length_are_compared() {
        let expected = track();
        let comparator = DifferentialComparator::new();

        let mut fractional = expected.clone();
        fractional.cues[0].position_s += 0.0004;
        assert!(comparator.compare(&expected, &fractional).passed);

        let mut slot = expected.clone();
        slot.cues[0].hotcue = Some(1);
        assert!(failed(&comparator.compare(&expected, &slot), "cue_0"));

        let mut kind = expected.clone();
        kind.cues[0].type_name = "hotcue".into();
        assert!(failed(&comparator.compare(&expected, &kind), "cue_0"));

        let mut length = expected.clone();
        length.cues[0].length_s = Some(3.75);
        assert!(failed(&comparator.compare(&expected, &length), "cue_0"));
    }

    #[test]
    fn phrase_and_vocal_boundaries() {
        let expected = track();
        let comparator = DifferentialComparator::new();

        // 1 compasso a 128 BPM = 1,875 s.
        let mut phrase = expected.clone();
        phrase.phrases[0].end = Some(16.8);
        assert!(comparator.compare(&expected, &phrase).passed);
        phrase.phrases[0].end = Some(17.0);
        assert!(failed(&comparator.compare(&expected, &phrase), "phrase_0"));

        let mut vocal = expected.clone();
        vocal.vocal_segments.as_mut().unwrap()[0].start = 10.3;
        assert!(failed(
            &comparator.compare(&expected, &vocal),
            "vocal_segment_0"
        ));
    }

    #[test]
    fn validation_rejects_wrong_version_and_unknown_fields() {
        let json = track().to_json_str().unwrap();
        assert!(ReferenceTrack::from_json_str(&json).is_ok());

        let old = json.replace("\"schema_version\": 1", "\"schema_version\": 0");
        assert!(ReferenceTrack::from_json_str(&old).is_err());

        let extra = json.replacen('{', "{\"start_time_s\": 1.0,", 1);
        assert!(ReferenceTrack::from_json_str(&extra).is_err());

        let mut bad = track();
        bad.bpm = Some(f64::NAN);
        assert!(bad.validate().is_err());
    }

    #[test]
    fn playlists_distinguish_homonyms_and_order() {
        let leaf = |name: &str, tracks: &[&str]| ReferencePlaylist {
            name: name.into(),
            tracks: tracks.iter().map(|t| t.to_string()).collect(),
            children: vec![],
        };
        let expected = ReferencePlaylistSet {
            schema_version: SCHEMA_VERSION,
            name: None,
            playlists: vec![ReferencePlaylist {
                name: "Sets".into(),
                tracks: vec![],
                children: vec![leaf("Dup", &["a"]), leaf("Dup", &["b"])],
            }],
        };
        let comparator = DifferentialComparator::new();
        assert!(comparator.compare_playlists(&expected, &expected).passed);

        let mut swapped = expected.clone();
        swapped.playlists[0].children.swap(0, 1);
        assert!(!comparator.compare_playlists(&expected, &swapped).passed);

        let mut dropped = expected.clone();
        dropped.playlists[0].children.pop();
        assert!(!comparator.compare_playlists(&expected, &dropped).passed);
    }
}
