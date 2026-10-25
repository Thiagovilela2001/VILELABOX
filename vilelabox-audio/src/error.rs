use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("arquivo de áudio não encontrado: {0}")]
    FileNotFound(PathBuf),

    #[error("erro de E/S ao acessar áudio: {0}")]
    Io(#[from] std::io::Error),

    #[error("erro ao decodificar WAV: {0}")]
    Wav(#[from] hound::Error),

    #[error("formato de áudio não suportado: {0}")]
    UnsupportedFormat(String),

    #[error("arquivo de áudio corrompido ou truncado: {0}")]
    Corrupted(String),

    #[error("erro de dispositivo de áudio: {0}")]
    Device(String),
}
