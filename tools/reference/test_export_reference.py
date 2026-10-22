#!/usr/bin/env python3
"""Testes unitários para tools/reference/export_reference.py."""

import contextlib
import hashlib
import json
from pathlib import Path
import sqlite3
import struct
import tempfile
import unittest

import export_reference


def encode_varint(val: int) -> bytes:
    val &= (1 << 64) - 1  # int32 negativo vira varint de 64 bits, como no protobuf
    res = bytearray()
    while val > 0x7F:
        res.append((val & 0x7F) | 0x80)
        val >>= 7
    res.append(val & 0x7F)
    return bytes(res)


def zigzag_encode(n: int) -> int:
    return (n << 1) ^ (n >> 31)


def make_test_beatgrid_blob(bpm: float, first_beat_frame: int, downbeat_index: int) -> bytes:
    """Gera blob compatível com BeatGrid em src/proto/beats.proto."""
    # Bpm message (field 1 = bpm double, wire 1)
    bpm_msg = bytes([(1 << 3) | 1]) + struct.pack("<d", bpm)
    # Beat message (field 1 = frame_position varint, wire 0)
    beat_msg = bytes([(1 << 3) | 0]) + encode_varint(first_beat_frame)

    # BeatGrid: field 1 = Bpm (wire 2), field 2 = Beat (wire 2), field 3 = sint32 (wire 0)
    blob = (
        bytes([(1 << 3) | 2]) + encode_varint(len(bpm_msg)) + bpm_msg +
        bytes([(2 << 3) | 2]) + encode_varint(len(beat_msg)) + beat_msg +
        bytes([(3 << 3) | 0]) + encode_varint(zigzag_encode(downbeat_index))
    )
    return blob


def make_test_beatmap_blob(frames: list[int], downbeat_index: int) -> bytes:
    """Gera blob compatível com BeatMap: field 1 = Beat repetido, field 2 = sint32."""
    blob = b""
    for frame in frames:
        beat_msg = bytes([(1 << 3) | 0]) + encode_varint(frame)
        blob += bytes([(1 << 3) | 2]) + encode_varint(len(beat_msg)) + beat_msg
    return blob + bytes([(2 << 3) | 0]) + encode_varint(zigzag_encode(downbeat_index))


class TestExportReference(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp_dir.cleanup)
        self.profile_dir = Path(self.temp_dir.name) / "profile"
        self.profile_dir.mkdir(parents=True, exist_ok=True)
        self.output_dir = Path(self.temp_dir.name) / "output"

        # Cria áudio dummy para SHA-256
        self.audio_file = Path(self.temp_dir.name) / "test_track.wav"
        self.audio_file.write_bytes(b"RIFFdummydataforhashingWAVE")
        self.expected_hash = hashlib.sha256(b"RIFFdummydataforhashingWAVE").hexdigest()

        # Cria banco SQLite mínimo com esquema do Mixxx
        self.db_path = self.profile_dir / "mixxxdb.sqlite"
        with contextlib.closing(sqlite3.connect(str(self.db_path))) as conn:
            conn.execute("""
                CREATE TABLE track_locations (
                    id INTEGER PRIMARY KEY,
                    location TEXT
                )
            """)
            conn.execute("""
                CREATE TABLE library (
                    id INTEGER PRIMARY KEY,
                    artist TEXT,
                    title TEXT,
                    bpm REAL,
                    duration REAL,
                    key TEXT,
                    beats BLOB,
                    beats_version TEXT,
                    location INTEGER REFERENCES track_locations(id)
                )
            """)
            conn.execute("""
                CREATE TABLE cues (
                    id INTEGER PRIMARY KEY,
                    track_id INTEGER,
                    type INTEGER,
                    position INTEGER,
                    length INTEGER,
                    hotcue INTEGER,
                    label TEXT,
                    color INTEGER
                )
            """)

            # Insere localização e faixa
            conn.execute("INSERT INTO track_locations (id, location) VALUES (1, ?)", (str(self.audio_file),))
            beats_blob = make_test_beatgrid_blob(bpm=124.0, first_beat_frame=11025, downbeat_index=0)
            conn.execute("""
                INSERT INTO library (id, artist, title, bpm, duration, key, beats, beats_version, location)
                VALUES (1, 'Vilelabox Artist', 'Reference Track', 124.0, 180.0, '8A', ?, 'BeatGrid-2.0', 1)
            """, (beats_blob,))

            # Insere cues
            conn.execute("""
                INSERT INTO cues (id, track_id, type, position, length, hotcue, label, color)
                VALUES (1, 1, 1, 11025, 0, 0, 'MIX IN', 3004200)
            """)
            conn.commit()

        # Cria diretórios de cache de frases e vocais
        phrases_dir = self.profile_dir / "analysis" / "phrases"
        phrases_dir.mkdir(parents=True, exist_ok=True)
        phrase_data = {
            "version": "1.0",
            "location": str(self.audio_file),
            "phrases": [
                [0.25, 15.75, 8, 1, 0.95],
                [15.75, 31.25, 8, 3, 0.90],
            ]
        }
        (phrases_dir / "track1.phrases.json").write_text(json.dumps(phrase_data), encoding="utf-8")

        # Cache de vocal no formato real: sem location, nome = SHA-256 da
        # identidade de src/analyzer/analyzervocals.cpp (initialize).
        self.resources_dir = Path(self.temp_dir.name) / "res"
        (self.resources_dir / "vocals").mkdir(parents=True)
        (self.resources_dir / "vocals" / "model.json").write_bytes(b"{}")
        (self.resources_dir / "vocals" / "vocal_detector.py").write_bytes(b"# detector")
        stat = self.audio_file.stat()
        identity = (str(self.audio_file).encode() + str(stat.st_size).encode()
                    + str(stat.st_mtime_ns // 1_000_000).encode() + b"{}" + b"# detector" + b"classifier")
        vocals_dir = self.profile_dir / "analysis" / "vocals"
        vocals_dir.mkdir(parents=True, exist_ok=True)
        vocal_data = {"version": "1", "duration": 180.0, "segments": [[15.75, 30.00]], "engine": "classifier"}
        (vocals_dir / f"{hashlib.sha256(identity).hexdigest()}.json").write_text(
            json.dumps(vocal_data), encoding="utf-8")

    def export(self):
        return export_reference.export_reference(self.profile_dir, self.output_dir, self.resources_dir)

    def test_export_reference_success(self):
        index = self.export()
        self.assertEqual(index["count"], 1)
        self.assertEqual(index["tracks"][0]["sha256"], self.expected_hash)
        self.assertNotIn("artist", index["tracks"][0])
        self.assertEqual(index["tracks"][0]["cues_count"], 1)
        self.assertEqual(index["tracks"][0]["phrases_count"], 2)
        self.assertEqual(index["tracks"][0]["vocal_segments_count"], 1)

        # Confere arquivo exportado
        track_json_file = self.output_dir / f"{self.expected_hash}.json"
        self.assertTrue(track_json_file.is_file())

        with track_json_file.open("r", encoding="utf-8") as f:
            data = json.load(f)

        self.assertEqual(data["sha256"], self.expected_hash)
        self.assertAlmostEqual(data["bpm"], 124.0, places=2)
        self.assertEqual(data["first_beat_frame"], 11025)
        self.assertEqual(data["downbeat_index"], 0)
        self.assertEqual(data["key"], "8A")
        self.assertEqual(len(data["cues"]), 1)
        self.assertEqual(data["cues"][0]["label"], "MIX IN")
        self.assertEqual(len(data["phrases"]), 2)
        self.assertEqual(data["vocal_segments"], [[15.75, 30.00]])

    def test_protobuf_parser_standalone(self):
        blob = make_test_beatgrid_blob(bpm=128.5, first_beat_frame=22050, downbeat_index=2)
        parsed = export_reference.parse_beats_blob(blob)
        self.assertAlmostEqual(parsed["bpm"], 128.5, places=2)
        self.assertEqual(parsed["first_beat_frame"], 22050)
        self.assertEqual(parsed["first_downbeat_index"], 2)


    def test_beatmap_uses_first_beat_and_negative_frames(self):
        parsed = export_reference.parse_beats_blob(make_test_beatmap_blob([-512, 22050, 44100], 1))
        self.assertIsNone(parsed["bpm"])
        self.assertEqual(parsed["beats"], [-512, 22050, 44100])
        self.assertEqual(parsed["first_beat_frame"], -512)
        self.assertEqual(parsed["first_downbeat_index"], 1)

    def test_missing_audio_is_skipped_and_counted(self):
        self.audio_file.unlink()
        index = self.export()
        self.assertEqual(index["count"], 0)
        self.assertEqual(index["missing_audio"], 1)


if __name__ == "__main__":
    unittest.main()
