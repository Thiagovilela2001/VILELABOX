pub mod beats_proto;
pub mod db;
pub mod error;

pub use beats_proto::{beats_proto_to_grid, decode_beats_proto};
pub use db::{MixxxDb, MixxxPlaylist, MixxxTrack};
pub use error::MixxxCompatError;

#[cfg(test)]
mod tests {
    use super::*;
    use vilelabox_types::{Frame, SampleRate};

    #[test]
    fn test_beats_proto_decoding_and_grid_conversion() {
        // Blob simulado: submensagem tag 1 (wire 2), contendo field 1 (varint 0) e field 2 (float 120.0)
        // Sub-msg payload:
        // field 1 (pos 0): 0x08, 0x00
        // field 2 (bpm 120.0): 0x15, 0x00, 0x00, 0xF0, 0x42
        // total sub-len: 7 bytes
        // Outer: 0x0A, 0x07, [sub-bytes]
        let mut proto_data = vec![0x0A, 0x07, 0x08, 0x00, 0x15, 0x00, 0x00, 0xF0, 0x42];

        // Segunda batida: pos 22050 (0x5622 -> varint: 0xA2, 0xAC, 0x01)
        // Sub-msg: field 1: 0x08, 0xA2, 0xAC, 0x01
        // total sub-len: 4
        // Outer: 0x0A, 0x04, 0x08, 0xA2, 0xAC, 0x01
        proto_data.extend_from_slice(&[0x0A, 0x04, 0x08, 0xA2, 0xAC, 0x01]);

        let beats = decode_beats_proto(&proto_data).unwrap();
        assert_eq!(beats.len(), 2);
        assert_eq!(beats[0].0, 0);
        assert_eq!(beats[0].1, Some(120.0));
        assert_eq!(beats[1].0, 22050);

        let grid = beats_proto_to_grid(&proto_data, SampleRate::CD, None).unwrap();
        assert_eq!(grid.beats().len(), 2);
        assert_eq!(grid.beats()[0].position, Frame::new(0));
        assert_eq!(grid.beats()[1].position, Frame::new(22050));
        assert_eq!(grid.primary_bpm().unwrap().get(), 120.0);
    }

    #[test]
    fn test_mixxx_sqlite_in_memory_reading() {
        let db = MixxxDb::open_in_memory().unwrap();
        let conn = db.connection();

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
                year TEXT,
                genre TEXT,
                comment TEXT,
                bpm REAL,
                key TEXT,
                duration REAL,
                samplerate INTEGER,
                channels INTEGER,
                location INTEGER,
                beats BLOB
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
                position INTEGER,
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

        // Inserir dados
        conn.execute(
            "INSERT INTO track_locations (id, location, fs_deleted) VALUES (1, 'C:/Music/test.mp3', 0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO library (id, artist, title, album, year, genre, comment, bpm, key, duration, samplerate, channels, location)
             VALUES (1, 'Artist', 'Title', 'Album', '2024', 'House', 'Cool', 126.0, '8A', 240.0, 44100, 2, 1)",
            [],
        )
        .unwrap();

        // Inserir HotCue (slot 0) e MemoryCue (-1)
        conn.execute(
            "INSERT INTO cues (id, track_id, type, position, length, hotcue, label, color)
             VALUES (1, 1, 1, 88200, 0, 0, 'Hotcue 1', 16711680)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO cues (id, track_id, type, position, length, hotcue, label, color)
             VALUES (2, 1, 1, 176400, 0, -1, 'Memory 1', 65280)",
            [],
        )
        .unwrap();

        // Playlist
        conn.execute(
            "INSERT INTO Playlists (id, name, position, hidden) VALUES (1, 'Warmup/Deep', 1, 0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO PlaylistTracks (playlist_id, track_id, position) VALUES (1, 1, 1)",
            [],
        )
        .unwrap();

        // Validar leitura
        let tracks = db.read_tracks().unwrap();
        assert_eq!(tracks.len(), 1);
        let t = &tracks[0];
        assert_eq!(t.metadata.artist.as_deref(), Some("Artist"));
        assert_eq!(t.bpm.unwrap().get(), 126.0);
        assert_eq!(t.key.unwrap().to_camelot(), "8A");

        let cues = db.read_cues_for_track(t.id).unwrap();
        assert_eq!(cues.len(), 2);
        assert!(cues[0].is_hot_cue());
        assert_eq!(cues[0].slot, Some(0));
        assert_eq!(cues[0].position, Frame::new(44100)); // 88200 / 2
        assert!(cues[1].is_memory_cue());
        assert_eq!(cues[1].position, Frame::new(88200)); // 176400 / 2

        let playlists = db.read_playlists().unwrap();
        assert_eq!(playlists.len(), 1);
        assert_eq!(playlists[0].name, "Warmup/Deep");
        assert_eq!(playlists[0].track_ids, vec![t.id]);
    }
}
