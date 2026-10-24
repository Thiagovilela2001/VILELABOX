use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnlzError {
    #[error("erro de E/S: {0}")]
    Io(#[from] std::io::Error),
    #[error("erro de referência: {0}")]
    Referencia(#[from] vilelabox_reference::ReferenceError),
    #[error("inconsistência nos dados ANLZ: {0}")]
    Inconsistente(String),
}
