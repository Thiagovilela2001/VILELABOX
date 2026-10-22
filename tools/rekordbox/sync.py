#!/usr/bin/env python3
"""
Mixxx <-> Rekordbox Realtime XML Synchronization Bridge
=======================================================
Permite compartilhar faixas, Hot Cues, Loops, Beatgrid e Playlists
entre o Mixxx e o Rekordbox sem quebrar a integridade dos bancos de dados.

Uso:
  python3 sync.py export [caminho/rekordbox.xml]
  python3 sync.py import <caminho/rekordbox.xml>
  python3 sync.py status [caminho/rekordbox.xml]
"""

import sys
import os
import sqlite3
import struct
import contextlib
import urllib.parse
import xml.etree.ElementTree as ET
from xml.dom import minidom
from datetime import datetime
from pathlib import Path

def decode_varint(data, offset):
    res = 0
    shift = 0
    while offset < len(data):
        b = data[offset]
        offset += 1
        res |= (b & 0x7F) << shift
        if not (b & 0x80):
            return res, offset
        shift += 7
    return res, offset

def zigzag_decode(n):
    return (n >> 1) ^ (-(n & 1))

def parse_beats_blob(blob):
    if not blob:
        return None
    offset = 0
    bpm_val = None
    first_beat_frame = None
    first_downbeat_index = 0
    beats = []
    try:
        while offset < len(blob):
            tag, offset = decode_varint(blob, offset)
            field_num = tag >> 3
            wire_type = tag & 0x07
            if wire_type == 2:
                length, offset = decode_varint(blob, offset)
                field_data = blob[offset:offset+length]
                offset += length
                if field_num == 1:
                    if len(field_data) >= 9 and field_data[0] == 9:
                        bpm_val = struct.unpack("<d", field_data[1:9])[0]
                    elif len(field_data) >= 2 and field_data[0] == 8:
                        b_frame, _ = decode_varint(field_data, 1)
                        beats.append(b_frame)
                elif field_num == 2:
                    if len(field_data) >= 2 and field_data[0] == 8:
                        first_beat_frame, _ = decode_varint(field_data, 1)
            elif wire_type == 0:
                val, offset = decode_varint(blob, offset)
                if field_num in (2, 3):
                    first_downbeat_index = zigzag_decode(val)
            elif wire_type == 1:
                offset += 8
            elif wire_type == 5:
                offset += 4
            else:
                break
    except Exception:
        pass
    return {
        "bpm": bpm_val,
        "first_beat_frame": first_beat_frame,
        "first_downbeat_index": first_downbeat_index,
        "beats": beats,
    }

def extract_tempos(beats_info, fallback_bpm, samplerate):
    """Retorna lista de (inizio_sec, bpm, battito)."""
    if not beats_info:
        return [(0.0, fallback_bpm, 1)]

    beats = beats_info.get("beats") or []
    if len(beats) >= 2:
        downbeat_idx = beats_info.get("first_downbeat_index", 0) or 0
        tempos = []
        cur_bpm = None
        for i in range(len(beats) - 1):
            delta = (beats[i + 1] - beats[i]) / samplerate
            if delta <= 0:
                continue
            b_bpm = round(60.0 / delta, 2)
            if cur_bpm is None or abs(b_bpm - cur_bpm) >= 0.05:
                cur_bpm = b_bpm
                inizio_sec = max(0.0, beats[i] / samplerate)
                battito = 1 + (((i - downbeat_idx) % 4 + 4) % 4)
                tempos.append((inizio_sec, cur_bpm, battito))
        if tempos:
            return tempos

    first_frame = beats_info.get("first_beat_frame")
    bpm = beats_info.get("bpm") or fallback_bpm
    downbeat_idx = beats_info.get("first_downbeat_index", 0) or 0
    battito = 1 + (((-downbeat_idx) % 4 + 4) % 4)
    inizio_sec = max(0.0, (first_frame / samplerate)) if first_frame is not None else 0.0
    return [(inizio_sec, bpm, battito)]

DEFAULT_MIXXX_DB = os.path.expanduser("~/.mixxx/mixxxdb.sqlite")
DEFAULT_XML_PATH = os.path.expanduser("~/mixxx-rekordbox/rekordbox.xml")

def get_mixxx_connection(db_path=DEFAULT_MIXXX_DB):
    if not os.path.exists(db_path):
        raise FileNotFoundError(f"Banco do Mixxx não encontrado em: {db_path}")
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    return conn

def assert_mixxx_stopped():
    """Recusa importações enquanto um processo Mixxx local estiver aberto."""
    for entry in Path("/proc").iterdir():
        if not entry.name.isdigit():
            continue
        try:
            name = (entry / "comm").read_text().strip().lower()
        except (OSError, RuntimeError):
            continue
        try:
            executable = (entry / "exe").resolve().name.lower()
        except (OSError, RuntimeError):
            executable = ""
        if name in {"mixxx", "blackmesa", "vilelabox"} or executable in {
                "mixxx", "blackmesa", "vilelabox"}:
            raise RuntimeError("Feche o Mixxx/VILELABOX antes de importar no banco.")


@contextlib.contextmanager
def protected_import_connection(db_path):
    """Backup consistente e transação de escrita antes de qualquer escrita."""
    assert_mixxx_stopped()
    if not Path(db_path).is_file():
        raise FileNotFoundError(f"Banco do Mixxx não encontrado em: {db_path}")
    conn = sqlite3.connect(db_path, timeout=0)
    conn.row_factory = sqlite3.Row
    try:
        conn.execute("BEGIN IMMEDIATE")
        assert_mixxx_stopped()
        backup_dir = Path(db_path).parent / "backups-rekordbox"
        backup_dir.mkdir(exist_ok=True)
        stamp = datetime.now().strftime("%Y%m%d-%H%M%S-%f")
        backup_path = backup_dir / f"{Path(db_path).name}-{stamp}.bak"
        # Backup por outra conexão inclui o WAL sem bloquear a própria API.
        with contextlib.closing(sqlite3.connect(
                Path(db_path).resolve().as_uri() + "?mode=ro", uri=True,
                timeout=0)) as source:
            with contextlib.closing(sqlite3.connect(backup_path)) as backup:
                source.backup(backup)
        # IMMEDIATE bloqueia escritores e permite a conexão de backup em
        # bancos rollback e WAL. A checagem de processo exclui o aplicativo.
        print(f"[*] Backup criado: {backup_path}")
        # Manter os 10 últimos backups
        prefix = f"{Path(db_path).name}-"
        existing_backups = sorted(
            [f for f in backup_dir.glob(f"{prefix}*.bak") if f.is_file()],
            key=lambda p: p.stat().st_mtime,
            reverse=True,
        )
        for old_backup in existing_backups[10:]:
            try:
                old_backup.unlink()
            except OSError:
                pass
        yield conn
        conn.commit()
    except BaseException:
        conn.rollback()
        raise
    finally:
        conn.close()


def int_to_rgb(color_int):
    """Converte inteiro de cor do Mixxx (0xRRGGBB) para tupla (R, G, B)"""
    if color_int is None or color_int < 0:
        return 0, 122, 255 # Pioneer blue default
    r = (color_int >> 16) & 0xFF
    g = (color_int >> 8) & 0xFF
    b = color_int & 0xFF
    return r, g, b

def rgb_to_int(r, g, b):
    """Converte tupla (R, G, B) para inteiro 0xRRGGBB do Mixxx"""
    return (int(r) << 16) | (int(g) << 8) | int(b)

def path_to_file_url(path):
    """Converte caminho do Linux para URL file://localhost/... padrão Pioneer"""
    abs_path = os.path.abspath(path)
    encoded = urllib.parse.quote(abs_path, safe="/")
    return f"file://localhost{encoded}"

def file_url_to_path(url):
    """Converte URL file://localhost/... para caminho absoluto no filesystem"""
    if url.startswith("file://localhost"):
        url = url[len("file://localhost"):]
    elif url.startswith("file://"):
        url = url[len("file://"):]
    return urllib.parse.unquote(url)

# ==============================================================================
# EXPORTAÇÃO: MIXXX -> REKORDBOX XML
# ==============================================================================
def export_mixxx_to_rekordbox_xml(output_xml=DEFAULT_XML_PATH, db_path=DEFAULT_MIXXX_DB):
    print(f"[*] Conectando ao banco do Mixxx: {db_path}")
    conn = get_mixxx_connection(db_path)
    cursor = conn.cursor()

    # Raiz do documento XML Rekordbox
    root = ET.Element("DJ_PLAYLISTS", Version="1.0.0")
    ET.SubElement(root, "PRODUCT", Name="rekordbox", Version="6.8.5", Company="AlphaTheta")

    # 1. Carregar faixas ativas
    cursor.execute("PRAGMA table_info(library)")
    lib_cols = {row["name"] for row in cursor.fetchall()}
    has_beats = "beats" in lib_cols
    beats_col = ", l.beats" if has_beats else ""

    query_tracks = f"""
        SELECT l.id, l.artist, l.title, l.album, l.genre, l.bpm, l.duration,
               l.key, l.rating, l.bitrate, l.samplerate, l.datetime_added,
               l.comment, tl.location{beats_col}
        FROM library l
        JOIN track_locations tl ON l.location = tl.id
        WHERE tl.fs_deleted = 0
        ORDER BY l.id ASC
    """
    cursor.execute(query_tracks)
    tracks = cursor.fetchall()

    collection_el = ET.SubElement(root, "COLLECTION", Entries=str(len(tracks)))

    track_id_map = {} # mixxx_id -> rekordbox_track_id

    for idx, t in enumerate(tracks, start=1):
        track_id_map[t["id"]] = str(idx)
        loc_url = path_to_file_url(t["location"])
        duration_sec = int(round(t["duration"] or 0))
        bpm_val = f"{t['bpm']:.2f}" if t["bpm"] else "120.00"
        samplerate_val = t["samplerate"] or 44100

        track_el = ET.SubElement(
            collection_el, "TRACK",
            TrackID=str(idx),
            Name=t["title"] or Path(t["location"]).stem,
            Artist=t["artist"] or "",
            Album=t["album"] or "",
            Genre=t["genre"] or "",
            TotalTime=str(duration_sec),
            AverageBpm=bpm_val,
            Key=t["key"] or "",
            Rating=str(int(t["rating"] or 0)),
            BitRate=str(t["bitrate"] or 320),
            SampleRate=str(samplerate_val),
            Comments=t["comment"] or "",
            Location=loc_url
        )

        # Adicionar Beatgrid / TEMPO
        beats_blob = t["beats"] if has_beats else None
        beats_info = parse_beats_blob(beats_blob)
        fallback_bpm = float(t["bpm"]) if t["bpm"] else 120.0
        tempos = extract_tempos(beats_info, fallback_bpm, samplerate_val)
        for (inizio_sec, tempo_bpm, battito) in tempos:
            ET.SubElement(
                track_el, "TEMPO",
                Inizio=f"{inizio_sec:.3f}",
                Bpm=f"{tempo_bpm:.2f}",
                Metro="4/4",
                Battito=str(battito)
            )

        # 2. Carregar Cues e Loops da faixa
        cursor.execute("""
            SELECT type, position, length, hotcue, label, color
            FROM cues
            WHERE track_id = ? AND (type = 1 OR type = 4)
            ORDER BY hotcue ASC, position ASC
        """, (t["id"],))
        cues = cursor.fetchall()

        for c in cues:
            cue_type = c["type"] # 1 = HotCue, 4 = Loop
            hotcue_num = c["hotcue"] # 0 = A, 1 = B, etc. (-1 para Memory Cue)
            sample_pos = c["position"]
            length_samples = c["length"] or 0
            label = c["label"] or ""
            r, g, b = int_to_rgb(c["color"])

            # No Mixxx: seconds = sample_pos / (samplerate * 2)
            time_start_sec = max(0.0, sample_pos / (samplerate_val * 2.0))
            time_start_str = f"{time_start_sec:.3f}"

            if cue_type == 4: # Loop
                time_end_sec = max(time_start_sec, (sample_pos + length_samples) / (samplerate_val * 2.0))
                time_end_str = f"{time_end_sec:.3f}"
                ET.SubElement(
                    track_el, "POSITION_MARK",
                    Name=label or (f"Hot Loop {hotcue_num+1}" if hotcue_num >= 0 else "Loop"),
                    Type="1",
                    Start=time_start_str,
                    End=time_end_str,
                    Num=str(hotcue_num),
                    Red=str(r), Green=str(g), Blue=str(b)
                )
            else: # Hot Cue
                ET.SubElement(
                    track_el, "POSITION_MARK",
                    Name=label or (f"Hot Cue {hotcue_num+1}" if hotcue_num >= 0 else "Cue"),
                    Type="0",
                    Start=time_start_str,
                    Num=str(hotcue_num),
                    Red=str(r), Green=str(g), Blue=str(b)
                )

    # 3. Carregar Playlists do Mixxx
    playlists_el = ET.SubElement(root, "PLAYLISTS")
    root_node = ET.SubElement(playlists_el, "NODE", Type="0", Name="ROOT")

    # Playlists de usuário normais (hidden = 0)
    cursor.execute("""
        SELECT id, name FROM Playlists
        WHERE hidden = 0
        ORDER BY position ASC, id ASC
    """)
    playlists = cursor.fetchall()

    for pl in playlists:
        cursor.execute("""
            SELECT track_id FROM PlaylistTracks
            WHERE playlist_id = ?
            ORDER BY position ASC
        """, (pl["id"],))
        pl_tracks = cursor.fetchall()
        valid_tracks = [pt["track_id"] for pt in pl_tracks if pt["track_id"] in track_id_map]

        pl_node = ET.SubElement(
            root_node, "NODE",
            Name=pl["name"],
            Type="1",
            KeyType="0",
            Entries=str(len(valid_tracks))
        )

        for tid in valid_tracks:
            ET.SubElement(pl_node, "TRACK", Key=track_id_map[tid])

    # Sessões de Histórico do Mixxx (hidden = 2)
    cursor.execute("""
        SELECT id, name FROM Playlists
        WHERE hidden = 2
        ORDER BY id DESC
        LIMIT 20
    """)
    history_pls = cursor.fetchall()
    if history_pls:
        hist_folder = ET.SubElement(root_node, "NODE", Name="Mixxx History", Type="0")
        for pl in history_pls:
            cursor.execute("""
                SELECT track_id FROM PlaylistTracks
                WHERE playlist_id = ?
                ORDER BY position ASC
            """, (pl["id"],))
            pl_tracks = cursor.fetchall()
            valid_tracks = [pt["track_id"] for pt in pl_tracks if pt["track_id"] in track_id_map]
            if valid_tracks:
                pl_node = ET.SubElement(
                    hist_folder, "NODE",
                    Name=pl["name"],
                    Type="1",
                    KeyType="0",
                    Entries=str(len(valid_tracks))
                )
                for tid in valid_tracks:
                    ET.SubElement(pl_node, "TRACK", Key=track_id_map[tid])

    conn.close()

    # Formatar e Gravar XML
    xml_str = minidom.parseString(ET.tostring(root, encoding="utf-8")).toprettyxml(indent="  ", encoding="utf-8")
    os.makedirs(os.path.dirname(os.path.abspath(output_xml)), exist_ok=True)
    with open(output_xml, "wb") as f:
        f.write(xml_str)

    print(f"[+] Sucesso! Exportadas {len(tracks)} faixas e {len(playlists)} playlists para:")
    print(f"    {os.path.abspath(output_xml)}")

# ==============================================================================
# IMPORTAÇÃO: REKORDBOX XML -> MIXXX
# ==============================================================================
def import_rekordbox_xml_to_mixxx(xml_path, db_path=DEFAULT_MIXXX_DB):
    if not os.path.exists(xml_path):
        raise FileNotFoundError(f"Arquivo XML do Rekordbox não encontrado em: {xml_path}")

    print(f"[*] Lendo XML do Rekordbox: {xml_path}")
    tree = ET.parse(xml_path)
    root = tree.getroot()

    with protected_import_connection(db_path) as conn:
        cursor = conn.cursor()
    
        # 1. Mapear Collection do XML
        xml_tracks = {} # track_key -> track_dict
        collection = root.find("COLLECTION")
        if collection is not None:
            for t in collection.findall("TRACK"):
                key = t.get("TrackID")
                loc_url = t.get("Location", "")
                file_path = file_url_to_path(loc_url)
                xml_tracks[key] = {
                    "path": file_path,
                    "title": t.get("Name", ""),
                    "artist": t.get("Artist", ""),
                    "album": t.get("Album", ""),
                    "genre": t.get("Genre", ""),
                    "bpm": float(t.get("AverageBpm", 0) or 0),
                    "key": t.get("Key", ""),
                    "rating": int(t.get("Rating", 0) or 0),
                    "cues": []
                }
                # Cues
                for m in t.findall("POSITION_MARK"):
                    xml_tracks[key]["cues"].append({
                        "name": m.get("Name", ""),
                        "type": int(m.get("Type", 0)),
                        "start": float(m.get("Start", 0.0)),
                        "end": float(m.get("End", 0.0)) if m.get("End") else None,
                        "num": int(m.get("Num", -1)),
                        "red": int(m.get("Red", 0)),
                        "green": int(m.get("Green", 122)),
                        "blue": int(m.get("Blue", 255)),
                    })
    
        print(f"[*] Encontradas {len(xml_tracks)} faixas no XML do Rekordbox.")
    
        # 2. Sincronizar Cues para faixas existentes no Mixxx
        cues_synced = 0
        track_key_to_mixxx_id = {}
    
        for key, data in xml_tracks.items():
            # Procurar no Mixxx pelo caminho exato
            cursor.execute("SELECT id FROM track_locations WHERE location = ?", (data["path"],))
            loc_row = cursor.fetchone()
            if loc_row:
                cursor.execute("SELECT id, samplerate FROM library WHERE location = ?", (loc_row["id"],))
                lib_row = cursor.fetchone()
                if lib_row:
                    mixxx_id = lib_row["id"]
                    samplerate = lib_row["samplerate"] or 44100
                    track_key_to_mixxx_id[key] = mixxx_id
    
                    # Atualizar Cues
                    for cue in data["cues"]:
                        sample_pos = cue["start"] * (samplerate * 2.0)
                        length_samples = ((cue["end"] - cue["start"]) * (samplerate * 2.0)) if cue["end"] else 0
                        color_int = rgb_to_int(cue["red"], cue["green"], cue["blue"])
                        mixxx_cue_type = 4 if cue["type"] == 1 else 1 # 4 = Loop, 1 = HotCue
    
                        # Memory cues (hotcue = -1) casam por posição (±1 ms); hotcues por índice
                        if cue["num"] >= 0:
                            cursor.execute("""
                                SELECT id FROM cues
                                WHERE track_id = ? AND hotcue = ?
                            """, (mixxx_id, cue["num"]))
                            existing_cue = cursor.fetchone()
                        else:
                            tolerance = samplerate * 2.0 * 0.001
                            cursor.execute("""
                                SELECT id, position FROM cues
                                WHERE track_id = ? AND hotcue = -1
                            """, (mixxx_id,))
                            existing_cue = None
                            for cand in cursor.fetchall():
                                if abs(cand["position"] - sample_pos) <= tolerance:
                                    existing_cue = cand
                                    break
    
                        if existing_cue:
                            cursor.execute("""
                                UPDATE cues
                                SET position = ?, length = ?, type = ?, label = ?, color = ?
                                WHERE id = ?
                            """, (sample_pos, length_samples, mixxx_cue_type, cue["name"], color_int, existing_cue["id"]))
                        else:
                            cursor.execute("""
                                INSERT INTO cues (track_id, type, position, length, hotcue, label, color)
                                VALUES (?, ?, ?, ?, ?, ?, ?)
                            """, (mixxx_id, mixxx_cue_type, sample_pos, length_samples, cue["num"], cue["name"], color_int))
                        cues_synced += 1
    
        # 3. Sincronizar Playlists
        playlists_synced = 0
        playlists_root = root.find("PLAYLISTS")
        if playlists_root is not None:
            def process_node(node, path_prefix=""):
                nonlocal playlists_synced
                for child in node.findall("NODE"):
                    node_type = child.get("Type")
                    raw_name = child.get("Name", "Rekordbox Playlist")
                    if node_type == "1": # Playlist
                        full_name = f"{path_prefix}/{raw_name}" if path_prefix else raw_name
                        cursor.execute("SELECT id FROM Playlists WHERE name = ?", (full_name,))
                        pl_row = cursor.fetchone()
                        if pl_row:
                            pl_id = pl_row["id"]
                        else:
                            cursor.execute("INSERT INTO Playlists (name, position, hidden) VALUES (?, 0, 0)", (full_name,))
                            pl_id = cursor.lastrowid

                        # Limpar faixas anteriores e reinserir com a ordem do Rekordbox
                        cursor.execute("DELETE FROM PlaylistTracks WHERE playlist_id = ?", (pl_id,))
                        pos = 1
                        for t_el in child.findall("TRACK"):
                            t_key = t_el.get("Key")
                            if t_key in track_key_to_mixxx_id:
                                mid = track_key_to_mixxx_id[t_key]
                                cursor.execute("""
                                    INSERT INTO PlaylistTracks (playlist_id, track_id, position)
                                    VALUES (?, ?, ?)
                                """, (pl_id, mid, pos))
                                pos += 1
                        playlists_synced += 1
                    elif node_type == "0": # Folder
                        sub_prefix = path_prefix
                        if raw_name != "ROOT":
                            sub_prefix = f"{path_prefix}/{raw_name}" if path_prefix else raw_name
                        process_node(child, sub_prefix)

            process_node(playlists_root)
    

    print(f"[+] Sucesso! Sincronizados {cues_synced} Hot Cues e {playlists_synced} Playlists no Mixxx.")

# ==============================================================================
# STATUS
# ==============================================================================
def show_status(xml_path=DEFAULT_XML_PATH, db_path=DEFAULT_MIXXX_DB):
    print("==================================================")
    print("      MIXXX <-> REKORDBOX SYNC BRIDGE STATUS      ")
    print("==================================================")
    if os.path.exists(db_path):
        conn = get_mixxx_connection(db_path)
        c = conn.cursor()
        c.execute("SELECT count(*) FROM library")
        tracks_cnt = c.fetchone()[0]
        c.execute("SELECT count(*) FROM Playlists WHERE hidden = 0")
        pl_cnt = c.fetchone()[0]
        c.execute("SELECT count(*) FROM cues WHERE hotcue >= 0")
        cues_cnt = c.fetchone()[0]
        conn.close()
        print(f"[Mixxx DB] Local: {db_path}")
        print(f"           Faixas: {tracks_cnt} | Playlists: {pl_cnt} | Hot Cues: {cues_cnt}")
    else:
        print(f"[Mixxx DB] Não encontrado em: {db_path}")

    print("--------------------------------------------------")
    if os.path.exists(xml_path):
        size_kb = os.path.getsize(xml_path) / 1024
        mtime = datetime.fromtimestamp(os.path.getmtime(xml_path)).strftime('%Y-%m-%d %H:%M:%S')
        try:
            tree = ET.parse(xml_path)
            col = tree.getroot().find("COLLECTION")
            entries = col.get("Entries", "0") if col is not None else "0"
            print(f"[Rekordbox XML] Local: {xml_path}")
            print(f"                Tamanho: {size_kb:.1f} KB | Modificado: {mtime}")
            print(f"                Faixas no XML: {entries}")
        except Exception as e:
            print(f"[Rekordbox XML] Erro ao ler XML: {e}")
    else:
        print(f"[Rekordbox XML] Ainda não gerado em: {xml_path}")
        print("                Execute: python3 sync.py export")
    print("==================================================")

if __name__ == "__main__":
    if len(sys.argv) < 2 or sys.argv[1] in ["-h", "--help"]:
        print(__doc__)
        sys.exit(0)

    cmd = sys.argv[1].lower()
    target_xml = sys.argv[2] if len(sys.argv) > 2 else DEFAULT_XML_PATH

    if cmd == "export":
        export_mixxx_to_rekordbox_xml(target_xml)
    elif cmd == "import":
        import_rekordbox_xml_to_mixxx(target_xml)
    elif cmd == "status":
        show_status(target_xml)
    else:
        print(f"[!] Comando desconhecido: '{cmd}'")
        print(__doc__)
        sys.exit(1)
