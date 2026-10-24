pub mod beats_proto;
pub mod db;
pub mod error;

pub use beats_proto::{beats_proto_to_grid, decode_beats, MixxxBeats};
pub use db::{MixxxDb, MixxxPlaylist, MixxxTrack};
pub use error::MixxxCompatError;

#[cfg(test)]
mod tests {
    use super::*;
    use vilelabox_types::{Frame, SampleRate};

    /// Codificadores mínimos de protobuf para montar blobs como o Mixxx grava.
    fn varint(mut v: u64, out: &mut Vec<u8>) {
        loop {
            let b = (v & 0x7F) as u8;
            v >>= 7;
            if v == 0 {
                out.push(b);
                return;
            }
            out.push(b | 0x80);
        }
    }

    fn beat_msg(frame: i32) -> Vec<u8> {
        let mut m = vec![0x08];
        varint(frame as i64 as u64, &mut m); // int32 negativo: 10 bytes
        m.extend_from_slice(&[0x10, 0x01]); // enabled = true (campo 2, bool)
        m
    }

    fn submsg(field: u8, msg: &[u8], out: &mut Vec<u8>) {
        out.push(field << 3 | 2);
        varint(msg.len() as u64, out);
        out.extend_from_slice(msg);
    }

    fn zigzag(v: i32) -> u64 {
        ((v << 1) ^ (v >> 31)) as u32 as u64
    }

    fn grid_blob(bpm: f64, first_beat: i32, downbeat: Option<i32>) -> Vec<u8> {
        let mut bpm_msg = vec![0x09];
        bpm_msg.extend_from_slice(&bpm.to_le_bytes());
        let mut out = Vec::new();
        submsg(1, &bpm_msg, &mut out);
        submsg(2, &beat_msg(first_beat), &mut out);
        if let Some(d) = downbeat {
            out.push(3 << 3);
            varint(zigzag(d), &mut out);
        }
        out
    }

    #[test]
    fn beat_grid_expands_constant_tempo_and_downbeat_phase() {
        // 120 BPM a 44,1 kHz = 22050 frames por batida; downbeat na batida 6.
        let blob = grid_blob(120.0, 11025, Some(6));
        assert_eq!(
            decode_beats(&blob, "BeatGrid-2.0").unwrap(),
            MixxxBeats::Grid {
                bpm: 120.0,
                first_beat: 11025,
                first_downbeat_index: Some(6),
            }
        );
        let grid =
            beats_proto_to_grid(&blob, "BeatGrid-2.0", SampleRate::CD, Frame::new(441000)).unwrap();
        assert_eq!(grid.beats().len(), 20);
        assert_eq!(grid.beats()[0].position, Frame::new(11025));
        assert_eq!(grid.beats()[19].position, Frame::new(11025 + 19 * 22050));
        assert_eq!(grid.primary_bpm().unwrap().get(), 120.0);
        // Fase 6 mod 4 = 2: terceira batida.
        assert_eq!(grid.downbeat(), Some(Frame::new(11025 + 2 * 22050)));

        let no_downbeat = grid_blob(120.0, 0, None);
        let grid = beats_proto_to_grid(&no_downbeat, "BeatGrid-2.0", SampleRate::CD, Frame::new(1))
            .unwrap();
        assert_eq!(grid.downbeat(), None);
        assert_eq!(grid.beats().len(), 1);
    }

    #[test]
    fn beat_map_keeps_negative_frames_and_tempo_changes() {
        let mut blob = Vec::new();
        for f in [-512, 21538, 43588, 62488, 81388] {
            submsg(1, &beat_msg(f), &mut blob);
        }
        blob.push(2 << 3);
        varint(zigzag(-3), &mut blob);

        let grid =
            beats_proto_to_grid(&blob, "BeatMap-1.0", SampleRate::CD, Frame::new(0)).unwrap();
        let positions: Vec<i64> = grid.beats().iter().map(|b| b.position.get()).collect();
        assert_eq!(positions, [-512, 21538, 43588, 62488, 81388]);
        // 22050 frames = 120 BPM; 18900 frames = 140 BPM.
        let bpms: Vec<f64> = grid.tempo_map().iter().map(|t| t.bpm.get()).collect();
        assert_eq!(bpms, [120.0, 140.0]);
        // -3 mod 4 = 1.
        assert_eq!(grid.downbeat(), Some(Frame::new(21538)));
    }

    #[test]
    fn legacy_grid_and_invalid_blobs() {
        let mut legacy = 128.0f64.to_ne_bytes().to_vec();
        legacy.extend_from_slice(&1000.0f64.to_ne_bytes());
        // 0x00 inicial = tag do campo 0: protobuf inválido, cai no formato antigo.
        assert_eq!(legacy[0], 0);
        assert_eq!(
            decode_beats(&legacy, "BeatGrid-1.0").unwrap(),
            MixxxBeats::Grid {
                bpm: 128.0,
                first_beat: 1000,
                first_downbeat_index: None,
            }
        );

        let truncated = &grid_blob(120.0, 0, None)[..5];
        assert!(decode_beats(truncated, "BeatGrid-2.0").is_err());
        let mut one_beat = Vec::new();
        submsg(1, &beat_msg(0), &mut one_beat);
        assert!(decode_beats(&one_beat, "BeatMap-1.0").is_err());
        assert!(decode_beats(&one_beat, "Outro-1.0").is_err());
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
                beats BLOB,
                beats_version TEXT
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
