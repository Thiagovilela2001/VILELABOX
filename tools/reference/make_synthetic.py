#!/usr/bin/env python3
"""Gera casos sintéticos determinísticos de áudio e seus JSONs de referência.

Matriz de fixtures (PLANO-RUST.md, R1.3), WAVs gerados só com a biblioteca padrão:
- fixed_120bpm: andamento fixo a 120 BPM com cliques limpos.
- tempo_change_120_to_140bpm: mudança de andamento 120 -> 140 BPM.
- silent_intro_125bpm: introdução sem batidas seguida de ritmo a 125 BPM.
- first_beat_before_zero_128bpm: grade ancorada antes do frame zero.
- no_downbeat_120bpm: grade sem downbeat definido.
- short_audio_1s e silence_10s: áudio curto e silêncio total.
- cues_and_loops_124bpm: oito hot cues fracionários, dois memory cues e loops.
- phrases_8_16_32_bars_128bpm: frases de 8, 16 e 32 compassos.
- instrumental_126bpm e vocal_segments_126bpm: sem e com segmentos vocais.
- playlists_tree.json: playlists vazias, aninhadas e homônimas.

Os WAVs não são versionados (tests/reference/synthetic/wavs/ com .gitignore).
Os JSONs seguem o esquema canônico v1 de rust/vilelabox-reference.

Uso:
    python3 tools/reference/make_synthetic.py [--output-dir tests/reference/synthetic] [--wav-dir tests/reference/synthetic/wavs]
"""

import argparse
from array import array
import hashlib
import json
import math
from pathlib import Path
import sys
import wave

SCHEMA_VERSION = 1
RATE = 44100


def generate_click_sample(t_local: float) -> float:
    """Mesma função de clique dos testes C++ (AnalyzerGridTest, clickTrack)."""
    return math.exp(-t_local * 250.0) * math.cos(t_local * 2.0 * math.pi * 180.0)


def beat_times(first: float, bpm: float, until: float) -> list[float]:
    times = []
    t = first
    while t < until - 0.05:
        times.append(t)
        t += 60.0 / bpm
    return times


def synthesize(duration: float, beats: list[float], tones: tuple = ()) -> bytes:
    """PCM int16 estéreo: cliques de 30 ms nas batidas e tons (início, fim, Hz)
    que fazem o papel de voz. Batidas antes de 0 contribuem só com a cauda."""
    total = int(duration * RATE)
    buf = [0.0] * total
    click_len = int(RATE * 0.03)
    for bf in sorted({int(bt * RATE) for bt in beats}):
        for j in range(max(0, -bf), min(click_len, total - bf)):
            buf[bf + j] += generate_click_sample(j / RATE)
    for start, end, freq in tones:
        for i in range(int(start * RATE), min(int(end * RATE), total)):
            t = i / RATE
            buf[i] += 0.3 * math.sin(2 * math.pi * freq * t) + 0.1 * math.sin(4 * math.pi * freq * t)
    pcm = array("h")
    for s in buf:
        v = int(max(-1.0, min(1.0, s)) * 32767.0)
        pcm.append(v)
        pcm.append(v)
    if sys.byteorder != "little":
        pcm.byteswap()
    return pcm.tobytes()


def write_wav(path: Path, raw_pcm: bytes):
    with wave.open(str(path), "wb") as w:
        w.setnchannels(2)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(raw_pcm)


def sha256_of_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()


def phrases_from_bars(first: float, bpm: float, spec: list[tuple[int, str]]) -> list[dict]:
    bar = 240.0 / bpm
    phrases, t = [], first
    for bars, kind in spec:
        phrases.append({"start": round(t, 6), "end": round(t + bars * bar, 6), "bars": bars,
                        "type": kind, "confidence": 1.0})
        t += bars * bar
    return phrases


def build_cases() -> list[tuple[dict, bytes]]:
    """(referência sem sha256, PCM) de cada caso."""
    cases = []

    def add(name, duration, beats, tones=(), **ref):
        cases.append(({"name": name, "duration": duration, **ref}, synthesize(duration, beats, tones)))

    add("fixed_120bpm", 24.0, beat_times(0.317, 120.0, 24.0),
        bpm=120.0, first_beat_time_s=0.317, downbeat_index=0,
        phrases=[
            {"start": 0.317, "end": 8.317, "bars": 4, "type": "INTRO", "confidence": 1.0},
            {"start": 8.317, "end": 24.0, "bars": 8, "type": "CHORUS", "confidence": 1.0},
        ],
        cues=[
            {"type": "hotcue", "position_s": 0.317, "hotcue": 0, "label": "MIX IN"},
            {"type": "hotcue", "position_s": 8.317, "hotcue": 1, "label": "DROP"},
        ],
        has_vocal=False)

    split = 16.0
    # beat_times para em until - 0.05: o primeiro trecho termina antes de split.
    change_beats = beat_times(0.5, 120.0, split + 0.05) + beat_times(split + 60.0 / 140.0, 140.0, 32.0)
    add("tempo_change_120_to_140bpm", 32.0, change_beats,
        bpm=120.0, tempo_type="variable",
        sections=[
            {"start": 0.5, "end": 16.0, "bpm": 120.0},
            {"start": 16.0 + 60.0 / 140.0, "end": 32.0, "bpm": 140.0},
        ],
        first_beat_time_s=0.5, downbeat_index=0, has_vocal=False)

    add("silent_intro_125bpm", 24.0, beat_times(5.0, 125.0, 24.0),
        bpm=125.0, first_beat_time_s=5.0, downbeat_index=0, has_vocal=False)

    add("first_beat_before_zero_128bpm", 16.0, beat_times(-0.01, 128.0, 16.0),
        bpm=128.0, first_beat_time_s=-0.01, downbeat_index=0, has_vocal=False)

    # Sem downbeat_index: a referência não define a fase do compasso.
    add("no_downbeat_120bpm", 16.0, beat_times(0.25, 120.0, 16.0),
        bpm=120.0, first_beat_time_s=0.25, has_vocal=False)

    add("short_audio_1s", 1.0, beat_times(0.1, 120.0, 1.0),
        bpm=120.0, first_beat_time_s=0.1, has_vocal=False)

    # Silêncio: sem bpm nem batidas; só a ausência de voz é afirmada.
    add("silence_10s", 10.0, [], has_vocal=False, vocal_segments=[])

    bpm, first = 124.0, 0.2
    bar = 240.0 / bpm
    # Posições fora da grade de frames (fração de amostra) de propósito.
    hotcues = [{"type": "hotcue", "position_s": round(first + k * bar + 0.0001234, 7), "hotcue": k}
               for k in range(8)]
    cues = sorted(hotcues + [
        {"type": "memory", "position_s": round(first + 2.5 * bar, 7), "label": "BREAK"},
        {"type": "memory", "position_s": round(first + 9.25 * bar, 7)},
        {"type": "loop", "position_s": round(first + 4 * bar, 7), "length_s": round(bar, 7)},
        {"type": "loop", "position_s": round(first + 12 * bar, 7), "length_s": round(bar / 4, 7),
         "label": "1 BEAT"},
    ], key=lambda c: (c["position_s"], c["type"]))
    add("cues_and_loops_124bpm", 32.0, beat_times(first, bpm, 32.0),
        bpm=bpm, first_beat_time_s=first, downbeat_index=0, cues=cues, has_vocal=False)

    bpm, first = 128.0, 0.1
    phrases = phrases_from_bars(first, bpm, [(8, "INTRO"), (16, "UP"), (32, "CHORUS")])
    duration = round(phrases[-1]["end"] + 0.4, 3)
    add("phrases_8_16_32_bars_128bpm", duration, beat_times(first, bpm, duration),
        bpm=bpm, first_beat_time_s=first, downbeat_index=0, phrases=phrases, has_vocal=False)

    add("instrumental_126bpm", 16.0, beat_times(0.2, 126.0, 16.0),
        bpm=126.0, first_beat_time_s=0.2, downbeat_index=0, has_vocal=False, vocal_segments=[])

    segments = [(4.0, 10.0), (14.0, 20.0)]
    add("vocal_segments_126bpm", 24.0, beat_times(0.2, 126.0, 24.0),
        tones=tuple((s, e, 220.0) for s, e in segments),
        bpm=126.0, first_beat_time_s=0.2, downbeat_index=0, has_vocal=True,
        vocal_segments=[{"start": s, "end": e} for s, e in segments])
    return cases


def playlists_tree(hashes: dict[str, str]) -> dict:
    """Vazia, aninhada e homônima (mesmo nome em pastas diferentes e entre irmãos)."""
    h = hashes
    return {
        "schema_version": SCHEMA_VERSION,
        "name": "playlists_tree",
        "playlists": [
            {"name": "Vazia"},
            {"name": "Sets", "children": [
                {"name": "Warmup", "tracks": [h["fixed_120bpm"], h["silent_intro_125bpm"]]},
                {"name": "Favoritas", "tracks": [h["cues_and_loops_124bpm"]]},
                {"name": "Peak", "children": [
                    {"name": "Favoritas", "tracks": [h["vocal_segments_126bpm"], h["fixed_120bpm"]]},
                    {"name": "Vazia"},
                ]},
            ]},
            {"name": "Favoritas", "tracks": [h["instrumental_126bpm"]]},
            {"name": "Favoritas", "tracks": []},
        ],
    }


def create_synthetic_suite(output_dir: Path, wav_dir: Path) -> dict:
    output_dir = Path(output_dir).resolve()
    wav_dir = Path(wav_dir).resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    wav_dir.mkdir(parents=True, exist_ok=True)

    # Garante que arquivos WAV gerados não sejam versionados
    gitignore_file = wav_dir / ".gitignore"
    if not gitignore_file.exists():
        gitignore_file.write_text("*.wav\n", encoding="utf-8")

    entries, hashes = [], {}
    for ref, pcm in build_cases():
        name = ref["name"]
        wav = wav_dir / f"{name}.wav"
        write_wav(wav, pcm)
        hashes[name] = sha256_of_file(wav)
        case = {"schema_version": SCHEMA_VERSION, "name": name, "sha256": hashes[name],
                "source": "synthetic", "sample_rate": RATE, **ref}
        case.setdefault("tempo_type", "fixed")
        (output_dir / f"{name}.json").write_text(json.dumps(case, indent=2) + "\n", encoding="utf-8")
        entries.append({"name": name, "sha256": hashes[name], "duration": case["duration"],
                        "bpm": case.get("bpm"), "tempo_type": case["tempo_type"]})

    (output_dir / "playlists_tree.json").write_text(
        json.dumps(playlists_tree(hashes), indent=2) + "\n", encoding="utf-8")

    index_data = {"count": len(entries), "playlists": ["playlists_tree.json"], "tracks": entries}
    (output_dir / "index.json").write_text(json.dumps(index_data, indent=2) + "\n", encoding="utf-8")
    return index_data


def main():
    parser = argparse.ArgumentParser(description="Gera conjunto de casos sintéticos determinísticos")
    parser.add_argument("--output-dir", default="tests/reference/synthetic", help="Diretório dos JSONs de saída")
    parser.add_argument("--wav-dir", default="tests/reference/synthetic/wavs", help="Diretório onde salvar os WAVs")
    args = parser.parse_args()

    index_data = create_synthetic_suite(Path(args.output_dir), Path(args.wav_dir))
    print(f"Sucesso: {index_data['count']} casos sintéticos gerados em {args.output_dir}")


if __name__ == "__main__":
    main()
