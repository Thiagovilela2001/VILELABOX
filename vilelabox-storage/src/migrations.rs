//! Migrações versionadas por `PRAGMA user_version`.
//!
//! Cada migração roda na própria transação: em caso de falha o banco inteiro
//! permanece na versão anterior (rollback), e uma abertura posterior continua
//! de onde a sequência parou (recuperação de interrupção). Versões precisam
//! ser estritamente sequenciais; lacunas são rejeitadas.

use crate::error::ErroStorage;
use rusqlite::Connection;

/// Versão atual do esquema próprio.
pub const VERSAO_ESQUEMA: u32 = 1;

/// Migrações do banco próprio, em ordem estrita de versão.
pub const MIGRACOES: &[(u32, &str)] = &[(1, V1)];

/// Esquema v1 (P3.1): faixas, localizações, cues, grade, frases, playlists e
/// resultados de análise. Nomes próprios, sem a máquina de sincronização do
/// catálogo de referência (ADAPTAÇÃO, `spec/rekordbox/10-master-db.md`).
const V1: &str = "
CREATE TABLE tracks (
    id INTEGER PRIMARY KEY,
    sha256 TEXT,
    title TEXT,
    artist TEXT,
    album TEXT,
    genre TEXT,
    year INTEGER,
    comment TEXT,
    duration_frames INTEGER,
    sample_rate INTEGER,
    channels INTEGER,
    bpm REAL,
    key_text TEXT,
    rating INTEGER NOT NULL DEFAULT 0 CHECK (rating BETWEEN 0 AND 5),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE track_locations (
    id INTEGER PRIMARY KEY,
    track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    path TEXT NOT NULL UNIQUE,
    missing INTEGER NOT NULL DEFAULT 0,
    last_seen_at TEXT
);
CREATE INDEX idx_locations_track ON track_locations(track_id);

CREATE TABLE cues (
    id INTEGER PRIMARY KEY,
    track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    slot INTEGER,
    position_frames INTEGER NOT NULL,
    end_position_frames INTEGER,
    label TEXT NOT NULL DEFAULT '',
    color INTEGER,
    origin TEXT NOT NULL DEFAULT 'user'
);
CREATE INDEX idx_cues_track ON cues(track_id);

CREATE TABLE beatgrids (
    track_id INTEGER PRIMARY KEY REFERENCES tracks(id) ON DELETE CASCADE,
    sample_rate INTEGER NOT NULL,
    downbeat_frame INTEGER,
    locked INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE grid_beats (
    track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    idx INTEGER NOT NULL,
    position_frames INTEGER NOT NULL,
    beat_num INTEGER NOT NULL,
    PRIMARY KEY (track_id, idx)
);

CREATE TABLE grid_tempo (
    track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    idx INTEGER NOT NULL,
    position_frames INTEGER NOT NULL,
    bpm REAL NOT NULL,
    PRIMARY KEY (track_id, idx)
);

CREATE TABLE phrases (
    track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    idx INTEGER NOT NULL,
    phrase_id INTEGER NOT NULL,
    start_frames INTEGER NOT NULL,
    end_frames INTEGER NOT NULL,
    kind TEXT NOT NULL,
    beats INTEGER NOT NULL,
    PRIMARY KEY (track_id, idx)
);

CREATE TABLE playlists (
    id INTEGER PRIMARY KEY,
    parent_id INTEGER REFERENCES playlists(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    is_folder INTEGER NOT NULL DEFAULT 0,
    seq INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE playlist_tracks (
    playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    track_no INTEGER NOT NULL,
    PRIMARY KEY (playlist_id, track_no)
);

CREATE TABLE analysis_results (
    track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    stage TEXT NOT NULL,
    algo_version INTEGER NOT NULL,
    config TEXT NOT NULL DEFAULT '',
    payload BLOB NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (track_id, stage, algo_version, config)
);
";

/// Versão registrada no cabeçalho do banco (`0` em arquivo novo).
pub fn versao_atual(con: &Connection) -> Result<u32, ErroStorage> {
    Ok(con.pragma_query_value(None, "user_version", |r| r.get(0))?)
}

/// Aplica `migracoes` a partir da versão atual, uma transação por versão.
/// Retorna a versão final. Idempotente: versões já aplicadas são puladas.
pub fn migrar(con: &Connection, migracoes: &[(u32, &str)]) -> Result<u32, ErroStorage> {
    con.pragma_update(None, "foreign_keys", "ON")?;
    let mut atual = versao_atual(con)?;
    for &(versao, sql) in migracoes {
        if versao <= atual {
            continue;
        }
        if versao != atual + 1 {
            return Err(ErroStorage::Migracao(format!(
                "lacuna de versão: banco em {atual}, próxima migração {versao}"
            )));
        }
        let tx = con.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", versao)?;
        tx.commit()?;
        atual = versao;
    }
    Ok(atual)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn esquema_v1_tem_todas_as_tabelas() {
        let con = Connection::open_in_memory().unwrap();
        assert_eq!(migrar(&con, MIGRACOES).unwrap(), VERSAO_ESQUEMA);
        for tabela in [
            "tracks",
            "track_locations",
            "cues",
            "beatgrids",
            "grid_beats",
            "grid_tempo",
            "phrases",
            "playlists",
            "playlist_tracks",
            "analysis_results",
        ] {
            let n: i64 = con
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [tabela],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(n, 1, "tabela ausente: {tabela}");
        }
    }
}
