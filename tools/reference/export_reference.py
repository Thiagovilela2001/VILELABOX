#!/usr/bin/env python3
"""Exportador de faixas e resultados de referência do VILELABOX.

Lê um perfil do Mixxx em modo estritamente somente leitura (SQLite com URI mode=ro),
extrai metadados, grades de batidas (decodificando src/proto/beats.proto),
frases (analysis/phrases/), cues e dados de vocal (analysis/vocals/), e grava em
tests/reference/<sha256>.json e tests/reference/index.json.

Uso:
    python3 tools/reference/export_reference.py [--profile <caminho_perfil>] [--output <dir_saida>]
        [--resources <dir_res>]
"""

import argparse
import hashlib
import json
import sqlite3
import struct
import sys
from pathlib import Path

# Recursos do repositório; o perfil instalado pode usar outra cópia (--resources).
DEFAULT_RESOURCES = Path(__file__).resolve().parents[2] / "res"

SCHEMA_VERSION = 1

# mixxx::CueType (src/track/cueinfo.h) -> tipo canônico.
CUE_TYPES = {1: "hotcue", 2: "cue", 4: "loop", 5: "jump", 6: "intro", 7: "outro"}

# PhraseType (src/track/phrases.h), na ordem do enum.
PHRASE_TYPES = ["UNKNOWN", "INTRO", "UP", "CHORUS", "DOWN", "VERSE", "OUTRO"]


def decode_varint(data: bytes, offset: int) -> tuple[int, int]:
    res = 0
    shift = 0
    while True:
        if offset >= len(data):
            raise ValueError("Varint truncado")
        b = data[offset]
        offset += 1
        res |= (b & 0x7F) << shift
        if not (b & 0x80):
            break
        shift += 7
    return res, offset


def zigzag_decode(n: int) -> int:
    return (n >> 1) ^ (-(n & 1))


def to_int32(n: int) -> int:
    """int32 negativo chega como varint de 64 bits (complemento de dois)."""
    return n - (1 << 64) if n >= (1 << 63) else n


def parse_bpm_message(data: bytes) -> float | None:
    offset = 0
    bpm = None
    while offset < len(data):
        tag, offset = decode_varint(data, offset)
        field_num = tag >> 3
        wire_type = tag & 0x07
        if field_num == 1 and wire_type == 1:
            if offset + 8 > len(data):
                break
            bpm = struct.unpack("<d", data[offset : offset + 8])[0]
            offset += 8
        elif wire_type == 0:
            _, offset = decode_varint(data, offset)
        elif wire_type == 1:
            offset += 8
        elif wire_type == 2:
            length, offset = decode_varint(data, offset)
            offset += length
        elif wire_type == 5:
            offset += 4
        else:
            break
    return bpm


def parse_beat_message(data: bytes) -> int | None:
    offset = 0
    frame_pos = None
    while offset < len(data):
        tag, offset = decode_varint(data, offset)
        field_num = tag >> 3
        wire_type = tag & 0x07
        if field_num == 1 and wire_type == 0:
            frame_pos, offset = decode_varint(data, offset)
            frame_pos = to_int32(frame_pos)
        elif wire_type == 0:
            _, offset = decode_varint(data, offset)
        elif wire_type == 1:
            offset += 8
        elif wire_type == 2:
            length, offset = decode_varint(data, offset)
            offset += length
        elif wire_type == 5:
            offset += 4
        else:
            break
    return frame_pos


def parse_beats_blob(blob: bytes) -> dict:
    """Decodifica BeatGrid ou BeatMap definidos em src/proto/beats.proto."""
    if not blob:
        return {}

    offset = 0
    result = {
        "bpm": None,
        "first_beat_frame": None,
        "first_downbeat_index": None,
        "beats": [],
    }

    try:
        while offset < len(blob):
            tag, offset = decode_varint(blob, offset)
            field_num = tag >> 3
            wire_type = tag & 0x07

            if wire_type == 2:  # Length-delimited
                length, offset = decode_varint(blob, offset)
                field_data = blob[offset : offset + length]
                offset += length

                if field_num == 1:
                    # Pode ser BeatGrid.bpm ou BeatMap.beat
                    # Tentamos decodificar como Bpm
                    bpm_val = parse_bpm_message(field_data)
                    if bpm_val is not None:
                        result["bpm"] = bpm_val
                    else:
                        beat_val = parse_beat_message(field_data)
                        if beat_val is not None:
                            result["beats"].append(beat_val)
                elif field_num == 2:
                    # BeatGrid.first_beat
                    beat_val = parse_beat_message(field_data)
                    if beat_val is not None:
                        result["first_beat_frame"] = beat_val
            elif wire_type == 0:  # Varint
                val, offset = decode_varint(blob, offset)
                # sint32 first_downbeat_index: field 2 (BeatMap) ou field 3 (BeatGrid)
                if field_num in (2, 3):
                    result["first_downbeat_index"] = zigzag_decode(val)
            elif wire_type == 1:
                offset += 8
            elif wire_type == 5:
                offset += 4
            else:
                break
    except Exception:
        # Retorna o que foi possível extrair
        pass

    # BeatMap não tem first_beat: a primeira batida é o início da lista.
    if result["first_beat_frame"] is None and result["beats"]:
        result["first_beat_frame"] = result["beats"][0]
    return result


def compute_file_sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()


def load_phrases_index(profile: Path) -> dict:
    """Lê os caches de frases uma vez: location -> frases."""
    index = {}
    for path in (profile / "analysis" / "phrases").glob("*.phrases.json"):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            continue
        if data.get("location"):
            index[data["location"]] = data.get("phrases", [])
    return index


def load_vocals_for_track(profile: Path, location: str, resources: Path) -> list:
    """O cache não guarda a location: o nome é o SHA-256 da identidade montada
    em src/analyzer/analyzervocals.cpp (initialize): location, tamanho, mtime
    em ms, model.json, vocal_detector.py e o motor. Testa os dois motores."""
    try:
        model = (resources / "vocals" / "model.json").read_bytes()
        script = (resources / "vocals" / "vocal_detector.py").read_bytes()
    except OSError:
        return []
    stat = Path(location).stat()
    base = (location.encode("utf-8") + str(stat.st_size).encode()
            + str(stat.st_mtime_ns // 1_000_000).encode() + model + script)
    for engine in (b"demucs", b"classifier"):
        cache = profile / "analysis" / "vocals" / f"{hashlib.sha256(base + engine).hexdigest()}.json"
        try:
            return json.loads(cache.read_text(encoding="utf-8")).get("segments", [])
        except (OSError, ValueError):
            continue
    return None


def load_energies(profile: Path) -> dict:
    """Tabela energy de analysis/blackmesa.sqlite (src/library/related/relatedtracks.cpp)."""
    db = profile / "analysis" / "blackmesa.sqlite"
    if not db.is_file():
        return {}
    conn = sqlite3.connect(db.as_uri() + "?mode=ro", uri=True)
    try:
        return dict(conn.execute("SELECT identity, value FROM energy"))
    except sqlite3.OperationalError:
        return {}  # banco sem a tabela: energia nunca calculada
    finally:
        conn.close()


def energy_identity(location: str) -> str:
    """energyIdentity: location, tamanho e mtime em ms separados por NUL + "energy-v1"."""
    stat = Path(location).stat()
    raw = (location.encode("utf-8") + b"\0" + str(stat.st_size).encode() + b"\0"
           + str(stat.st_mtime_ns // 1_000_000).encode() + b"energy-v1")
    return hashlib.sha256(raw).hexdigest()


def canonical_phrases(rows: list) -> list[dict]:
    """Cache de frases ([início, fim, compassos, tipo, confiança]) no esquema v1."""
    phrases = []
    for start, end, bars, kind, confidence in rows:
        name = PHRASE_TYPES[kind] if 0 <= kind < len(PHRASE_TYPES) else f"TYPE_{kind}"
        phrases.append({"start": start, "end": end, "bars": bars, "type": name,
                        "confidence": confidence})
    return phrases


def canonical_cues(rows: list[dict], rate: int) -> list[dict]:
    """Cues do Mixxx no esquema v1. O banco guarda posição e duração em
    amostras do engine (estéreo, 2 por frame); hotcue -1 é "sem slot"."""
    cues = []
    for c in rows:
        kind = CUE_TYPES.get(c["type"])
        if kind is None or c["position"] is None or c["position"] < 0:
            continue  # Invalid, Beat e N60dBSound não são cues do usuário
        cue = {"type": kind, "position_s": c["position"] / (2 * rate)}
        if c["length"]:
            cue["length_s"] = c["length"] / (2 * rate)
        if c["hotcue"] is not None and c["hotcue"] >= 0:
            cue["hotcue"] = c["hotcue"]
        if c["label"]:
            cue["label"] = c["label"]
        if c["color"] is not None:
            cue["color"] = c["color"] & 0xFFFFFF
        cues.append(cue)
    return cues


def export_reference(profile_dir: Path, output_dir: Path, resources_dir: Path = DEFAULT_RESOURCES) -> dict:
    profile_dir = Path(profile_dir).resolve()
    db_path = profile_dir / "mixxxdb.sqlite"
    if not db_path.is_file():
        raise FileNotFoundError(f"mixxxdb.sqlite não encontrado em {profile_dir}")

    output_dir = Path(output_dir).resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    # Abre SQLite estritamente em modo somente leitura (mode=ro)
    conn = sqlite3.connect(db_path.as_uri() + "?mode=ro", uri=True)
    conn.row_factory = sqlite3.Row

    cursor = conn.cursor()
    cursor.execute("""
        SELECT l.id, l.bpm, l.duration, l.samplerate, l.key, l.beats, tl.location
        FROM library l
        JOIN track_locations tl ON l.location = tl.id
        WHERE tl.location IS NOT NULL
    """)
    tracks = cursor.fetchall()

    index_entries = []
    missing = 0
    duplicates = 0
    seen = set()
    locations = {}
    phrases_index = load_phrases_index(profile_dir)
    energies = load_energies(profile_dir)

    for row in tracks:
        track_id = row["id"]
        location = row["location"]
        audio_path = Path(location)

        # Sem o áudio não há identificador estável (o hash do caminho mudaria
        # de máquina para máquina): a faixa fica de fora e é contada.
        if not audio_path.is_file():
            missing += 1
            continue
        audio_hash = compute_file_sha256(audio_path)
        # O mesmo áudio em dois caminhos tem um só arquivo de referência.
        if audio_hash in seen:
            duplicates += 1
            continue
        seen.add(audio_hash)
        locations[location] = audio_hash  # só o caminho que gerou a referência

        # Decodifica beats do protobuf
        beats_info = parse_beats_blob(row["beats"] or b"")

        # Cues
        c_cursor = conn.cursor()
        c_cursor.execute("""
            SELECT type, position, length, hotcue, label, color
            FROM cues
            WHERE track_id = ?
            ORDER BY position
        """, (track_id,))
        cues_list = [dict(c) for c in c_cursor.fetchall()]

        # Frases e vocais dos caches de análise
        phrases = phrases_index.get(location, [])
        vocal_segments = load_vocals_for_track(profile_dir, location, Path(resources_dir))

        rate = row["samplerate"] or None
        bpm = row["bpm"] or beats_info.get("bpm") or None
        first_frame = beats_info.get("first_beat_frame")
        downbeat = beats_info.get("first_downbeat_index")
        track_data = {
            "schema_version": SCHEMA_VERSION,
            "sha256": audio_hash,
            "source": "mixxx",
            "sample_rate": rate,
            "duration": row["duration"],
            "bpm": bpm,
            # BeatMap (lista de batidas) é a grade de andamento variável do Mixxx.
            "tempo_type": "variable" if beats_info.get("beats") else "fixed",
            "first_beat_time_s": first_frame / rate if first_frame is not None and rate else None,
            # O proto guarda o índice absoluto da batida de downbeat
            # (src/track/beats.cpp); o esquema guarda a fase em 4/4.
            "downbeat_index": downbeat % 4 if downbeat is not None else None,
            "key": row["key"] or None,
            "energy": energies.get(energy_identity(location)),
            # Frases exigem bpm no esquema (tolerância de 1 compasso).
            "phrases": canonical_phrases(phrases) if bpm else [],
            "cues": canonical_cues(cues_list, rate) if rate else [],
            "has_vocal": bool(vocal_segments) if vocal_segments is not None else None,
            "vocal_segments": [{"start": s, "end": e} for s, e in vocal_segments]
            if vocal_segments is not None else None,
        }
        track_data = {k: v for k, v in track_data.items() if v is not None}

        # Salva arquivo por hash
        out_file = output_dir / f"{audio_hash}.json"
        out_file.write_text(json.dumps(track_data, indent=2, ensure_ascii=False), encoding="utf-8")

        # Sem artista/título: o índice pode ser versionado sem expor a biblioteca.
        index_entries.append({
            "sha256": audio_hash,
            "bpm": track_data.get("bpm"),
            "duration": track_data.get("duration"),
            "key": track_data.get("key"),
            "cues_count": len(track_data["cues"]),
            "phrases_count": len(track_data["phrases"]),
            "vocal_segments_count": len(track_data.get("vocal_segments", [])),
        })

    index_data = {
        "count": len(index_entries),
        "missing_audio": missing,
        "duplicate_audio": duplicates,
        "tracks": sorted(index_entries, key=lambda x: x["sha256"]),
    }
    # Caminho -> SHA-256 para testes locais casarem o banco com as referências
    # (rust/vilelabox-mixxx-compat/tests). Tem caminhos pessoais: nunca versionar.
    (output_dir / "locations.json").write_text(
        json.dumps(locations, indent=2, ensure_ascii=False), encoding="utf-8")
    (output_dir / "index.json").write_text(json.dumps(index_data, indent=2, ensure_ascii=False), encoding="utf-8")
    conn.close()
    return index_data


def main():
    parser = argparse.ArgumentParser(description="Exporta conjunto de referência do VILELABOX")
    parser.add_argument("--profile", default=str(Path.home() / ".mixxx"), help="Caminho do perfil do Mixxx")
    parser.add_argument("--output", default="tests/reference", help="Diretório de saída para os JSONs")
    parser.add_argument("--resources", default=str(DEFAULT_RESOURCES),
                        help="Diretório res/ usado pelo VILELABOX (identifica o cache de vocal)")
    args = parser.parse_args()

    try:
        index_data = export_reference(Path(args.profile), Path(args.output), Path(args.resources))
        print(f"Exportadas com sucesso {index_data['count']} faixas para {args.output}")
        if index_data["missing_audio"]:
            print(f"{index_data['missing_audio']} faixas ignoradas: arquivo de áudio ausente")
    except Exception as e:
        print(f"Erro ao exportar referências: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
