//! Árvore de playlists com ordem estável.

use crate::error::{mapear_constraint, ErroStorage};
use rusqlite::{params, Connection};
use vilelabox_types::TrackId;

/// Nó da árvore de playlists (pasta ou lista normal).
#[derive(Debug, Clone, PartialEq)]
pub struct ListaRow {
    pub id: i64,
    pub pai: Option<i64>,
    pub nome: String,
    pub pasta: bool,
    pub seq: i64,
}

/// Cria pasta (`pasta = true`) ou playlist como filha de `pai` (`None` =
/// raiz), no fim da sequência dos irmãos. Retorna o `id` criado.
pub fn criar(
    con: &Connection,
    nome: &str,
    pai: Option<i64>,
    pasta: bool,
) -> Result<i64, ErroStorage> {
    let seq: i64 = con.query_row(
        "SELECT COALESCE(MAX(seq), 0) + 1 FROM playlists
         WHERE (parent_id IS NULL AND ?1 IS NULL) OR parent_id = ?1",
        params![pai],
        |r| r.get(0),
    )?;
    con.execute(
        "INSERT INTO playlists (parent_id, name, is_folder, seq) VALUES (?1, ?2, ?3, ?4)",
        params![pai, nome, pasta, seq],
    )
    .map_err(mapear_constraint)?;
    Ok(con.last_insert_rowid())
}

/// Acrescenta a faixa no fim da playlist (`track_no` sequencial).
pub fn adicionar_faixa(con: &Connection, lista: i64, faixa: TrackId) -> Result<(), ErroStorage> {
    let pos: i64 = con.query_row(
        "SELECT COALESCE(MAX(track_no), 0) + 1 FROM playlist_tracks WHERE playlist_id = ?1",
        params![lista],
        |r| r.get(0),
    )?;
    con.execute(
        "INSERT INTO playlist_tracks (playlist_id, track_id, track_no) VALUES (?1, ?2, ?3)",
        params![lista, faixa.0, pos],
    )
    .map_err(mapear_constraint)?;
    Ok(())
}

/// Move a entrada da posição `de` para `para` (índices 0-based na ordem
/// atual) e renumera `track_no` de 1 em diante na mesma transação.
/// Retorna `false` quando algum índice está fora da lista.
pub fn mover_faixa(
    con: &Connection,
    lista: i64,
    de: usize,
    para: usize,
) -> Result<bool, ErroStorage> {
    let mut ids = faixas(con, lista)?;
    if de >= ids.len() || para >= ids.len() {
        return Ok(false);
    }
    let id = ids.remove(de);
    ids.insert(para, id);
    let tx = con.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM playlist_tracks WHERE playlist_id = ?1",
        params![lista],
    )?;
    for (i, f) in ids.iter().enumerate() {
        tx.execute(
            "INSERT INTO playlist_tracks (playlist_id, track_id, track_no) VALUES (?1, ?2, ?3)",
            params![lista, f.0, i as i64 + 1],
        )
        .map_err(mapear_constraint)?;
    }
    tx.commit()?;
    Ok(true)
}

/// Remove todas as faixas da playlist (a lista em si fica).
pub fn limpar(con: &Connection, lista: i64) -> Result<(), ErroStorage> {
    con.execute(
        "DELETE FROM playlist_tracks WHERE playlist_id = ?1",
        params![lista],
    )?;
    Ok(())
}

/// Faixas da playlist na ordem definida.
pub fn faixas(con: &Connection, lista: i64) -> Result<Vec<TrackId>, ErroStorage> {
    let mut st = con
        .prepare("SELECT track_id FROM playlist_tracks WHERE playlist_id = ?1 ORDER BY track_no")?;
    let ids = st
        .query_map(params![lista], |r| r.get::<_, i64>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ids.into_iter().map(TrackId).collect())
}

/// Todos os nós da árvore, pais antes de filhos na sequência gravada.
pub fn listar(con: &Connection) -> Result<Vec<ListaRow>, ErroStorage> {
    let mut st =
        con.prepare("SELECT id, parent_id, name, is_folder, seq FROM playlists ORDER BY seq, id")?;
    let rows = st.query_map([], |r| {
        Ok(ListaRow {
            id: r.get(0)?,
            pai: r.get(1)?,
            nome: r.get(2)?,
            pasta: r.get::<_, i64>(3)? != 0,
            seq: r.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}
