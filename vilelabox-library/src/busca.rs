//! Busca textual, ordenação e filtros sobre o banco.

use crate::ErroLibrary;
use std::cmp::Ordering;
use vilelabox_storage::{Database, FaixaRow};

/// Coluna de ordenação (espelha as colunas clicáveis do app).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coluna {
    Id,
    Titulo,
    Artista,
    Bpm,
    Duracao,
    Estrelas,
}

/// Consulta à biblioteca: texto livre (caixa-insensível) em título e artista,
/// universo da coleção inteira ou de uma playlist, e coluna de ordenação com
/// desempate estável pelo `id`.
#[derive(Debug, Clone, Default)]
pub struct Consulta {
    pub texto: String,
    pub coluna: Option<Coluna>,
    pub ascendente: bool,
    pub lista: Option<i64>,
}

/// Busca as faixas da consulta, já ordenadas.
pub fn buscar(db: &Database, consulta: &Consulta) -> Result<Vec<FaixaRow>, ErroLibrary> {
    let mut faixas = db.listar_faixas(consulta.lista)?;
    let termo = consulta.texto.trim().to_lowercase();
    if !termo.is_empty() {
        faixas.retain(|f| {
            chave(&f.metadados.title).contains(&termo)
                || chave(&f.metadados.artist).contains(&termo)
        });
    }
    if let Some(coluna) = consulta.coluna {
        let asc = consulta.ascendente;
        faixas.sort_by(|a, b| {
            let ord = match coluna {
                Coluna::Id => a.id.cmp(&b.id),
                Coluna::Titulo => chave(&a.metadados.title).cmp(&chave(&b.metadados.title)),
                Coluna::Artista => chave(&a.metadados.artist).cmp(&chave(&b.metadados.artist)),
                Coluna::Bpm => a
                    .bpm
                    .map(|x| x.get())
                    .partial_cmp(&b.bpm.map(|x| x.get()))
                    .unwrap_or(Ordering::Equal),
                Coluna::Duracao => duracao(a).cmp(&duracao(b)),
                Coluna::Estrelas => a.estrelas.cmp(&b.estrelas),
            };
            let ord = ord.then_with(|| a.id.cmp(&b.id));
            if asc {
                ord
            } else {
                ord.reverse()
            }
        });
    }
    Ok(faixas)
}

fn chave(s: &Option<String>) -> String {
    s.as_deref().unwrap_or("").to_lowercase()
}

fn duracao(f: &FaixaRow) -> i64 {
    f.identidade
        .as_ref()
        .map(|i| i.duration_frames.get())
        .unwrap_or(0)
}
