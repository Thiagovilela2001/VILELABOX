use crate::schema::{ReferencePlaylistSet, ReferenceTrack};
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

/// Tolerâncias de `tests/reference/README.md`.
pub struct Tolerances {
    pub bpm: f64,                // 0,01 BPM
    pub first_beat_seconds: f64, // 1 ms (sintético) / 25 ms (áudio real)
    pub cue_position_ms: f64,    // 1 ms
    pub vocal_boundary_s: f64,   // 250 ms
    pub energy: f64,             // 0,05 na escala 1..=10
}

impl Default for Tolerances {
    fn default() -> Self {
        Self {
            bpm: 0.01,
            first_beat_seconds: 0.001,
            cue_position_ms: 1.0,
            vocal_boundary_s: 0.25,
            energy: 0.05,
        }
    }
}

impl Tolerances {
    /// Áudio real complexo: alinhamento de grid até 25 ms.
    pub fn real_audio() -> Self {
        Self {
            first_beat_seconds: 0.025,
            ..Self::default()
        }
    }

    /// Escolhe pelo campo `source` da referência: só `synthetic` usa 1 ms.
    pub fn for_source(source: Option<&str>) -> Self {
        match source {
            Some("synthetic") => Self::default(),
            _ => Self::real_audio(),
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

fn metric(
    metric: impl Into<String>,
    expected: impl Into<String>,
    actual: impl Into<String>,
    tolerance: impl Into<String>,
    passed: bool,
    details: Option<String>,
) -> MetricResult {
    MetricResult {
        metric: metric.into(),
        expected: expected.into(),
        actual: actual.into(),
        tolerance: tolerance.into(),
        passed,
        details,
    }
}

fn missing(name: impl Into<String>, expected: String) -> MetricResult {
    metric(
        name,
        expected,
        "ausente",
        "presença",
        false,
        Some("Referência define o valor; resultado não".to_string()),
    )
}

fn count_mismatch(name: &str, expected: usize, actual: usize) -> MetricResult {
    metric(
        format!("{}_count", name),
        expected.to_string(),
        actual.to_string(),
        "exato",
        false,
        Some(format!("Quantidade de {} divergente", name)),
    )
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

    /// Compara `actual` com a referência `expected`. Campos ausentes na
    /// referência não são cobrados; presentes na referência e ausentes no
    /// resultado falham. A ordem de frases, cues e segmentos é significativa.
    pub fn compare(&self, expected: &ReferenceTrack, actual: &ReferenceTrack) -> ComparisonReport {
        let tol = &self.tolerances;
        let mut metrics = Vec::new();

        // 1. BPM
        if let Some(exp) = expected.bpm {
            match actual.bpm {
                None => metrics.push(missing("bpm", format!("{:.2}", exp))),
                Some(act) => {
                    let diff = (exp - act).abs();
                    let passed = diff <= tol.bpm;
                    metrics.push(metric(
                        "bpm",
                        format!("{:.2}", exp),
                        format!("{:.2}", act),
                        format!("±{:.2}", tol.bpm),
                        passed,
                        (!passed)
                            .then(|| format!("Diferença de {:.4} BPM excede tolerância", diff)),
                    ));
                }
            }
        }

        // 2. Seções de tempo (faixas de andamento variável)
        if let Some(exp_sections) = &expected.sections {
            let act_sections = actual.sections.as_deref().unwrap_or_default();
            if exp_sections.len() != act_sections.len() {
                metrics.push(count_mismatch(
                    "sections",
                    exp_sections.len(),
                    act_sections.len(),
                ));
            } else {
                for (i, (e, a)) in exp_sections.iter().zip(act_sections).enumerate() {
                    let passed = (e.bpm - a.bpm).abs() <= tol.bpm;
                    metrics.push(metric(
                        format!("section_{}_bpm", i),
                        format!("{:.2}", e.bpm),
                        format!("{:.2}", a.bpm),
                        format!("±{:.2}", tol.bpm),
                        passed,
                        None,
                    ));
                }
            }
        }

        // 3. Downbeat (fase do compasso)
        if let Some(exp) = expected.downbeat_index {
            match actual.downbeat_index {
                None => metrics.push(missing("downbeat_index", exp.to_string())),
                Some(act) => metrics.push(metric(
                    "downbeat_index",
                    exp.to_string(),
                    act.to_string(),
                    "exato (0)",
                    exp == act,
                    (exp != act)
                        .then(|| format!("Fase de compasso divergente: {} != {}", exp, act)),
                )),
            }
        }

        // 4. Primeira batida
        if let Some(exp) = expected.first_beat_time_s {
            match actual.first_beat_time_s {
                None => metrics.push(missing("first_beat_time_s", format!("{:.4} s", exp))),
                Some(act) => {
                    let diff = (exp - act).abs();
                    let passed = diff <= tol.first_beat_seconds;
                    metrics.push(metric(
                        "first_beat_time_s",
                        format!("{:.4} s", exp),
                        format!("{:.4} s", act),
                        format!("±{:.4} s", tol.first_beat_seconds),
                        passed,
                        (!passed).then(|| format!("Erro temporal de {:.4} s", diff)),
                    ));
                }
            }
        }

        // 5. Energia
        if let Some(exp) = expected.energy {
            match actual.energy {
                None => metrics.push(missing("energy", format!("{:.2}", exp))),
                Some(act) => metrics.push(metric(
                    "energy",
                    format!("{:.2}", exp),
                    format!("{:.2}", act),
                    format!("±{:.2}", tol.energy),
                    (exp - act).abs() <= tol.energy,
                    None,
                )),
            }
        }

        // 6. Frases: limites até 1 compasso. `validate` garante bpm com frases.
        if expected.phrases.len() != actual.phrases.len() {
            metrics.push(count_mismatch(
                "phrases",
                expected.phrases.len(),
                actual.phrases.len(),
            ));
        } else if !expected.phrases.is_empty() {
            let bar_s = 240.0 / expected.bpm.unwrap_or(f64::INFINITY);
            let mut all_match = true;
            for (i, (e, a)) in expected.phrases.iter().zip(&actual.phrases).enumerate() {
                let start_ok = (e.start - a.start).abs() <= bar_s;
                let end_ok = match (e.end, a.end) {
                    (Some(ee), Some(ae)) => (ee - ae).abs() <= bar_s,
                    (Some(_), None) => false,
                    (None, _) => true,
                };
                if !(start_ok && end_ok) {
                    all_match = false;
                    metrics.push(metric(
                        format!("phrase_{}_boundary", i),
                        format!("[{:.2}, {:?}]", e.start, e.end),
                        format!("[{:.2}, {:?}]", a.start, a.end),
                        format!("até 1 compasso ({:.2} s)", bar_s),
                        false,
                        Some("Limite da frase excede 1 compasso".to_string()),
                    ));
                }
            }
            if all_match {
                metrics.push(metric(
                    "phrases_boundaries",
                    format!("{} frases", expected.phrases.len()),
                    format!("{} frases", actual.phrases.len()),
                    "até 1 compasso",
                    true,
                    None,
                ));
            }
        }

        // 7. Cues: tipo, slot, posição e duração de loop
        if expected.cues.len() != actual.cues.len() {
            metrics.push(count_mismatch(
                "cues",
                expected.cues.len(),
                actual.cues.len(),
            ));
        } else if !expected.cues.is_empty() {
            let tol_s = tol.cue_position_ms / 1000.0;
            let mut all_match = true;
            for (i, (e, a)) in expected.cues.iter().zip(&actual.cues).enumerate() {
                let length_ok = match (e.length_s, a.length_s) {
                    (Some(el), Some(al)) => (el - al).abs() <= tol_s,
                    (None, None) => true,
                    _ => false,
                };
                let ok = e.type_name == a.type_name
                    && e.hotcue == a.hotcue
                    && (e.position_s - a.position_s).abs() <= tol_s
                    && length_ok;
                if !ok {
                    all_match = false;
                    metrics.push(metric(
                        format!("cue_{}", i),
                        format!(
                            "{} slot {:?} em {:.6} s, duração {:?}",
                            e.type_name, e.hotcue, e.position_s, e.length_s
                        ),
                        format!(
                            "{} slot {:?} em {:.6} s, duração {:?}",
                            a.type_name, a.hotcue, a.position_s, a.length_s
                        ),
                        format!("tipo e slot exatos, ±{:.1} ms", tol.cue_position_ms),
                        false,
                        None,
                    ));
                }
            }
            if all_match {
                metrics.push(metric(
                    "cues_alignment",
                    format!("{} cues", expected.cues.len()),
                    format!("{} cues", actual.cues.len()),
                    format!("±{:.1} ms", tol.cue_position_ms),
                    true,
                    None,
                ));
            }
        }

        // 8. Vocal
        if let Some(exp) = expected.has_vocal {
            match actual.has_vocal {
                None => metrics.push(missing("has_vocal", exp.to_string())),
                Some(act) => metrics.push(metric(
                    "has_vocal",
                    exp.to_string(),
                    act.to_string(),
                    "exato",
                    exp == act,
                    None,
                )),
            }
        }
        if let Some(exp_segments) = &expected.vocal_segments {
            let act_segments = actual.vocal_segments.as_deref().unwrap_or_default();
            if exp_segments.len() != act_segments.len() {
                metrics.push(count_mismatch(
                    "vocal_segments",
                    exp_segments.len(),
                    act_segments.len(),
                ));
            } else {
                for (i, (e, a)) in exp_segments.iter().zip(act_segments).enumerate() {
                    let passed = (e.start - a.start).abs() <= tol.vocal_boundary_s
                        && (e.end - a.end).abs() <= tol.vocal_boundary_s;
                    metrics.push(metric(
                        format!("vocal_segment_{}", i),
                        format!("[{:.2}, {:.2}]", e.start, e.end),
                        format!("[{:.2}, {:.2}]", a.start, a.end),
                        format!("±{:.2} s nas bordas", tol.vocal_boundary_s),
                        passed,
                        None,
                    ));
                }
            }
        }

        ComparisonReport {
            name: expected.name.clone(),
            sha256: expected.sha256.clone(),
            passed: metrics.iter().all(|m| m.passed),
            metrics,
        }
    }

    /// Playlists: caminho, ordem entre irmãos e ordem das faixas são exatos.
    pub fn compare_playlists(
        &self,
        expected: &ReferencePlaylistSet,
        actual: &ReferencePlaylistSet,
    ) -> MetricResult {
        let exp = expected.flatten();
        let act = actual.flatten();
        let first_diff = exp
            .iter()
            .zip(&act)
            .find(|(e, a)| e != a)
            .map(|(e, a)| format!("{} {:?} != {} {:?}", e.0, e.1, a.0, a.1))
            .or_else(|| {
                (exp.len() != act.len()).then(|| format!("{} nós != {} nós", exp.len(), act.len()))
            });
        metric(
            "playlists",
            format!("{} nós", exp.len()),
            format!("{} nós", act.len()),
            "exato (caminho, ordem e faixas)",
            first_diff.is_none(),
            first_diff,
        )
    }
}
