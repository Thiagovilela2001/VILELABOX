use crate::error::TypeError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum KeyScale {
    Major,
    Minor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MusicalKey {
    root_pitch_class: u8, // 0 = C, 1 = C#, 2 = D, ..., 11 = B
    scale: KeyScale,
}

impl MusicalKey {
    pub const fn new(root_pitch_class: u8, scale: KeyScale) -> Self {
        Self {
            root_pitch_class: root_pitch_class % 12,
            scale,
        }
    }

    pub fn root_pitch_class(self) -> u8 {
        self.root_pitch_class
    }

    pub fn scale(self) -> KeyScale {
        self.scale
    }

    pub fn to_camelot(self) -> &'static str {
        match (self.root_pitch_class, self.scale) {
            (11, KeyScale::Major) => "1B",  // B Major
            (8, KeyScale::Minor) => "1A",   // G# Minor / Ab Minor
            (6, KeyScale::Major) => "2B",   // F# Major
            (3, KeyScale::Minor) => "2A",   // D# Minor / Eb Minor
            (1, KeyScale::Major) => "3B",   // Db / C# Major
            (10, KeyScale::Minor) => "3A",  // Bb / A# Minor
            (8, KeyScale::Major) => "4B",   // Ab Major
            (5, KeyScale::Minor) => "4A",   // F Minor
            (3, KeyScale::Major) => "5B",   // Eb Major
            (0, KeyScale::Minor) => "5A",   // C Minor
            (10, KeyScale::Major) => "6B",  // Bb Major
            (7, KeyScale::Minor) => "6A",   // G Minor
            (5, KeyScale::Major) => "7B",   // F Major
            (2, KeyScale::Minor) => "7A",   // D Minor
            (0, KeyScale::Major) => "8B",   // C Major
            (9, KeyScale::Minor) => "8A",   // A Minor
            (7, KeyScale::Major) => "9B",   // G Major
            (4, KeyScale::Minor) => "9A",   // E Minor
            (2, KeyScale::Major) => "10B",  // D Major
            (11, KeyScale::Minor) => "10A", // B Minor
            (9, KeyScale::Major) => "11B",  // A Major
            (6, KeyScale::Minor) => "11A",  // F# Minor
            (4, KeyScale::Major) => "12B",  // E Major
            (1, KeyScale::Minor) => "12A",  // C# Minor
            _ => unreachable!(),
        }
    }

    pub fn to_open_key(self) -> &'static str {
        match (self.root_pitch_class, self.scale) {
            (0, KeyScale::Major) => "1d",
            (9, KeyScale::Minor) => "1m",
            (7, KeyScale::Major) => "2d",
            (4, KeyScale::Minor) => "2m",
            (2, KeyScale::Major) => "3d",
            (11, KeyScale::Minor) => "3m",
            (9, KeyScale::Major) => "4d",
            (6, KeyScale::Minor) => "4m",
            (4, KeyScale::Major) => "5d",
            (1, KeyScale::Minor) => "5m",
            (11, KeyScale::Major) => "6d",
            (8, KeyScale::Minor) => "6m",
            (6, KeyScale::Major) => "7d",
            (3, KeyScale::Minor) => "7m",
            (1, KeyScale::Major) => "8d",
            (10, KeyScale::Minor) => "8m",
            (8, KeyScale::Major) => "9d",
            (5, KeyScale::Minor) => "9m",
            (3, KeyScale::Major) => "10d",
            (0, KeyScale::Minor) => "10m",
            (10, KeyScale::Major) => "11d",
            (7, KeyScale::Minor) => "11m",
            (5, KeyScale::Major) => "12d",
            (2, KeyScale::Minor) => "12m",
            _ => unreachable!(),
        }
    }

    pub fn is_harmonic_compatible_with(self, other: Self) -> bool {
        let (num_a, mode_a) = parse_camelot_components(self.to_camelot());
        let (num_b, mode_b) = parse_camelot_components(other.to_camelot());

        if mode_a == mode_b {
            let diff = (num_a as i32 - num_b as i32).abs();
            diff == 0 || diff == 1 || diff == 11
        } else {
            num_a == num_b
        }
    }
}

fn parse_camelot_components(s: &str) -> (u8, char) {
    let mode = s.chars().last().unwrap();
    let num = s[..s.len() - 1].parse::<u8>().unwrap();
    (num, mode)
}

impl FromStr for MusicalKey {
    type Err = TypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        // Parse Camelot notation: 1A..12B
        if trimmed.len() >= 2 && trimmed.len() <= 3 {
            let (num_str, suffix) = (
                &trimmed[..trimmed.len() - 1],
                &trimmed[trimmed.len() - 1..].to_uppercase(),
            );
            if let Ok(num) = num_str.parse::<u8>() {
                if (1..=12).contains(&num) {
                    let scale = match suffix.as_str() {
                        "A" => KeyScale::Minor,
                        "B" => KeyScale::Major,
                        _ => return Err(TypeError::InvalidKey(s.to_string())),
                    };
                    return Ok(from_camelot_number_and_scale(num, scale));
                }
            }
        }

        // Parse Standard notation
        match trimmed.to_lowercase().as_str() {
            "c" | "c major" | "c maj" => Ok(MusicalKey::new(0, KeyScale::Major)),
            "c minor" | "c min" | "cm" => Ok(MusicalKey::new(0, KeyScale::Minor)),
            "c#" | "c# major" | "db" | "db major" => Ok(MusicalKey::new(1, KeyScale::Major)),
            "c# minor" | "c#m" | "db minor" | "dbm" => Ok(MusicalKey::new(1, KeyScale::Minor)),
            "d" | "d major" => Ok(MusicalKey::new(2, KeyScale::Major)),
            "d minor" | "dm" => Ok(MusicalKey::new(2, KeyScale::Minor)),
            "d#" | "eb" | "eb major" => Ok(MusicalKey::new(3, KeyScale::Major)),
            "d# minor" | "d#m" | "eb minor" | "ebm" => Ok(MusicalKey::new(3, KeyScale::Minor)),
            "e" | "e major" => Ok(MusicalKey::new(4, KeyScale::Major)),
            "e minor" | "em" => Ok(MusicalKey::new(4, KeyScale::Minor)),
            "f" | "f major" => Ok(MusicalKey::new(5, KeyScale::Major)),
            "f minor" | "fm" => Ok(MusicalKey::new(5, KeyScale::Minor)),
            "f#" | "f# major" | "gb" | "gb major" => Ok(MusicalKey::new(6, KeyScale::Major)),
            "f# minor" | "f#m" | "gb minor" | "gbm" => Ok(MusicalKey::new(6, KeyScale::Minor)),
            "g" | "g major" => Ok(MusicalKey::new(7, KeyScale::Major)),
            "g minor" | "gm" => Ok(MusicalKey::new(7, KeyScale::Minor)),
            "g#" | "ab" | "ab major" => Ok(MusicalKey::new(8, KeyScale::Major)),
            "g# minor" | "g#m" | "ab minor" | "abm" => Ok(MusicalKey::new(8, KeyScale::Minor)),
            "a" | "a major" => Ok(MusicalKey::new(9, KeyScale::Major)),
            "a minor" | "am" => Ok(MusicalKey::new(9, KeyScale::Minor)),
            "a#" | "bb" | "bb major" => Ok(MusicalKey::new(10, KeyScale::Major)),
            "a# minor" | "a#m" | "bb minor" | "bbm" => Ok(MusicalKey::new(10, KeyScale::Minor)),
            "b" | "b major" => Ok(MusicalKey::new(11, KeyScale::Major)),
            "b minor" | "bm" => Ok(MusicalKey::new(11, KeyScale::Minor)),
            _ => Err(TypeError::InvalidKey(s.to_string())),
        }
    }
}

fn from_camelot_number_and_scale(num: u8, scale: KeyScale) -> MusicalKey {
    let pitch_class = match (num, scale) {
        (1, KeyScale::Major) => 11,
        (1, KeyScale::Minor) => 8,
        (2, KeyScale::Major) => 6,
        (2, KeyScale::Minor) => 3,
        (3, KeyScale::Major) => 1,
        (3, KeyScale::Minor) => 10,
        (4, KeyScale::Major) => 8,
        (4, KeyScale::Minor) => 5,
        (5, KeyScale::Major) => 3,
        (5, KeyScale::Minor) => 0,
        (6, KeyScale::Major) => 10,
        (6, KeyScale::Minor) => 7,
        (7, KeyScale::Major) => 5,
        (7, KeyScale::Minor) => 2,
        (8, KeyScale::Major) => 0,
        (8, KeyScale::Minor) => 9,
        (9, KeyScale::Major) => 7,
        (9, KeyScale::Minor) => 4,
        (10, KeyScale::Major) => 2,
        (10, KeyScale::Minor) => 11,
        (11, KeyScale::Major) => 9,
        (11, KeyScale::Minor) => 6,
        (12, KeyScale::Major) => 4,
        (12, KeyScale::Minor) => 1,
        _ => unreachable!(),
    };
    MusicalKey::new(pitch_class, scale)
}

impl fmt::Display for MusicalKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_camelot())
    }
}
