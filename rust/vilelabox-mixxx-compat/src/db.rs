use crate::beats_proto::beats_proto_to_grid;
use crate::error::MixxxCompatError;
use rusqlite::{params, Connection};
use std::path::Path;
use vilelabox_types::{
    AudioIdentity, BeatGrid, Bpm, Cue, CueKind, CueOrigin, Frame, MusicalKey, SampleRate, Seconds,
    TrackId, TrackMetadata,
};

#[derive(Debug, Clone)]
pub struct MixxxTrack {
    pub id: TrackId,
    pub location: String,
    pub metadata: TrackMetadata,
    pub identity: AudioIdentity,
    pub bpm: Option<Bpm>,
    pub key: Option<MusicalKey>,
    pub duration: Seconds,
    pub beats_raw: Option<Vec<u8>>,
    /// `library.beats_version`: escolhe a mensagem de `beats_raw`.
    pub beats_version: Option<String>,
}

impl MixxxTrack {
    /// Grade decodificada; `None` quando a faixa não foi analisada.
    pub fn beat_grid(&self) -> Option<Result<BeatGrid, MixxxCompatError>> {
        let data = self.beats_raw.as_deref().filter(|d| !d.is_empty())?;
        let version = self.beats_version.as_deref().filter(|v| !v.is_empty())?;
        Some(beats_proto_to_grid(
            data,
            version,
            self.identity.sample_rate,
            self.identity.duration_frames,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct MixxxPlaylist {
    pub id: i64,
    pub name: String,
    pub track_ids: Vec<TrackId>,
}

pub struct MixxxDb {
    conn: Connection,
}

impl MixxxDb {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, MixxxCompatError> {
        let conn = Connection::open(path)?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self, MixxxCompatError> {
        let conn = Connection::open_in_memory()?;
        Ok(Self { conn })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Lê todas as faixas ativas (fs_deleted = 0) do banco do Mixxx.
    pub fn read_tracks(&self) -> Result<Vec<MixxxTrack>, MixxxCompatError> {
        let mut stmt = self.conn.prepare(
            "SELECT l.id, tl.location, l.artist, l.title, l.album, l.year, l.genre,
                    l.comment, l.bpm, l.key, l.duration, l.samplerate, l.channels, l.beats,
                    l.beats_version
             FROM library l
             JOIN track_locations tl ON l.location = tl.id
             WHERE tl.fs_deleted = 0",
        )?;

        let track_iter = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let location: String = row.get(1)?;
            let artist: Option<String> = row.get(2)?;
            let title: Option<String> = row.get(3)?;
            let album: Option<String> = row.get(4)?;
            let year_str: Option<String> = row.get(5)?;
            let genre: Option<String> = row.get(6)?;
            let comment: Option<String> = row.get(7)?;
            let raw_bpm: Option<f64> = row.get(8)?;
            let raw_key: Option<String> = row.get(9)?;
            let raw_duration: Option<f64> = row.get(10)?;
            let raw_samplerate: Option<u32> = row.get(11)?;
            let raw_channels: Option<u16> = row.get(12)?;
            let beats: Option<Vec<u8>> = row.get(13)?;
            let beats_version: Option<String> = row.get(14)?;

            let year = year_str.and_then(|y| y.parse::<u32>().ok());
            let rate_u32 = raw_samplerate.unwrap_or(44100).max(1);
            let rate = SampleRate::new(rate_u32).unwrap_or(SampleRate::CD);
            let duration_sec = raw_duration.unwrap_or(0.0).max(0.0);
            let duration = Seconds::new(duration_sec).unwrap_or(Seconds::ZERO);
            let duration_frames = duration.to_frame(rate);

            let bpm = raw_bpm.and_then(|b| Bpm::new(b).ok());
            let key = raw_key.and_then(|k| k.parse::<MusicalKey>().ok());

            let metadata = TrackMetadata {
                title,
                artist,
                album,
                genre,
                year,
                comment,
            };

            let identity = AudioIdentity {
                sha256: None,
                duration_frames,
                sample_rate: rate,
                channels: raw_channels.unwrap_or(2),
            };

            Ok(MixxxTrack {
                id: TrackId(id),
                location,
                metadata,
                identity,
                bpm,
                key,
                duration,
                beats_raw: beats,
                beats_version,
            })
        })?;

        let mut tracks = Vec::new();
        for t in track_iter {
            tracks.push(t?);
        }
        Ok(tracks)
    }

    /// Lê os cues de uma faixa específica.
    pub fn read_cues_for_track(&self, track_id: TrackId) -> Result<Vec<Cue>, MixxxCompatError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, type, position, length, hotcue, label, color
             FROM cues
             WHERE track_id = ?1
             ORDER BY position ASC",
        )?;

        let cue_iter = stmt.query_map(params![track_id.0], |row| {
            let id: i64 = row.get(0)?;
            let cue_type: i32 = row.get(1)?;
            let position_samples: i64 = row.get(2)?;
            let length_samples: i64 = row.get(3)?;
            let hotcue_slot: i32 = row.get(4)?;
            let label: Option<String> = row.get(5)?;
            let color: Option<u32> = row.get(6)?;

            let start_frame = Frame::new(position_samples / 2);
            let slot = if hotcue_slot >= 0 {
                Some(hotcue_slot as u8)
            } else {
                None
            };

            let cue = match cue_type {
                2 => {
                    // Loop
                    let end_frame = Frame::new((position_samples + length_samples) / 2);
                    if end_frame > start_frame {
                        Cue::new_loop(start_frame, end_frame, CueOrigin::User)
                            .unwrap()
                            .with_id(id)
                            .with_label(label.unwrap_or_default())
                    } else {
                        Cue::new(CueKind::CuePoint, start_frame, CueOrigin::User)
                            .with_id(id)
                            .with_label(label.unwrap_or_default())
                    }
                }
                _ => {
                    let kind = if slot.is_some() {
                        CueKind::HotCue
                    } else if hotcue_slot == -1 {
                        CueKind::MemoryCue
                    } else {
                        CueKind::CuePoint
                    };
                    let mut c = Cue::new(kind, start_frame, CueOrigin::User)
                        .with_id(id)
                        .with_label(label.unwrap_or_default());
                    if let Some(s) = slot {
                        c = c.with_slot(s);
                    }
                    if let Some(col) = color {
                        c = c.with_color(col);
                    }
                    c
                }
            };

            Ok(cue)
        })?;

        let mut cues = Vec::new();
        for c in cue_iter {
            cues.push(c?);
        }
        Ok(cues)
    }

    /// Lê todas as playlists e suas faixas.
    pub fn read_playlists(&self) -> Result<Vec<MixxxPlaylist>, MixxxCompatError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name
             FROM Playlists
             WHERE hidden = 0
             ORDER BY position ASC, id ASC",
        )?;

        let pl_iter = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            Ok((id, name))
        })?;

        let mut playlists = Vec::new();
        for pl in pl_iter {
            let (id, name) = pl?;
            let mut track_stmt = self.conn.prepare(
                "SELECT track_id
                 FROM PlaylistTracks
                 WHERE playlist_id = ?1
                 ORDER BY position ASC",
            )?;

            let t_iter = track_stmt.query_map(params![id], |r| {
                let tid: i64 = r.get(0)?;
                Ok(TrackId(tid))
            })?;

            let mut track_ids = Vec::new();
            for tid in t_iter {
                track_ids.push(tid?);
            }

            playlists.push(MixxxPlaylist {
                id,
                name,
                track_ids,
            });
        }
        Ok(playlists)
    }
}
