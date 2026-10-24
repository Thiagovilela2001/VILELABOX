use crate::error::ReferenceError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Versão do esquema canônico. Todo exportador (sintético, Mixxx e rekordbox)
/// grava este número; arquivos de outra versão são rejeitados.
pub const SCHEMA_VERSION: u32 = 1;

/// Frase musical. Tempos em segundos; `end` ausente quando a fonte não informa
/// o fim (última frase do rekordbox sem `end_beat` válido).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePhrase {
    pub start: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bars: Option<u32>,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
}

/// Cue, memory cue ou loop. Posição em segundos para preservar valores
/// fracionários e não depender da taxa de amostragem da fonte.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceCue {
    /// `cue`, `hotcue`, `memory`, `loop`, `intro`, `outro`.
    #[serde(rename = "type")]
    pub type_name: String,
    pub position_s: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hotcue: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceVocalSegment {
    pub start: f64,
    pub end: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
}

/// Trecho de andamento constante em faixas com `tempo_type = "variable"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceTempoSection {
    pub start: f64,
    pub end: f64,
    pub bpm: f64,
}

/// Resultado de referência de uma faixa. Campos `Option` ausentes significam
/// "a fonte não sabe"; o comparador só cobra o que a referência afirma.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceTrack {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub sha256: String,
    /// `synthetic`, `mixxx` ou `rekordbox_anlz`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Localizador dentro da fonte (ex.: caminho relativo do ANLZ), sem dados pessoais.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bpm: Option<f64>,
    #[serde(default = "default_tempo_type")]
    pub tempo_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<ReferenceTempoSection>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_beat_time_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downbeat_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// Energia da faixa na escala 1..=10 (`src/analyzer/analyzerenergy.cpp`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy: Option<f64>,
    #[serde(default)]
    pub phrases: Vec<ReferencePhrase>,
    #[serde(default)]
    pub cues: Vec<ReferenceCue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_vocal: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vocal_segments: Option<Vec<ReferenceVocalSegment>>,
}

fn default_tempo_type() -> String {
    "fixed".to_string()
}

/// Nó da árvore de playlists. A identidade é o caminho com a posição entre
/// irmãos, então nomes repetidos continuam distintos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePlaylist {
    pub name: String,
    /// SHA-256 das faixas, na ordem da playlist.
    #[serde(default)]
    pub tracks: Vec<String>,
    #[serde(default)]
    pub children: Vec<ReferencePlaylist>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePlaylistSet {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub playlists: Vec<ReferencePlaylist>,
}

fn check_version(version: u32) -> Result<(), ReferenceError> {
    if version != SCHEMA_VERSION {
        return Err(ReferenceError::InvalidSchema(format!(
            "schema_version {} (esperado {})",
            version, SCHEMA_VERSION
        )));
    }
    Ok(())
}

fn invalid(message: String) -> ReferenceError {
    ReferenceError::InvalidSchema(message)
}

fn finite(field: &str, value: f64) -> Result<(), ReferenceError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(invalid(format!("{} não é finito", field)))
    }
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
        track.validate()?;
        Ok(track)
    }

    pub fn to_json_str(&self) -> Result<String, ReferenceError> {
        let s = serde_json::to_string_pretty(self)?;
        Ok(s)
    }

    pub fn validate(&self) -> Result<(), ReferenceError> {
        check_version(self.schema_version)?;
        if self.sha256.len() != 64 || !self.sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(invalid(format!("sha256 inválido: {}", self.sha256)));
        }
        if let Some(bpm) = self.bpm {
            if !(bpm.is_finite() && bpm > 0.0) {
                return Err(invalid(format!("bpm inválido: {}", bpm)));
            }
        }
        if let Some(duration) = self.duration {
            if !(duration.is_finite() && duration >= 0.0) {
                return Err(invalid(format!("duration inválida: {}", duration)));
            }
        }
        if let Some(t) = self.first_beat_time_s {
            finite("first_beat_time_s", t)?;
        }
        if let Some(energy) = self.energy {
            if !(1.0..=10.0).contains(&energy) {
                return Err(invalid(format!("energy fora de 1..=10: {}", energy)));
            }
        }
        if self.downbeat_index.is_some_and(|i| i > 3) {
            return Err(invalid("downbeat_index fora de 0..=3".into()));
        }
        if !self.phrases.is_empty() && self.bpm.is_none() {
            return Err(invalid(
                "frases exigem bpm para a tolerância de compasso".into(),
            ));
        }
        for (i, p) in self.phrases.iter().enumerate() {
            finite("phrase.start", p.start)?;
            if let Some(end) = p.end {
                if !(end.is_finite() && end > p.start) {
                    return Err(invalid(format!("frase {} com fim <= início", i)));
                }
            }
        }
        for (i, c) in self.cues.iter().enumerate() {
            finite("cue.position_s", c.position_s)?;
            if let Some(len) = c.length_s {
                if !(len.is_finite() && len > 0.0) {
                    return Err(invalid(format!("cue {} com length_s <= 0", i)));
                }
            }
        }
        for (i, v) in self.vocal_segments.iter().flatten().enumerate() {
            if !(v.start.is_finite() && v.end.is_finite() && v.end > v.start) {
                return Err(invalid(format!("segmento vocal {} inválido", i)));
            }
        }
        Ok(())
    }
}

impl ReferencePlaylistSet {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ReferenceError> {
        let set: Self = serde_json::from_str(&fs::read_to_string(path)?)?;
        check_version(set.schema_version)?;
        Ok(set)
    }

    /// Lista `(caminho, faixas)` em pré-ordem. O caminho inclui o índice entre
    /// irmãos (`Sets[0]/Favoritas[1]`) para distinguir playlists homônimas.
    pub fn flatten(&self) -> Vec<(String, Vec<String>)> {
        fn walk(nodes: &[ReferencePlaylist], prefix: &str, out: &mut Vec<(String, Vec<String>)>) {
            for (i, node) in nodes.iter().enumerate() {
                let path = format!("{}{}[{}]", prefix, node.name, i);
                out.push((path.clone(), node.tracks.clone()));
                walk(&node.children, &format!("{}/", path), out);
            }
        }
        let mut out = Vec::new();
        walk(&self.playlists, "", &mut out);
        out
    }
}
