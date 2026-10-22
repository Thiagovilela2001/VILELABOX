use thiserror::Error;

#[derive(Error, Debug)]
pub enum MixxxCompatError {
    #[error("Erro de banco SQLite do Mixxx: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Erro de conversão para vilelabox-types: {0}")]
    TypeConversion(#[from] vilelabox_types::TypeError),

    #[error("Formato beats.proto corrompido ou inesperado: {0}")]
    InvalidBeatsProto(String),

    #[error("Faixa não encontrada: ID {0}")]
    TrackNotFound(i64),
}
