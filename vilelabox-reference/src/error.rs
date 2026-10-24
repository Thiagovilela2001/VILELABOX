use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReferenceError {
    #[error("Erro de serialização ou parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Erro de I/O na referência: {0}")]
    Io(#[from] std::io::Error),

    #[error("Erro de conversão de tipos de domínio: {0}")]
    TypeConversion(#[from] vilelabox_types::TypeError),

    #[error("Schema inválido ou versão incompatível: {0}")]
    InvalidSchema(String),
}
