use crate::error::TypeError;
use crate::units::Frame;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CueKind {
    CuePoint,
    HotCue,
    MemoryCue,
    Loop,
    FadeIn,
    FadeOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CueOrigin {
    User,
    Automatic,
    Imported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cue {
    pub id: Option<i64>,
    pub slot: Option<u8>,
    pub kind: CueKind,
    pub position: Frame,
    pub end_position: Option<Frame>,
    pub label: String,
    pub color: Option<u32>,
    pub origin: CueOrigin,
}

impl Cue {
    pub fn new(kind: CueKind, position: Frame, origin: CueOrigin) -> Self {
        Self {
            id: None,
            slot: None,
            kind,
            position,
            end_position: None,
            label: String::new(),
            color: None,
            origin,
        }
    }

    pub fn new_loop(
        position: Frame,
        end_position: Frame,
        origin: CueOrigin,
    ) -> Result<Self, TypeError> {
        if end_position <= position {
            return Err(TypeError::InvalidInterval {
                start: position.get(),
                end: end_position.get(),
            });
        }
        Ok(Self {
            id: None,
            slot: None,
            kind: CueKind::Loop,
            position,
            end_position: Some(end_position),
            label: String::new(),
            color: None,
            origin,
        })
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_slot(mut self, slot: u8) -> Self {
        self.slot = Some(slot);
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_color(mut self, color: u32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_end_position(mut self, end: Frame) -> Result<Self, TypeError> {
        if end <= self.position {
            return Err(TypeError::InvalidInterval {
                start: self.position.get(),
                end: end.get(),
            });
        }
        self.end_position = Some(end);
        Ok(self)
    }

    pub fn is_memory_cue(&self) -> bool {
        self.kind == CueKind::MemoryCue
    }

    pub fn is_hot_cue(&self) -> bool {
        self.kind == CueKind::HotCue
    }

    pub fn is_loop(&self) -> bool {
        self.kind == CueKind::Loop
    }
}
