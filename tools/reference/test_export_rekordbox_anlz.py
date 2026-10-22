import struct
import tempfile
import unittest
from pathlib import Path

from export_rekordbox_anlz import (
    parse_anlz_sections,
    parse_ppth,
    parse_pqtz,
    parse_pssi,
    parse_cues,
    resolve_audio_file,
)


class TestExportRekordboxAnlz(unittest.TestCase):
    def test_parse_anlz_sections(self):
        # Cria um buffer ANLZ sintético com PMAI e uma seção fictícia PTEST
        payload = b"Hello, Pioneer!"
        sec_header = struct.pack(">4sII", b"TEST", 12, 12 + len(payload))
        data = b"PMAI" + struct.pack(">II", 16, 16 + len(sec_header) + len(payload)) + b"\x00\x00\x00\x00"
        data += sec_header + payload

        sections = parse_anlz_sections(data)
        self.assertIn("TEST", sections)
        self.assertEqual(sections["TEST"], payload)

    def test_parse_ppth(self):
        path_str = "/music/sample.mp3"
        encoded = path_str.encode("utf-16be") + b"\x00\x00"
        payload = struct.pack(">I", len(encoded)) + encoded
        res = parse_ppth(payload)
        self.assertEqual(res, path_str)

    def test_parse_pqtz(self):
        # 2 batidas a 120 bpm (tempo = 12000): beat 1 a 500ms, beat 2 a 1000ms
        header = struct.pack(">III", 0, 0x80000, 2)
        beat1 = struct.pack(">HHI", 1, 12000, 500)
        beat2 = struct.pack(">HHI", 2, 12000, 1000)
        payload = header + beat1 + beat2

        res = parse_pqtz(payload)
        self.assertIsNotNone(res)
        self.assertEqual(res["bpm"], 120.0)
        self.assertEqual(res["first_beat_time_s"], 0.5)
        self.assertEqual(res["downbeat_index"], 0)
        self.assertEqual(res["beats_count"], 2)

    def test_parse_pssi_unmasking(self):
        # Gera PSSI mascarado com 2 frases
        len_entry = 24
        len_entries = 2
        # Header do PSSI: len_entry (4 bytes), len_entries (2 bytes)
        # Seguido de body: mood=1 (high), 6 bytes pad, end_beat=128, 2 bytes pad, bank=1, 1 pad
        body_unmasked = bytearray()
        body_unmasked += struct.pack(">H6xH2xBB", 1, 128, 1, 0)
        # Entry 1: index=1, beat=1, kind=1 (intro)
        body_unmasked += struct.pack(">HHH18x", 1, 1, 1)
        # Entry 2: index=2, beat=33, kind=2 (up)
        body_unmasked += struct.pack(">HHH18x", 2, 33, 2)

        # Mascara o body com a chave XOR
        c = len_entries & 0xFF
        mask = bytes([(b + c) & 0xFF for b in [
            0xCB, 0xE1, 0xEE, 0xFA, 0xE5, 0xEE, 0xAD, 0xEE,
            0xE9, 0xD2, 0xE9, 0xEB, 0xE1, 0xE9, 0xF3, 0xE8,
            0xE9, 0xF4, 0xE1
        ]])
        body_masked = bytearray(body_unmasked)
        for i in range(len(body_masked)):
            body_masked[i] ^= mask[i % 19]

        payload = struct.pack(">IH", len_entry, len_entries) + bytes(body_masked)
        phrases = parse_pssi(payload, beat_times_ms=[500] + [500 + i * 500 for i in range(1, 128)])

        self.assertEqual(len(phrases), 2)
        self.assertEqual(phrases[0]["index"], 1)
        self.assertEqual(phrases[0]["beat"], 1)
        self.assertEqual(phrases[0]["kind"], "intro")
        self.assertEqual(phrases[0]["start_time_s"], 0.5)

        self.assertEqual(phrases[1]["index"], 2)
        self.assertEqual(phrases[1]["beat"], 33)
        self.assertEqual(phrases[1]["kind"], "up")


if __name__ == "__main__":
    unittest.main()
