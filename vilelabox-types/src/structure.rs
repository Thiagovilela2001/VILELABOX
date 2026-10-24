use crate::error::TypeError;
use crate::units::Frame;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PhraseKind {
    Intro,
    Verse,
    Chorus,
    Bridge,
    Up,
    Down,
    Outro,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phrase {
    pub id: u32,
    pub start: Frame,
    pub end: Frame,
    pub kind: PhraseKind,
    pub beats: u32,
}

impl Phrase {
    pub fn new(
        id: u32,
        start: Frame,
        end: Frame,
        kind: PhraseKind,
        beats: u32,
    ) -> Result<Self, TypeError> {
        if start >= end {
            return Err(TypeError::InvalidInterval {
                start: start.get(),
                end: end.get(),
            });
        }
        Ok(Self {
            id,
            start,
            end,
            kind,
            beats,
        })
    }

    pub fn duration_frames(&self) -> Frame {
        self.end - self.start
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct VocalSegment {
    pub start: Frame,
    pub end: Frame,
    pub confidence: f32,
}

impl VocalSegment {
    pub fn new(start: Frame, end: Frame, confidence: f32) -> Result<Self, TypeError> {
        if start >= end {
            return Err(TypeError::InvalidInterval {
                start: start.get(),
                end: end.get(),
            });
        }
        Ok(Self {
            start,
            end,
            confidence: confidence.clamp(0.0, 1.0),
        })
    }

    pub fn duration_frames(&self) -> Frame {
        self.end - self.start
    }
}
