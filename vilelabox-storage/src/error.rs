use thiserror::Error;

/// Erros do banco próprio.
#[derive(Error, Debug)]
pub enum ErroStorage {
    #[error("erro de E/S: {0}")]
    Io(#[from] std::io::Error),
    #[error("erro no banco de dados: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("migração inválida: {0}")]
    Migracao(String),
    #[error("dado inválido: {0}")]
    Validacao(String),
    #[error("invariante do domínio: {0}")]
    Tipo(#[from] vilelabox_types::TypeError),
}

/// Código estendido do SQLite para violação de chave estrangeira.
const SQLITE_CONSTRAINT_FOREIGNKEY: i32 = 787;
/// Código estendido do SQLite para violação de restrição UNIQUE.
const SQLITE_CONSTRAINT_UNIQUE: i32 = 2067;

/// Traduz violações de restrição para `Validacao`, com mensagem acionável;
/// os demais erros seguem como `Sqlite`.
pub(crate) fn mapear_constraint(e: rusqlite::Error) -> ErroStorage {
    match &e {
        rusqlite::Error::SqliteFailure(f, _) if f.extended_code == SQLITE_CONSTRAINT_FOREIGNKEY => {
            ErroStorage::Validacao("referência a registro inexistente".into())
        }
        rusqlite::Error::SqliteFailure(f, _) if f.extended_code == SQLITE_CONSTRAINT_UNIQUE => {
            ErroStorage::Validacao("valor duplicado (restrição UNIQUE)".into())
        }
        _ => ErroStorage::Sqlite(e),
    }
}
