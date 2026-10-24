use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum TypeError {
    #[error("Taxa de amostragem inválida: {0} Hz (deve ser > 0)")]
    InvalidSampleRate(u32),

    #[error("BPM inválido: {0} (deve ser finito e > 0.0)")]
    InvalidBpm(f64),

    #[error("Intervalo temporal inválido: início ({start}) >= fim ({end})")]
    InvalidInterval { start: i64, end: i64 },

    #[error("Duração em segundos inválida: {0} (deve ser finita e >= 0.0)")]
    InvalidSeconds(f64),

    #[error("Tom musical desconhecido ou inválido: '{0}'")]
    InvalidKey(String),

    #[error("Invariante de grid violada: {0}")]
    GridInvariantViolation(String),
}
