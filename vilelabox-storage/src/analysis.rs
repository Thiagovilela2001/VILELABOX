//! Cache de resultados de análise por faixa, etapa, versão e configuração.

use crate::error::{mapear_constraint, ErroStorage};
use rusqlite::{params, Connection, OptionalExtension};
use vilelabox_types::TrackId;

/// Grava (ou substitui) o resultado de uma etapa de análise para a faixa.
/// A chave do cache é (faixa, etapa, versão do algoritmo, configuração):
/// versões ou configurações diferentes coexistem.
pub fn salvar(
    con: &Connection,
    faixa: TrackId,
    etapa: &str,
    versao: u32,
    config: &str,
    dados: &[u8],
) -> Result<(), ErroStorage> {
    con.execute(
        "INSERT INTO analysis_results (track_id, stage, algo_version, config, payload, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         ON CONFLICT(track_id, stage, algo_version, config) DO UPDATE SET
            payload = excluded.payload,
            updated_at = excluded.updated_at",
        params![faixa.0, etapa, versao, config, dados],
    )
    .map_err(mapear_constraint)?;
    Ok(())
}

/// Resultado em cache, se existir para exatamente esta versão e configuração.
pub fn obter(
    con: &Connection,
    faixa: TrackId,
    etapa: &str,
    versao: u32,
    config: &str,
) -> Result<Option<Vec<u8>>, ErroStorage> {
    Ok(con
        .query_row(
            "SELECT payload FROM analysis_results
             WHERE track_id = ?1 AND stage = ?2 AND algo_version = ?3 AND config = ?4",
            params![faixa.0, etapa, versao, config],
            |r| r.get(0),
        )
        .optional()?)
}
