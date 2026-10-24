use crate::error::TypeError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SampleRate(u32);

impl SampleRate {
    pub const CD: Self = Self(44_100);
    pub const PRO_AUDIO: Self = Self(48_000);
    pub const HIGH_RES: Self = Self(96_000);

    pub fn new(rate: u32) -> Result<Self, TypeError> {
        if rate == 0 {
            Err(TypeError::InvalidSampleRate(rate))
        } else {
            Ok(Self(rate))
        }
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for SampleRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} Hz", self.0)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct Frame(i64);

impl Frame {
    pub const ZERO: Self = Self(0);

    pub fn new(frame: i64) -> Self {
        Self(frame)
    }

    pub fn get(self) -> i64 {
        self.0
    }

    pub fn to_seconds(self, rate: SampleRate) -> Seconds {
        Seconds(self.0 as f64 / rate.get() as f64)
    }

    pub fn to_engine_samples(self) -> EngineSample {
        EngineSample(self.0 * 2)
    }
}

impl Add for Frame {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Frame {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} frames", self.0)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct EngineSample(i64);

impl EngineSample {
    pub fn new(samples: i64) -> Self {
        Self(samples)
    }

    pub fn get(self) -> i64 {
        self.0
    }

    pub fn to_frame(self) -> Frame {
        Frame(self.0 / 2)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Seconds(f64);

impl Seconds {
    pub const ZERO: Self = Self(0.0);

    pub fn new(seconds: f64) -> Result<Self, TypeError> {
        if !seconds.is_finite() || seconds < 0.0 {
            Err(TypeError::InvalidSeconds(seconds))
        } else {
            Ok(Self(seconds))
        }
    }

    pub fn get(self) -> f64 {
        self.0
    }

    pub fn to_frame(self, rate: SampleRate) -> Frame {
        Frame((self.0 * rate.get() as f64).round() as i64)
    }

    pub fn to_ms(self) -> f64 {
        self.0 * 1000.0
    }
}

impl fmt::Display for Seconds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3} s", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Bpm(f64);

impl Bpm {
    pub fn new(bpm: f64) -> Result<Self, TypeError> {
        if !bpm.is_finite() || bpm <= 0.0 {
            Err(TypeError::InvalidBpm(bpm))
        } else {
            Ok(Self(bpm))
        }
    }

    pub fn get(self) -> f64 {
        self.0
    }

    pub fn beat_duration_seconds(self) -> Seconds {
        Seconds(60.0 / self.0)
    }
}

impl fmt::Display for Bpm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} BPM", self.0)
    }
}
