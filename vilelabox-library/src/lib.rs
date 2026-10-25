//! `vilelabox-library`: biblioteca do VILELABOX sobre o banco próprio (P3.2).
//!
//! Importadores chamam os adaptadores de formato e escrevem no
//! `vilelabox-storage`: catálogo do rekordbox (`spec/rekordbox/10-master-db.md`,
//! via `vilelabox-catalog`), documento de intercâmbio XML
//! (`spec/rekordbox/12-xml.md`, via `vilelabox-xml`) e varredura de diretórios
//! de áudio. Sobre os dados persistidos ficam a busca textual, a ordenação
//! por coluna, as playlists e a identidade por conteúdo (SHA-256), com
//! detecção de arquivos ausentes, movidos e duplicados.
//!
//! Formatos externos não definem o modelo interno: terminam nestes
//! importadores. Nenhuma escrita é feita fora do banco próprio.

mod busca;
mod identidade;
mod importar;
mod scan;

pub use busca::{buscar, Coluna, Consulta};
pub use identidade::{
    buscar_movidos, duplicatas, nome_normalizado, sha256_arquivo, verificar_localizacoes,
};
pub use importar::{importar_catalogo, importar_xml, Relatorio};
pub use scan::{varrer_diretorio, RelatorioScan};

/// Erros da biblioteca.
#[derive(Debug, thiserror::Error)]
pub enum ErroLibrary {
    #[error("erro de E/S: {0}")]
    Io(#[from] std::io::Error),
    #[error("erro no banco próprio: {0}")]
    Storage(#[from] vilelabox_storage::ErroStorage),
    #[error("erro no catálogo: {0}")]
    Catalogo(#[from] vilelabox_catalog::ErroCatalogo),
    #[error("erro no XML: {0}")]
    Xml(#[from] vilelabox_xml::XmlError),
    #[error("invariante do domínio: {0}")]
    Tipo(#[from] vilelabox_types::TypeError),
}
