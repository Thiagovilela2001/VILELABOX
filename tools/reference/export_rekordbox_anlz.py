#!/usr/bin/env python3
"""
Exportador de análises do rekordbox (R1.2a)
===========================================
Lê arquivos ANLZ (.DAT e .EXT) gerados pelo rekordbox em
%APPDATA%\\Pioneer\\rekordbox\\share\\PIONEER\\USBANLZ e converte para o
formato canônico de referência do VILELABOX.

Extrai:
  - BeatGrid (PQTZ em .DAT): batidas, andamento (BPM) e downbeat
  - Cues e Loops (PCOB / PCO2 em .EXT): memory cues e hot cues
  - Frases / Song Structure (PSSI em .EXT): estrutura com unmasking XOR
  - Identificador SHA-256 do arquivo de áudio real correspondente
"""

import argparse
import hashlib
import json
import os
import struct
import sys
from pathlib import Path

SCHEMA_VERSION = 1

PHRASE_HIGH_NAMES = {
    1: "intro",
    2: "up",
    3: "down",
    5: "chorus",
    6: "outro",
}

PHRASE_MID_NAMES = {
    1: "intro",
    2: "verse_1",
    3: "verse_2",
    4: "verse_3",
    5: "verse_4",
    6: "verse_5",
    7: "verse_6",
    8: "bridge",
    9: "chorus",
    10: "outro",
}

PHRASE_LOW_NAMES = {
    1: "intro",
    2: "verse_1",
    3: "verse_1",
    4: "verse_1",
    5: "verse_2",
    6: "verse_2",
    7: "verse_2",
    8: "bridge",
    9: "chorus",
    10: "outro",
}


def compute_file_sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()


def parse_anlz_sections(data: bytes) -> dict[str, bytes]:
    """Separa as seções type-tagged (PQTZ, PCOB, PSSI, etc.) de um arquivo ANLZ."""
    if len(data) < 12 or data[:4] != b"PMAI":
        return {}
    len_hdr, _ = struct.unpack(">II", data[4:12])
    pos = len_hdr
    sections = {}
    while pos + 12 <= len(data):
        fourcc = data[pos : pos + 4].decode("latin1", errors="ignore")
        _, len_tag = struct.unpack(">II", data[pos + 4 : pos + 12])
        if len_tag < 12 or pos + len_tag > len(data):
            break
        sections[fourcc] = data[pos + 12 : pos + len_tag]
        pos += len_tag
    return sections


def parse_ppth(payload: bytes) -> str | None:
    """Extrai caminho da faixa de áudio de uma seção PPTH."""
    if len(payload) < 4:
        return None
    len_path = struct.unpack(">I", payload[:4])[0]
    if len(payload) < 4 + len_path:
        return None
    raw = payload[4 : 4 + max(0, len_path - 2)]
    try:
        return raw.decode("utf-16be").strip("\x00")
    except UnicodeDecodeError:
        return None


def parse_pqtz(payload: bytes) -> dict | None:
    """Extrai grid de batidas e andamento de uma seção PQTZ."""
    if len(payload) < 12:
        return None
    num_beats = struct.unpack(">I", payload[8:12])[0]
    beat_data = payload[12:]
    if len(beat_data) < num_beats * 8:
        return None

    beats = []
    downbeat_index = None
    for i in range(num_beats):
        offset = i * 8
        beat_num, tempo_raw, time_ms = struct.unpack(">HHI", beat_data[offset : offset + 8])
        bpm = tempo_raw / 100.0
        beats.append({
            "beat_number": beat_num,
            "bpm": round(bpm, 2),
            "time_ms": time_ms,
            "time_s": round(time_ms / 1000.0, 3),
        })
        if beat_num == 1 and downbeat_index is None:
            downbeat_index = i

    if not beats:
        return None

    first_beat_time_s = beats[0]["time_s"]
    avg_bpm = beats[0]["bpm"]
    return {
        "bpm": avg_bpm,
        "tempo_type": "variable" if len({b["bpm"] for b in beats}) > 1 else "fixed",
        "first_beat_time_s": first_beat_time_s,
        "downbeat_index": downbeat_index or 0,
        "beats_count": len(beats),
        "beats": beats,
    }


def parse_pssi(payload: bytes, beat_times_ms: list[int] | None = None) -> list[dict]:
    """Extrai estrutura de frases com unmasking XOR de uma seção PSSI."""
    if len(payload) < 6:
        return []
    len_entry_bytes, len_entries = struct.unpack(">IH", payload[:6])
    if len_entries == 0 or len_entry_bytes < 6:
        return []

    body = bytearray(payload[6:])
    if len(body) < 2:
        return []
    raw_mood = struct.unpack(">H", body[:2])[0]

    if raw_mood > 20:
        c = len_entries & 0xFF
        mask = bytes([(b + c) & 0xFF for b in [
            0xCB, 0xE1, 0xEE, 0xFA, 0xE5, 0xEE, 0xAD, 0xEE,
            0xE9, 0xD2, 0xE9, 0xEB, 0xE1, 0xE9, 0xF3, 0xE8,
            0xE9, 0xF4, 0xE1
        ]])
        for i in range(len(body)):
            body[i] ^= mask[i % 19]

    if len(body) < 14:
        return []

    mood = struct.unpack(">H", body[:2])[0]
    name_map = PHRASE_HIGH_NAMES if mood == 1 else (PHRASE_MID_NAMES if mood == 2 else PHRASE_LOW_NAMES)

    phrases = []
    pos = 14
    for _ in range(len_entries):
        if pos + len_entry_bytes > len(body):
            break
        p_idx, p_beat, p_kind = struct.unpack(">HHH", body[pos : pos + 6])
        kind_name = name_map.get(p_kind, f"phrase_{p_kind}")

        start_time_s = None
        if beat_times_ms and 1 <= p_beat <= len(beat_times_ms):
            start_time_s = round(beat_times_ms[p_beat - 1] / 1000.0, 3)

        phrases.append({
            "index": p_idx,
            "beat": p_beat,
            "kind": kind_name,
            "kind_id": p_kind,
            "start_time_s": start_time_s,
        })
        pos += len_entry_bytes

    return phrases


def parse_cues(payload: bytes) -> list[dict]:
    """Extrai pontos de cue e loops de seções PCOB ou PCO2."""
    if len(payload) < 8:
        return []
    cue_type_flag, num_cues = struct.unpack(">IH", payload[:6])
    cues = []
    pos = 8
    while pos + 40 <= len(payload):
        if payload[pos : pos + 4] != b"PCPT":
            pos += 1
            continue
        len_hdr, len_entry = struct.unpack(">II", payload[pos + 4 : pos + 12])
        if pos + len_entry > len(payload):
            break
        entry = payload[pos + 12 : pos + len_entry]
        if len(entry) >= 28:
            hot_cue = struct.unpack(">I", entry[:4])[0]
            status = struct.unpack(">I", entry[4:8])[0]
            c_type = entry[16]
            time_ms = struct.unpack(">I", entry[20:24])[0]
            loop_ms = struct.unpack(">I", entry[24:28])[0]
            cues.append({
                "hot_cue": hot_cue,
                "status": status,
                "is_loop": c_type == 2,
                "start_time_s": round(time_ms / 1000.0, 3),
                "end_time_s": round(loop_ms / 1000.0, 3) if c_type == 2 and loop_ms > time_ms else None,
            })
        pos += len_entry
    return cues


def canonical_phrases(raw: list[dict]) -> list[dict]:
    """Frases do PSSI no esquema v1: início em segundos, fim no início da
    próxima e compassos pela distância em batidas (4/4). A última frase fica
    sem fim; frases com batida fora da grade são descartadas."""
    raw = [p for p in raw if p["start_time_s"] is not None]
    phrases = []
    for p, nxt in zip(raw, raw[1:] + [None]):
        kind = "VERSE" if p["kind"].startswith("verse") else p["kind"].upper()
        phrase = {"start": p["start_time_s"], "type": kind}
        if nxt is not None:
            phrase["end"] = nxt["start_time_s"]
            phrase["bars"] = (nxt["beat"] - p["beat"]) // 4
        phrases.append(phrase)
    return phrases


def canonical_cues(raw: list[dict]) -> list[dict]:
    """Cues do rekordbox no esquema v1. hot_cue 0 é memory cue; 1..8 são os
    slots A..H (hotcue 0..7)."""
    cues = []
    for c in raw:
        cue = {
            "type": "loop" if c["is_loop"] else ("hotcue" if c["hot_cue"] else "memory"),
            "position_s": c["start_time_s"],
        }
        if c["is_loop"] and c["end_time_s"] is not None:
            cue["length_s"] = round(c["end_time_s"] - c["start_time_s"], 3)
        if c["hot_cue"]:
            cue["hotcue"] = c["hot_cue"] - 1
        cues.append(cue)
    return sorted(cues, key=lambda c: (c["position_s"], c["type"], c.get("hotcue", -1)))


def build_music_index(music_dir: Path | None, mixxx_db: Path | None) -> dict[str, Path]:
    """Indexa arquivos de áudio por nome de arquivo (stem e nome completo) para resolução rápida."""
    index = {}
    if mixxx_db and mixxx_db.is_file():
        import sqlite3
        try:
            conn = sqlite3.connect(f"{mixxx_db.as_uri()}?mode=ro", uri=True)
            for row in conn.execute("SELECT location FROM track_locations WHERE fs_deleted = 0"):
                p = Path(row[0])
                if p.is_file():
                    index[p.name.lower()] = p
                    index[p.stem.lower()] = p
            conn.close()
        except Exception:
            pass

    if music_dir and music_dir.is_dir():
        for ext in ("*.mp3", "*.flac", "*.wav", "*.aif", "*.aiff", "*.m4a"):
            for p in music_dir.rglob(ext):
                index[p.name.lower()] = p
                index[p.stem.lower()] = p

    return index


def resolve_audio_file(anlz_path_str: str | None, music_index: dict[str, Path]) -> Path | None:
    if not anlz_path_str:
        return None
    clean = anlz_path_str.replace("?/", "").replace("\\", "/")
    p = Path(clean)
    if p.is_file():
        return p
    name_key = p.name.lower()
    if name_key in music_index:
        return music_index[name_key]
    stem_key = p.stem.lower()
    if stem_key in music_index:
        return music_index[stem_key]
    return None


def export_rekordbox_anlz(
    anlz_dir: Path,
    output_dir: Path,
    music_dir: Path | None = None,
    mixxx_db: Path | None = None,
    limit: int | None = None,
) -> dict:
    anlz_dir = Path(anlz_dir).resolve()
    output_dir = Path(output_dir).resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"[*] Indexando biblioteca musical para resolução de caminhos...")
    music_index = build_music_index(music_dir, mixxx_db)
    print(f"[+] {len(music_index)} arquivos/nomes indexados.")

    dat_files = sorted(anlz_dir.rglob("*.DAT"))
    if limit:
        dat_files = dat_files[:limit]

    print(f"[*] Processando {len(dat_files)} arquivos .DAT em {anlz_dir}...")

    index_entries = []
    missing_audio = 0
    with_phrases = 0
    with_cues = 0

    for dat_path in dat_files:
        ext_path = dat_path.with_suffix(".EXT")
        dat_data = dat_path.read_bytes()
        ext_data = ext_path.read_bytes() if ext_path.is_file() else b""

        dat_secs = parse_anlz_sections(dat_data)
        ext_secs = parse_anlz_sections(ext_data) if ext_data else {}

        raw_path = parse_ppth(ext_secs.get("PPTH") or dat_secs.get("PPTH") or b"")
        audio_file = resolve_audio_file(raw_path, music_index)

        # Sem o áudio não há SHA-256 estável: a faixa fica de fora e é contada,
        # como em export_reference.py.
        if not (audio_file and audio_file.is_file()):
            missing_audio += 1
            continue
        sha256_hash = compute_file_sha256(audio_file)

        grid_info = parse_pqtz(dat_secs.get("PQTZ") or b"")
        beat_times = [b["time_ms"] for b in grid_info["beats"]] if grid_info else []

        phrases = []
        if "PSSI" in ext_secs and grid_info:
            phrases = canonical_phrases(parse_pssi(ext_secs["PSSI"], beat_times))
            if phrases:
                with_phrases += 1

        raw_cues = []
        for tag in ("PCO2", "PCOB"):
            if tag in ext_secs:
                raw_cues.extend(parse_cues(ext_secs[tag]))
            elif tag in dat_secs:
                raw_cues.extend(parse_cues(dat_secs[tag]))
        cues = canonical_cues(raw_cues)
        if cues:
            with_cues += 1

        # Esquema canônico v1 (rust/vilelabox-reference). O ANLZ não guarda
        # taxa de amostragem, duração nem vocal: esses campos ficam ausentes.
        track_data = {
            "schema_version": SCHEMA_VERSION,
            "sha256": sha256_hash,
            "source": "rekordbox_anlz",
            "source_ref": str(dat_path.relative_to(anlz_dir)).replace("\\", "/"),
            "bpm": grid_info["bpm"] if grid_info else None,
            "tempo_type": grid_info["tempo_type"] if grid_info else "fixed",
            "first_beat_time_s": grid_info["first_beat_time_s"] if grid_info else None,
            "downbeat_index": grid_info["downbeat_index"] if grid_info else None,
            "phrases": phrases,
            "cues": cues,
        }
        track_data = {k: v for k, v in track_data.items() if v is not None}

        out_file = output_dir / f"{sha256_hash}.json"
        out_file.write_text(json.dumps(track_data, indent=2, ensure_ascii=False), encoding="utf-8")

        index_entries.append({
            "sha256": sha256_hash,
            "bpm": track_data.get("bpm"),
            "first_beat_time_s": track_data.get("first_beat_time_s"),
            "cues_count": len(cues),
            "phrases_count": len(phrases),
        })

    index_data = {
        "count": len(index_entries),
        "with_phrases": with_phrases,
        "with_cues": with_cues,
        "missing_audio": missing_audio,
        "tracks": sorted(index_entries, key=lambda x: x["sha256"]),
    }
    (output_dir / "index.json").write_text(json.dumps(index_data, indent=2, ensure_ascii=False), encoding="utf-8")

    print(f"[+] Concluído! Exportadas {len(index_entries)} faixas ({with_phrases} com frases, {with_cues} com cues, {missing_audio} sem áudio correspondente).")
    return index_data


def main():
    default_anlz = Path(os.environ.get("APPDATA", "")) / "Pioneer" / "rekordbox" / "share" / "PIONEER" / "USBANLZ"
    default_music = Path(os.environ.get("USERPROFILE", "")) / "Music"
    default_db = Path("bars-profile-win") / "mixxxdb.sqlite"

    parser = argparse.ArgumentParser(description="Exporta análises ANLZ do rekordbox para formato canônico de referência")
    parser.add_argument("--anlz-dir", default=str(default_anlz), help="Diretório USBANLZ do rekordbox")
    parser.add_argument("--output", default="tests/reference/rekordbox", help="Diretório de saída para os JSONs")
    parser.add_argument("--music-dir", default=str(default_music), help="Diretório de arquivos de música")
    parser.add_argument("--mixxx-db", default=str(default_db), help="Banco SQLite do Mixxx para resolver caminhos")
    parser.add_argument("--limit", type=int, default=None, help="Limite de faixas para exportar")

    args = parser.parse_args()
    export_rekordbox_anlz(
        anlz_dir=Path(args.anlz_dir),
        output_dir=Path(args.output),
        music_dir=Path(args.music_dir) if args.music_dir else None,
        mixxx_db=Path(args.mixxx_db) if args.mixxx_db else None,
        limit=args.limit,
    )


if __name__ == "__main__":
    main()
