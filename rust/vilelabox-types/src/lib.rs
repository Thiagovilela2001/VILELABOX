pub mod cue;
pub mod error;
pub mod grid;
pub mod musical;
pub mod structure;
pub mod track;
pub mod units;

pub use cue::{Cue, CueKind, CueOrigin};
pub use error::TypeError;
pub use grid::{Beat, BeatGrid, TempoChange};
pub use musical::{KeyScale, MusicalKey};
pub use structure::{Phrase, PhraseKind, VocalSegment};
pub use track::{AudioIdentity, TrackId, TrackMetadata};
pub use units::{Bpm, EngineSample, Frame, SampleRate, Seconds};

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_units_conversion_and_validation() {
        let rate = SampleRate::new(44100).unwrap();
        assert_eq!(rate.get(), 44100);
        assert!(SampleRate::new(0).is_err());

        let frame = Frame::new(44100);
        let sec = frame.to_seconds(rate);
        assert_eq!(sec.get(), 1.0);
        assert_eq!(sec.to_ms(), 1000.0);

        let sec_back = Seconds::new(2.5).unwrap();
        let frame_back = sec_back.to_frame(rate);
        assert_eq!(frame_back.get(), 110250);

        let eng_samples = frame.to_engine_samples();
        assert_eq!(eng_samples.get(), 88200);
        assert_eq!(eng_samples.to_frame(), frame);

        assert!(Seconds::new(-0.1).is_err());
        assert!(Seconds::new(f64::NAN).is_err());

        let bpm = Bpm::new(128.0).unwrap();
        assert_eq!(bpm.get(), 128.0);
        assert_eq!(bpm.beat_duration_seconds().get(), 60.0 / 128.0);
        assert!(Bpm::new(0.0).is_err());
        assert!(Bpm::new(-120.0).is_err());
        assert!(Bpm::new(f64::INFINITY).is_err());
    }

    #[test]
    fn test_musical_key_camelot_and_harmony() {
        let c_maj = MusicalKey::from_str("C Major").unwrap();
        assert_eq!(c_maj.to_camelot(), "8B");
        assert_eq!(c_maj.to_open_key(), "1d");

        let a_min = MusicalKey::from_str("8A").unwrap();
        assert_eq!(a_min.to_camelot(), "8A");
        assert_eq!(a_min.to_open_key(), "1m");

        // Mesma posição de Camelot (8B e 8A) são compatíveis
        assert!(c_maj.is_harmonic_compatible_with(a_min));

        let g_maj = MusicalKey::from_str("9B").unwrap();
        // 8B e 9B são vizinhos (diferença de 1) -> compatíveis
        assert!(c_maj.is_harmonic_compatible_with(g_maj));

        let f_sharp_maj = MusicalKey::from_str("2B").unwrap();
        // 8B e 2B não são vizinhos -> incompatíveis
        assert!(!c_maj.is_harmonic_compatible_with(f_sharp_maj));
    }

    #[test]
    fn test_grid_invariants_and_lookup() {
        let rate = SampleRate::CD;
        let beats = vec![
            Beat {
                position: Frame::new(0),
                beat_num: 1,
            },
            Beat {
                position: Frame::new(22050),
                beat_num: 2,
            },
            Beat {
                position: Frame::new(44100),
                beat_num: 3,
            },
        ];
        let tempo = vec![TempoChange {
            position: Frame::new(0),
            bpm: Bpm::new(120.0).unwrap(),
        }];

        let grid = BeatGrid::new(rate, beats, tempo, Some(Frame::new(0))).unwrap();
        assert_eq!(grid.primary_bpm().unwrap().get(), 120.0);
        assert_eq!(grid.bpm_at(Frame::new(30000)).unwrap().get(), 120.0);

        // Closest beat
        let closest = grid.closest_beat(Frame::new(20000)).unwrap();
        assert_eq!(closest.position, Frame::new(22050));

        // Rejeitar batidas fora de ordem
        let unordered_beats = vec![
            Beat {
                position: Frame::new(100),
                beat_num: 1,
            },
            Beat {
                position: Frame::new(50),
                beat_num: 2,
            },
        ];
        assert!(BeatGrid::new(rate, unordered_beats, vec![], None).is_err());
    }

    #[test]
    fn test_phrase_and_vocal_interval_invariants() {
        let phrase = Phrase::new(
            1,
            Frame::new(0),
            Frame::new(44100 * 8),
            PhraseKind::Intro,
            32,
        )
        .unwrap();
        assert_eq!(phrase.duration_frames(), Frame::new(44100 * 8));

        // Início >= Fim deve falhar
        assert!(Phrase::new(2, Frame::new(100), Frame::new(100), PhraseKind::Chorus, 16).is_err());
        assert!(Phrase::new(3, Frame::new(100), Frame::new(50), PhraseKind::Up, 16).is_err());

        let vocal = VocalSegment::new(Frame::new(1000), Frame::new(2000), 0.95).unwrap();
        assert_eq!(vocal.confidence, 0.95);
        assert!(VocalSegment::new(Frame::new(2000), Frame::new(1000), 0.95).is_err());
    }

    #[test]
    fn test_cue_loop_invariants() {
        let hotcue = Cue::new(CueKind::HotCue, Frame::new(1000), CueOrigin::User)
            .with_id(1)
            .with_slot(0)
            .with_label("Drop")
            .with_color(0xFF0000);
        assert!(hotcue.is_hot_cue());

        let valid_loop = Cue::new_loop(Frame::new(1000), Frame::new(2000), CueOrigin::User)
            .unwrap()
            .with_id(2)
            .with_label("Loop A");
        assert!(valid_loop.is_loop());

        // Loop com end <= position deve falhar
        assert!(Cue::new_loop(Frame::new(2000), Frame::new(1000), CueOrigin::User).is_err());
        assert!(Cue::new_loop(Frame::new(1000), Frame::new(1000), CueOrigin::User).is_err());
    }
}
