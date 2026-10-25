//! Varredura de diretórios de áudio.
//!
//! Registra cada arquivo de áudio como faixa mínima (título/artista pelo nome
//! do arquivo) com sua localização. Arquivos já conhecidos são só marcados
//! como presentes; a leitura do conteúdo (identidade, metadados completos)
//! fica para as fases de análise (P4).

use crate::ErroLibrary;
use std::path::Path;
use vilelabox_storage::{Database, NovaFaixa};
use vilelabox_types::TrackMetadata;

/// Extensões reconhecidas como áudio na varredura (minúsculas).
pub const EXTENSOES: [&str; 7] = ["mp3", "wav", "flac", "aac", "m4a", "ogg", "aiff"];

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RelatorioScan {
    pub novos: usize,
    pub presentes: usize,
    pub ignorados: usize,
}

/// "Artista - Título" pelo nome do arquivo (mesma regra do app): separa no
/// primeiro " - "; sem ele, tudo é título.
fn titulo_artista(stem: &str) -> (String, Option<String>) {
    match stem.split_once(" - ") {
        Some((a, t)) if !a.trim().is_empty() && !t.trim().is_empty() => {
            (t.trim().to_string(), Some(a.trim().to_string()))
        }
        _ => (stem.trim().to_string(), None),
    }
}

/// Varre `dir` recursivamente, cadastrando arquivos de áudio novos e marcando
/// os conhecidos como presentes.
pub fn varrer_diretorio(db: &Database, dir: &Path) -> Result<RelatorioScan, ErroLibrary> {
    let mut rel = RelatorioScan::default();
    varrer(db, dir, &mut rel)?;
    Ok(rel)
}

fn varrer(db: &Database, dir: &Path, rel: &mut RelatorioScan) -> Result<(), ErroLibrary> {
    let mut entradas: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entradas.sort_by_key(|e| e.file_name());
    for e in entradas {
        let caminho = e.path();
        // `file_type` não segue links simbólicos: links viram "ignorados".
        if e.file_type()?.is_dir() {
            varrer(db, &caminho, rel)?;
            continue;
        }
        let ext = caminho
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !EXTENSOES.contains(&ext.as_str()) {
            rel.ignorados += 1;
            continue;
        }
        let chave = caminho.to_string_lossy().replace('\\', "/");
        if db.id_por_localizacao(&chave)?.is_some() {
            rel.presentes += 1;
            continue;
        }
        let stem = caminho
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("arquivo");
        let (titulo, artista) = titulo_artista(stem);
        let id = db.inserir_faixa(&NovaFaixa {
            metadados: TrackMetadata {
                title: Some(titulo),
                artist: artista,
                ..TrackMetadata::default()
            },
            ..NovaFaixa::default()
        })?;
        db.adicionar_localizacao(id, &chave)?;
        rel.novos += 1;
    }
    Ok(())
}
