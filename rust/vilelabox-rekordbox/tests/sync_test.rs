use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use vilelabox_rekordbox::{
    export_mixxx_to_rekordbox_xml, import_rekordbox_xml_to_mixxx, int_to_rgb, path_to_file_url,
    protected_import, rgb_to_int, SyncError,
};

fn setup_minimal_mixxx_db(db_path: &Path) {
    let conn = Connection::open(db_path).unwrap();
    conn.execute_batch(
        "CREATE TABLE track_locations (
            id INTEGER PRIMARY KEY,
            location TEXT,
            fs_deleted INTEGER DEFAULT 0
        );
        CREATE TABLE library (
            id INTEGER PRIMARY KEY,
            artist TEXT,
            title TEXT,
            album TEXT,
            genre TEXT,
            bpm REAL,
            duration REAL,
            key TEXT,
            rating INTEGER,
            bitrate INTEGER,
            samplerate INTEGER,
            comment TEXT,
            location INTEGER REFERENCES track_locations(id)
        );
        CREATE TABLE cues (
            id INTEGER PRIMARY KEY,
            track_id INTEGER,
            type INTEGER,
            position INTEGER,
            length INTEGER,
            hotcue INTEGER,
            label TEXT,
            color INTEGER
        );
        CREATE TABLE Playlists (
            id INTEGER PRIMARY KEY,
            name TEXT,
            position INTEGER DEFAULT 0,
            hidden INTEGER DEFAULT 0
        );
        CREATE TABLE PlaylistTracks (
            id INTEGER PRIMARY KEY,
            playlist_id INTEGER,
            track_id INTEGER,
            position INTEGER
        );",
    )
    .unwrap();
}

#[test]
fn test_color_and_url_conversions() {
    assert_eq!(int_to_rgb(None), (0, 122, 255));
    assert_eq!(int_to_rgb(Some(-1)), (0, 122, 255));
    let rgb = (255, 69, 69); // Drop red
    let color_int = rgb_to_int(rgb.0, rgb.1, rgb.2);
    assert_eq!(int_to_rgb(Some(color_int)), rgb);

    let url = path_to_file_url("/home/user/Music/track.mp3");
    assert_eq!(url, "file://localhost/home/user/Music/track.mp3");
}

// Os três testes abaixo portam tools/rekordbox/test_sync.py; o quarto
// (processo aberto) está em src/lib.rs, porque troca o verificador de processo.
fn db_with_value(dir: &Path) -> std::path::PathBuf {
    let db = dir.join("mixxxdb.sqlite");
    let conn = Connection::open(&db).unwrap();
    conn.execute_batch("CREATE TABLE test (value TEXT); INSERT INTO test VALUES ('original');")
        .unwrap();
    db
}

fn value(db: &Path) -> String {
    Connection::open(db)
        .unwrap()
        .query_row("SELECT value FROM test", [], |r| r.get(0))
        .unwrap()
}

fn backups(dir: &Path) -> Vec<std::path::PathBuf> {
    match fs::read_dir(dir.join("backups-rekordbox")) {
        Ok(entries) => entries.map(|e| e.unwrap().path()).collect(),
        Err(_) => Vec::new(),
    }
}

#[test]
fn test_backup_precedes_write_and_rollback() {
    let dir = tempdir().unwrap();
    let db = db_with_value(dir.path());

    let result = protected_import(&db, |tx| -> Result<(), SyncError> {
        tx.execute("UPDATE test SET value = 'changed'", [])?;
        Err(SyncError::Validation("invalid XML data".into()))
    });
    assert!(result.is_err());

    let saved = backups(dir.path());
    assert_eq!(saved.len(), 1);
    assert_eq!(value(&db), "original");
    assert_eq!(value(&saved[0]), "original");
}

#[test]
fn test_writer_lock_refuses_import_without_backup() {
    let dir = tempdir().unwrap();
    let db = db_with_value(dir.path());
    let writer = Connection::open(&db).unwrap();
    writer.execute_batch("BEGIN IMMEDIATE").unwrap();

    let result = protected_import(&db, |_| -> Result<(), SyncError> {
        panic!("must not write")
    });
    assert!(matches!(result, Err(SyncError::Sqlite(_))));
    assert!(!dir.path().join("backups-rekordbox").exists());
}

#[test]
fn test_wal_backup_includes_committed_data() {
    let dir = tempdir().unwrap();
    let db = db_with_value(dir.path());
    let writer = Connection::open(&db).unwrap();
    writer
        .query_row("PRAGMA journal_mode=WAL", [], |r| r.get::<_, String>(0))
        .unwrap();
    writer.execute("UPDATE test SET value = 'wal'", []).unwrap();

    protected_import(&db, |tx| {
        tx.execute("UPDATE test SET value = 'imported'", [])?;
        Ok(())
    })
    .unwrap();

    assert_eq!(value(&backups(dir.path())[0]), "wal");
    assert_eq!(value(&db), "imported");
}

#[test]
fn test_roundtrip_export_import() {
    let dir = tempdir().unwrap();
    let db1 = dir.path().join("db1.sqlite");
    let db2 = dir.path().join("db2.sqlite");
    let xml_path = dir.path().join("rekordbox.xml");

    setup_minimal_mixxx_db(&db1);
    setup_minimal_mixxx_db(&db2);

    let audio_path = dir.path().join("song.mp3");
    fs::write(&audio_path, b"audio_content").unwrap();
    // O Mixxx grava caminhos com "/" também no Windows (Qt).
    let audio_str = audio_path.to_str().unwrap().replace('\\', "/");

    // Insere dados no banco 1
    {
        let conn = Connection::open(&db1).unwrap();
        conn.execute(
            "INSERT INTO track_locations (id, location, fs_deleted) VALUES (1, ?, 0)",
            params![audio_str],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO library (id, artist, title, album, genre, bpm, duration, key, rating, bitrate, samplerate, comment, location)
             VALUES (1, 'Artist 1', 'Track 1', 'Album A', 'Techno', 128.0, 180.0, '1A', 5, 320, 44100, 'Test comment', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO cues (id, track_id, type, position, length, hotcue, label, color)
             VALUES (1, 1, 1, 44100.5, 0, 0, 'MIX IN', 3004200)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Playlists (id, name, position, hidden) VALUES (1, 'Favorites', 0, 0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO PlaylistTracks (id, playlist_id, track_id, position) VALUES (1, 1, 1, 1)",
            [],
        )
        .unwrap();
    }

    // Exporta DB1 para XML
    let (t_exp, p_exp) = export_mixxx_to_rekordbox_xml(&db1, &xml_path).unwrap();
    assert_eq!(t_exp, 1);
    assert_eq!(p_exp, 1);
    assert!(xml_path.is_file());

    // Prepara DB2 com a mesma faixa (para permitir sync de cues e playlists)
    {
        let conn = Connection::open(&db2).unwrap();
        conn.execute(
            "INSERT INTO track_locations (id, location, fs_deleted) VALUES (1, ?, 0)",
            params![audio_str],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO library (id, artist, title, bpm, duration, samplerate, location)
             VALUES (1, 'Artist 1', 'Track 1', 128.0, 180.0, 44100, 1)",
            [],
        )
        .unwrap();
    }

    // Importa XML no DB2
    let (cues_synced, pls_synced) = import_rekordbox_xml_to_mixxx(&xml_path, &db2).unwrap();
    assert_eq!(cues_synced, 1);
    assert_eq!(pls_synced, 1);

    // Verifica que DB2 agora possui o cue e a playlist sincronizados
    {
        let conn = Connection::open(&db2).unwrap();
        let cue_count: i64 = conn
            .query_row("SELECT count(*) FROM cues WHERE hotcue = 0", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(cue_count, 1);

        let pl_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM Playlists WHERE name = 'Favorites'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(pl_count, 1);
    }
}
