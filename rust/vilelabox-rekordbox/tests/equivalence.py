"""Equivalência da exportação: o mesmo banco exportado por tools/rekordbox/sync.py
e pela CLI Rust deve gerar o mesmo XML (comparado por compare_with_python.py).

O banco cobre casos difíceis: caracteres especiais no caminho e nos nomes, cue
com posição fracionária, loop, memory cue, tipo ignorado, faixa apagada, BPM 0,
sample rate nulo, empate de arredondamento, playlist vazia, escondida e histórico.

Uso: python3 rust/vilelabox-rekordbox/tests/equivalence.py  (requer cargo)
"""
import os
import posixpath
import sqlite3
import subprocess
import sys
import tempfile
from pathlib import Path

repo = Path(__file__).resolve().parents[3]
tmp = Path(tempfile.mkdtemp())
(tmp / ".mixxx").mkdir()
db = tmp / ".mixxx" / "mixxxdb.sqlite"

conn = sqlite3.connect(db)
conn.executescript("""
CREATE TABLE track_locations (id INTEGER PRIMARY KEY, location TEXT, fs_deleted INTEGER DEFAULT 0);
CREATE TABLE library (id INTEGER PRIMARY KEY, artist TEXT, title TEXT, album TEXT, genre TEXT, bpm REAL,
  duration REAL, key TEXT, rating INTEGER, bitrate INTEGER, samplerate INTEGER, datetime_added TEXT,
  comment TEXT, location INTEGER);
CREATE TABLE cues (id INTEGER PRIMARY KEY, track_id INTEGER, type INTEGER, position INTEGER, length INTEGER,
  hotcue INTEGER, label TEXT, color INTEGER);
CREATE TABLE Playlists (id INTEGER PRIMARY KEY, name TEXT, position INTEGER DEFAULT 0, hidden INTEGER DEFAULT 0);
CREATE TABLE PlaylistTracks (id INTEGER PRIMARY KEY, playlist_id INTEGER, track_id INTEGER, position INTEGER);
INSERT INTO track_locations VALUES (1, '/music/AC&DC 100% (live)/faixa ç.mp3', 0),
  (2, '/music/sem título.flac', 0), (3, '/music/apagada.mp3', 1), (4, '/music/./x/../meio.wav', 0);
INSERT INTO library VALUES
  (1, 'AC/DC & "Amigos"', 'Rock <&> Roll', 'Álbum', 'Rock', 127.995, 180.5, '8A', 80, 320, 44100, NULL, 'linha1', 1),
  (2, NULL, NULL, NULL, NULL, 0, 181.5, NULL, NULL, 0, NULL, NULL, NULL, 2),
  (3, 'X', 'Apagada', '', '', 120, 100, '', 0, 320, 44100, NULL, '', 3),
  (4, 'Y', 'Meio', '', '', NULL, 99.4, '1B', 0, 256, 48000, NULL, '', 4);
INSERT INTO cues VALUES
  (1, 1, 1, 44100.5, 0, 0, 'MIX IN', 3004200),
  (2, 1, 4, 88200, 176400.25, 1, '', NULL),
  (3, 1, 1, 1000, 0, -1, '', -1),
  (4, 1, 2, 5000, 0, -1, 'ignorado (tipo 2)', 0),
  (5, 4, 1, -500, 0, 2, '', 16711680);
INSERT INTO Playlists VALUES (1, 'Set & Festa', 1, 0), (2, 'Vazia', 0, 0), (3, 'Escondida', 0, 1),
  (4, '2026-10-01', 0, 2), (5, 'histórico vazio', 0, 2);
INSERT INTO PlaylistTracks VALUES (1, 1, 4, 1), (2, 1, 1, 2), (3, 1, 3, 3), (4, 4, 1, 1);
""")
conn.commit()
conn.close()

# sync.py é para Linux: o caminho usa os.path.abspath POSIX.
sys.path.insert(0, str(repo / "tools" / "rekordbox"))
import sync  # noqa: E402

sync.path_to_file_url = lambda path: (
    f"file://localhost{sync.urllib.parse.quote(posixpath.abspath(path), safe='/')}")
py_xml = tmp / "python.xml"
sync.export_mixxx_to_rekordbox_xml(output_xml=str(py_xml), db_path=str(db))

rs_xml = tmp / "rust.xml"
subprocess.run(["cargo", "build", "-q", "-p", "vilelabox-rekordbox"], cwd=repo / "rust", check=True)
# HOME/USERPROFILE apontam a CLI para o banco de teste (mesmo padrão do sync.py).
env = dict(os.environ, HOME=str(tmp), USERPROFILE=str(tmp))
subprocess.run([str(repo / "rust/target/debug/vilelabox-rekordbox"), "export", str(rs_xml)], env=env, check=True)

result = subprocess.run([sys.executable, "-I", str(repo / "rust/vilelabox-rekordbox/tests/compare_with_python.py"),
                         str(py_xml), str(rs_xml)])
print("XMLs em", tmp)
sys.exit(result.returncode)
