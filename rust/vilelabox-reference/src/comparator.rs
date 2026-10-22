use crate::schema::ReferenceTrack;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricResult {
    pub metric: String,
    pub expected: String,
    pub actual: String,
    pub tolerance: String,
    pub passed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparisonReport {
    pub name: Option<String>,
    pub sha256: String,
    pub passed: bool,
    pub metrics: Vec<MetricResult>,
}

pub struct Tolerances {
    pub bpm: f64,                // Padrão: 0.01 BPM
    pub first_beat_seconds: f64, // Padrão: 0.001 s (1 ms)
    pub cue_position_ms: f64,    // Padrão: 1.0 ms
    pub vocal_boundary_s: f64,   // Padrão: 0.25 s (250 ms)
}

impl Default for Tolerances {
    fn default() -> Self {
        Self {
            bpm: 0.01,
            first_beat_seconds: 0.001,
            cue_position_ms: 1.0,
            vocal_boundary_s: 0.25,
        }
    }
}

pub struct DifferentialComparator {
    tolerances: Tolerances,
}

impl Default for DifferentialComparator {
    fn default() -> Self {
        Self::new()
    }
}

impl DifferentialComparator {
    pub fn new() -> Self {
        Self {
            tolerances: Tolerances::default(),
        }
    }

    pub fn with_tolerances(tolerances: Tolerances) -> Self {
        Self { tolerances }
    }

    pub fn compare(&self, expected: &ReferenceTrack, actual: &ReferenceTrack) -> ComparisonReport {
        let mut metrics = Vec::new();

        // 1. BPM
        let bpm_diff = (expected.bpm - actual.bpm).abs();
        let bpm_passed = bpm_diff <= self.tolerances.bpm;
        metrics.push(MetricResult {
            metric: "bpm".to_string(),
            expected: format!("{:.2}", expected.bpm),
            actual: format!("{:.2}", actual.bpm),
            tolerance: format!("±{:.2}", self.tolerances.bpm),
            passed: bpm_passed,
            details: if !bpm_passed {
                Some(format!(
                    "Diferença de {:.4} BPM excede tolerância",
                    bpm_diff
                ))
            } else {
                None
            },
        });

        // 2. Downbeat index
        if let (Some(exp_db), Some(act_db)) = (expected.downbeat_index, actual.downbeat_index) {
            let passed = exp_db == act_db;
            metrics.push(MetricResult {
                metric: "downbeat_index".to_string(),
                expected: exp_db.to_string(),
                actual: act_db.to_string(),
                tolerance: "exato (0)".to_string(),
                passed,
                details: if !passed {
                    Some(format!(
                        "Fase de compasso divergente: {} != {}",
                        exp_db, act_db
                    ))
                } else {
                    None
                },
            });
        }

        // 3. First beat time
        if let (Some(exp_fb), Some(act_fb)) = (expected.first_beat_time_s, actual.first_beat_time_s)
        {
            let diff_s = (exp_fb - act_fb).abs();
            let passed = diff_s <= self.tolerances.first_beat_seconds;
            metrics.push(MetricResult {
                metric: "first_beat_time_s".to_string(),
                expected: format!("{:.4} s", exp_fb),
                actual: format!("{:.4} s", act_fb),
                tolerance: format!("±{:.4} s (1 ms)", self.tolerances.first_beat_seconds),
                passed,
                details: if !passed {
                    Some(format!("Erro temporal de {:.4} s excede 1 ms", diff_s))
                } else {
                    None
                },
            });
        }

        // 4. Frases (limites)
        let phrases_passed = if expected.phrases.len() != actual.phrases.len() {
            metrics.push(MetricResult {
                metric: "phrases_count".to_string(),
                expected: expected.phrases.len().to_string(),
                actual: actual.phrases.len().to_string(),
                tolerance: "exato".to_string(),
                passed: false,
                details: Some("Quantidade de frases divergente".to_string()),
            });
            false
        } else {
            let mut all_match = true;
            let bar_duration_s = (60.0 / expected.bpm) * 4.0;
            for (i, (exp_p, act_p)) in expected.phrases.iter().zip(&actual.phrases).enumerate() {
                let start_diff = (exp_p.start - act_p.start).abs();
                let end_diff = (exp_p.end - act_p.end).abs();
                if start_diff > bar_duration_s || end_diff > bar_duration_s {
                    all_match = false;
                    metrics.push(MetricResult {
                        metric: format!("phrase_{}_boundary", i),
                        expected: format!("[{:.2}, {:.2}]", exp_p.start, exp_p.end),
                        actual: format!("[{:.2}, {:.2}]", act_p.start, act_p.end),
                        tolerance: format!("até 1 compasso ({:.2} s)", bar_duration_s),
                        passed: false,
                        details: Some(
                            "Limite da frase excede 1 compasso de tolerância".to_string(),
                        ),
                    });
                }
            }
            all_match
        };
        if phrases_passed && !expected.phrases.is_empty() {
            metrics.push(MetricResult {
                metric: "phrases_boundaries".to_string(),
                expected: format!("{} frases", expected.phrases.len()),
                actual: format!("{} frases", actual.phrases.len()),
                tolerance: "até 1 compasso".to_string(),
                passed: true,
                details: None,
            });
        }

        // 5. Cues
        let cues_passed = if expected.cues.len() != actual.cues.len() {
            metrics.push(MetricResult {
                metric: "cues_count".to_string(),
                expected: expected.cues.len().to_string(),
                actual: actual.cues.len().to_string(),
                tolerance: "exato".to_string(),
                passed: false,
                details: Some("Quantidade de cues divergente".to_string()),
            });
            false
        } else {
            let mut all_match = true;
            let sample_rate = expected.sample_rate.max(1) as f64;
            for (i, (exp_c, act_c)) in expected.cues.iter().zip(&actual.cues).enumerate() {
                let diff_frames = (exp_c.position_frame - act_c.position_frame).abs();
                let diff_ms = (diff_frames as f64 / sample_rate) * 1000.0;
                if diff_ms > self.tolerances.cue_position_ms {
                    all_match = false;
                    metrics.push(MetricResult {
                        metric: format!("cue_{}_position", i),
                        expected: format!("frame {}", exp_c.position_frame),
                        actual: format!("frame {}", act_c.position_frame),
                        tolerance: format!("±{:.1} ms", self.tolerances.cue_position_ms),
                        passed: false,
                        details: Some(format!("Desvio de {:.2} ms excede tolerância", diff_ms)),
                    });
                }
            }
            all_match
        };
        if cues_passed && !expected.cues.is_empty() {
            metrics.push(MetricResult {
                metric: "cues_alignment".to_string(),
                expected: format!("{} cues", expected.cues.len()),
                actual: format!("{} cues", actual.cues.len()),
                tolerance: format!("±{:.1} ms", self.tolerances.cue_position_ms),
                passed: true,
                details: None,
            });
        }

        // 6. Has Vocal
        metrics.push(MetricResult {
            metric: "has_vocal".to_string(),
            expected: expected.has_vocal.to_string(),
            actual: actual.has_vocal.to_string(),
            tolerance: "exato".to_string(),
            passed: expected.has_vocal == actual.has_vocal,
            details: None,
        });

        let overall_passed = metrics.iter().all(|m| m.passed);

        ComparisonReport {
            name: expected.name.clone(),
            sha256: expected.sha256.clone(),
            passed: overall_passed,
            metrics,
        }
    }
}
