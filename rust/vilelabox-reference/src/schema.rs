use crate::error::ReferenceError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferencePhrase {
    pub start: f64,
    pub end: f64,
    #[serde(default)]
    pub bars: Option<u32>,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub confidence: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferenceCue {
    #[serde(rename = "type")]
    pub type_name: String,
    pub position_frame: i64,
    #[serde(default)]
    pub length_frame: Option<i64>,
    #[serde(default)]
    pub hotcue: Option<i32>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub color: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferenceVocalSegment {
    pub start: f64,
    pub end: f64,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferenceTrack {
    #[serde(default)]
    pub name: Option<String>,
    pub sha256: String,
    pub sample_rate: u32,
    pub duration: f64,
    pub bpm: f64,
    #[serde(default = "default_tempo_type")]
    pub tempo_type: String,
    #[serde(default)]
    pub first_beat_time_s: Option<f64>,
    #[serde(default)]
    pub first_beat_frame: Option<i64>,
    #[serde(default)]
    pub downbeat_index: Option<u32>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub phrases: Vec<ReferencePhrase>,
    #[serde(default)]
    pub cues: Vec<ReferenceCue>,
    #[serde(default)]
    pub has_vocal: bool,
    #[serde(default)]
    pub vocal_segments: Option<Vec<ReferenceVocalSegment>>,
}

fn default_tempo_type() -> String {
    "fixed".to_string()
}

impl ReferenceTrack {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ReferenceError> {
        let content = fs::read_to_string(path)?;
        Self::from_json_str(&content)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ReferenceError> {
        let content = self.to_json_str()?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn from_json_str(s: &str) -> Result<Self, ReferenceError> {
        let track: Self = serde_json::from_str(s)?;
        Ok(track)
    }

    pub fn to_json_str(&self) -> Result<String, ReferenceError> {
        let s = serde_json::to_string_pretty(self)?;
        Ok(s)
    }
}
