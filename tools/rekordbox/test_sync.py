import contextlib
import importlib.util
import io
from pathlib import Path
import sqlite3
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('bridge', Path(__file__).with_name('sync.py'))
bridge = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bridge)


class ImportProtectionTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.db = Path(self.temp.name) / 'mixxxdb.sqlite'
        with contextlib.closing(sqlite3.connect(self.db)) as conn:
            conn.execute('CREATE TABLE test (value TEXT)')
            conn.execute("INSERT INTO test VALUES ('original')")
            conn.commit()
        self.stopped = patch.object(bridge, 'assert_mixxx_stopped')
        self.stopped.start()
        self.addCleanup(self.stopped.stop)

    def test_backup_precedes_write_and_rollback(self):
        with contextlib.redirect_stdout(io.StringIO()):
            with self.assertRaises(ValueError):
                with bridge.protected_import_connection(str(self.db)) as conn:
                    conn.execute("UPDATE test SET value='changed'")
                    raise ValueError('invalid XML data')
        backups = list(self.db.parent.glob('backups-rekordbox/*.bak'))
        self.assertEqual(len(backups), 1)
        for path in [self.db, backups[0]]:
            with contextlib.closing(sqlite3.connect(path)) as conn:
                self.assertEqual(conn.execute('SELECT value FROM test').fetchone()[0], 'original')

    def test_writer_lock_refuses_import_without_backup(self):
        with contextlib.closing(sqlite3.connect(self.db)) as writer:
            writer.execute('BEGIN IMMEDIATE')
            with self.assertRaises(sqlite3.OperationalError):
                with bridge.protected_import_connection(str(self.db)):
                    self.fail('must not write')
        self.assertFalse((self.db.parent / 'backups-rekordbox').exists())

    def test_wal_backup_includes_committed_data(self):
        with contextlib.closing(sqlite3.connect(self.db)) as writer:
            writer.execute('PRAGMA journal_mode=WAL')
            writer.execute("UPDATE test SET value='wal'")
            writer.commit()
            with contextlib.redirect_stdout(io.StringIO()):
                with bridge.protected_import_connection(str(self.db)) as conn:
                    conn.execute("UPDATE test SET value='imported'")
            backup = next(self.db.parent.glob('backups-rekordbox/*.bak'))
            with contextlib.closing(sqlite3.connect(backup)) as conn:
                self.assertEqual(conn.execute('SELECT value FROM test').fetchone()[0], 'wal')

    def test_running_process_refuses_import(self):
        bridge.assert_mixxx_stopped.side_effect = RuntimeError('running')
        with self.assertRaises(RuntimeError):
            with bridge.protected_import_connection(str(self.db)):
                self.fail('must not write')
        self.assertFalse((self.db.parent / 'backups-rekordbox').exists())

    def test_backup_retention_keeps_ten_latest(self):
        with contextlib.redirect_stdout(io.StringIO()):
            for _ in range(15):
                with bridge.protected_import_connection(str(self.db)) as conn:
                    conn.execute("UPDATE test SET value='v'")
        backups = list(self.db.parent.glob('backups-rekordbox/*.bak'))
        self.assertEqual(len(backups), 10)


class BridgeRulesTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.db = Path(self.temp.name) / 'mixxxdb.sqlite'
        with contextlib.closing(sqlite3.connect(self.db)) as conn:
            conn.executescript("""
                CREATE TABLE track_locations (id INTEGER PRIMARY KEY, location TEXT, fs_deleted INTEGER DEFAULT 0);
                CREATE TABLE library (id INTEGER PRIMARY KEY, artist TEXT, title TEXT, album TEXT, genre TEXT, bpm REAL,
                  duration REAL, key TEXT, rating INTEGER, bitrate INTEGER, samplerate INTEGER, datetime_added TEXT,
                  comment TEXT, location INTEGER, beats BLOB);
                CREATE TABLE cues (id INTEGER PRIMARY KEY, track_id INTEGER, type INTEGER, position INTEGER, length INTEGER,
                  hotcue INTEGER, label TEXT, color INTEGER);
                CREATE TABLE Playlists (id INTEGER PRIMARY KEY, name TEXT, position INTEGER DEFAULT 0, hidden INTEGER DEFAULT 0);
                CREATE TABLE PlaylistTracks (id INTEGER PRIMARY KEY, playlist_id INTEGER, track_id INTEGER, position INTEGER);
            """)
            conn.commit()
        self.stopped = patch.object(bridge, 'assert_mixxx_stopped')
        self.stopped.start()
        self.addCleanup(self.stopped.stop)

    def test_memory_cues_matching_by_position(self):
        # Inserir faixa e dois memory cues no Mixxx
        with contextlib.closing(sqlite3.connect(self.db)) as conn:
            conn.execute("INSERT INTO track_locations VALUES (1, '/music/track1.mp3', 0)")
            conn.execute("INSERT INTO library VALUES (1, 'Artist', 'Track 1', '', '', 120, 180, '', 0, 320, 44100, NULL, '', 1, NULL)")
            # Cue 1 a 10s (44100 * 20), Cue 2 a 20s (44100 * 40)
            conn.execute("INSERT INTO cues VALUES (1, 1, 1, 882000, 0, -1, 'Mem 1', 16711680)")
            conn.execute("INSERT INTO cues VALUES (2, 1, 1, 1764000, 0, -1, 'Mem 2', 65280)")
            conn.commit()

        # XML com os mesmos dois memory cues atualizando nomes e um novo a 30s
        xml_content = """<?xml version="1.0" encoding="UTF-8"?>
<DJ_PLAYLISTS Version="1.0.0">
  <PRODUCT Name="rekordbox" Version="6.8.5" Company="AlphaTheta"/>
  <COLLECTION Entries="1">
    <TRACK TrackID="1" Name="Track 1" Location="file://localhost/music/track1.mp3">
      <POSITION_MARK Name="Mem 1 Updated" Type="0" Start="10.000" Num="-1" Red="255" Green="0" Blue="0"/>
      <POSITION_MARK Name="Mem 2 Updated" Type="0" Start="20.000" Num="-1" Red="0" Green="255" Blue="0"/>
      <POSITION_MARK Name="Mem 3 New" Type="0" Start="30.000" Num="-1" Red="0" Green="0" Blue="255"/>
    </TRACK>
  </COLLECTION>
</DJ_PLAYLISTS>"""
        xml_path = Path(self.temp.name) / "rekordbox.xml"
        xml_path.write_text(xml_content, encoding="utf-8")

        with contextlib.redirect_stdout(io.StringIO()):
            bridge.import_rekordbox_xml_to_mixxx(str(xml_path), str(self.db))

        with contextlib.closing(sqlite3.connect(self.db)) as conn:
            cues = conn.execute("SELECT id, position, label, hotcue FROM cues WHERE track_id = 1 ORDER BY position").fetchall()
            self.assertEqual(len(cues), 3)
            self.assertEqual(cues[0][2], "Mem 1 Updated")
            self.assertEqual(cues[0][3], -1)
            self.assertEqual(cues[1][2], "Mem 2 Updated")
            self.assertEqual(cues[1][3], -1)
            self.assertEqual(cues[2][2], "Mem 3 New")
            self.assertEqual(cues[2][3], -1)

    def test_playlist_folder_hierarchy(self):
        with contextlib.closing(sqlite3.connect(self.db)) as conn:
            conn.execute("INSERT INTO track_locations VALUES (1, '/music/track1.mp3', 0)")
            conn.execute("INSERT INTO library VALUES (1, 'Artist', 'Track 1', '', '', 120, 180, '', 0, 320, 44100, NULL, '', 1, NULL)")
            conn.commit()

        xml_content = """<?xml version="1.0" encoding="UTF-8"?>
<DJ_PLAYLISTS Version="1.0.0">
  <PRODUCT Name="rekordbox" Version="6.8.5" Company="AlphaTheta"/>
  <COLLECTION Entries="1">
    <TRACK TrackID="1" Name="Track 1" Location="file://localhost/music/track1.mp3"/>
  </COLLECTION>
  <PLAYLISTS>
    <NODE Type="0" Name="ROOT">
      <NODE Type="0" Name="House">
        <NODE Type="1" Name="Set">
          <TRACK Key="1"/>
        </NODE>
      </NODE>
      <NODE Type="0" Name="Techno">
        <NODE Type="1" Name="Set">
          <TRACK Key="1"/>
        </NODE>
      </NODE>
    </NODE>
  </PLAYLISTS>
</DJ_PLAYLISTS>"""
        xml_path = Path(self.temp.name) / "folders.xml"
        xml_path.write_text(xml_content, encoding="utf-8")

        with contextlib.redirect_stdout(io.StringIO()):
            bridge.import_rekordbox_xml_to_mixxx(str(xml_path), str(self.db))

        with contextlib.closing(sqlite3.connect(self.db)) as conn:
            pls = sorted(conn.execute("SELECT name FROM Playlists").fetchall())
            self.assertEqual(pls, [("House/Set",), ("Techno/Set",)])

    def test_playlist_entries_counts_only_exported_tracks(self):
        with contextlib.closing(sqlite3.connect(self.db)) as conn:
            conn.execute("INSERT INTO track_locations VALUES (1, '/music/t1.mp3', 0), (2, '/music/deleted.mp3', 1)")
            conn.execute("INSERT INTO library VALUES (1, 'A', 'T1', '', '', 120, 180, '', 0, 320, 44100, NULL, '', 1, NULL)")
            conn.execute("INSERT INTO library VALUES (2, 'B', 'Deleted', '', '', 120, 180, '', 0, 320, 44100, NULL, '', 2, NULL)")
            conn.execute("INSERT INTO Playlists VALUES (1, 'My List', 0, 0)")
            conn.execute("INSERT INTO PlaylistTracks VALUES (1, 1, 1, 1), (2, 1, 2, 2)")
            conn.commit()

        out_xml = Path(self.temp.name) / "out.xml"
        bridge.path_to_file_url = lambda p: f"file://localhost{p}"
        with contextlib.redirect_stdout(io.StringIO()):
            bridge.export_mixxx_to_rekordbox_xml(str(out_xml), str(self.db))

        tree = bridge.ET.parse(out_xml)
        pl_node = tree.find(".//PLAYLISTS/NODE/NODE")
        self.assertIsNotNone(pl_node)
        self.assertEqual(pl_node.get("Entries"), "1")
        self.assertEqual(len(pl_node.findall("TRACK")), 1)


if __name__ == '__main__':
    unittest.main()

