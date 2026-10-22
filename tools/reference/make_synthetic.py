#!/usr/bin/env python3
"""Gera casos sintéticos determinísticos de áudio e seus JSONs de referência.

Gera WAVs com a biblioteca padrão (wave, struct, math):
1. fixed_120bpm: andamento fixo a 120.0 BPM com cliques limpos.
2. tempo_change_120_to_140bpm: transição de andamento (120 BPM -> 140 BPM).
3. silent_intro_125bpm: introdução sem batidas seguida de ritmo a 125.0 BPM.

Os WAVs não são versionados (armazenados em tests/reference/synthetic/wavs/ com .gitignore).
Os JSONs de resultado esperado são versionados em tests/reference/synthetic/.

Uso:
    python3 tools/reference/make_synthetic.py [--output-dir tests/reference/synthetic] [--wav-dir tests/reference/synthetic/wavs]
"""

import argparse
import hashlib
import json
import math
from pathlib import Path
import struct
import wave


def generate_click_sample(t_local: float) -> float:
    """Mesma função de clique dos testes C++ (AnalyzerGridTest, clickTrack)."""
    return math.exp(-t_local * 250.0) * math.cos(t_local * 2.0 * math.pi * 180.0)


def synthesize_fixed_bpm(rate: int, bpm: float, duration: float, first_beat: float) -> bytes:
    total_frames = int(duration * rate)
    beat_interval = 60.0 / bpm
    frames_bytes = bytearray()

    # Gera batidas
    beat_times = []
    t = first_beat
    while t < duration - 0.05:
        beat_times.append(t)
        t += beat_interval

    beat_frames = {int(bt * rate) for bt in beat_times}
    click_len = int(rate * 0.03)

    for i in range(total_frames):
        sample = 0.0
        # Checa se está dentro da janela de algum clique
        for bf in beat_frames:
            if 0 <= i - bf < click_len:
                t_local = (i - bf) / rate
                sample += generate_click_sample(t_local)
        # Clipa e converte para int16
        sample_clamped = max(-1.0, min(1.0, sample))
        int_val = int(sample_clamped * 32767.0)
        frames_bytes.extend(struct.pack("<hh", int_val, int_val))

    return bytes(frames_bytes)


def synthesize_tempo_change(rate: int, duration: float, split_time: float, bpm1: float, bpm2: float) -> bytes:
    total_frames = int(duration * rate)
    frames_bytes = bytearray()

    beat_times = []
    t = 0.5
    while t < split_time:
        beat_times.append(t)
        t += 60.0 / bpm1

    t = split_time + (60.0 / bpm2)
    while t < duration - 0.05:
        beat_times.append(t)
        t += 60.0 / bpm2

    beat_frames = {int(bt * rate) for bt in beat_times}
    click_len = int(rate * 0.03)

    for i in range(total_frames):
        sample = 0.0
        for bf in beat_frames:
            if 0 <= i - bf < click_len:
                t_local = (i - bf) / rate
                sample += generate_click_sample(t_local)
        sample_clamped = max(-1.0, min(1.0, sample))
        int_val = int(sample_clamped * 32767.0)
        frames_bytes.extend(struct.pack("<hh", int_val, int_val))

    return bytes(frames_bytes)


def synthesize_silent_intro(rate: int, duration: float, intro_silence: float, bpm: float) -> bytes:
    total_frames = int(duration * rate)
    frames_bytes = bytearray()

    beat_interval = 60.0 / bpm
    beat_times = []
    t = intro_silence
    while t < duration - 0.05:
        beat_times.append(t)
        t += beat_interval

    beat_frames = {int(bt * rate) for bt in beat_times}
    click_len = int(rate * 0.03)

    for i in range(total_frames):
        sample = 0.0
        if i >= int(intro_silence * rate):
            for bf in beat_frames:
                if 0 <= i - bf < click_len:
                    t_local = (i - bf) / rate
                    sample += generate_click_sample(t_local)
        sample_clamped = max(-1.0, min(1.0, sample))
        int_val = int(sample_clamped * 32767.0)
        frames_bytes.extend(struct.pack("<hh", int_val, int_val))

    return bytes(frames_bytes)


def write_wav(path: Path, raw_pcm: bytes, rate: int = 44100):
    path.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(path), "wb") as w:
        w.setnchannels(2)
        w.setsampwidth(2)
        w.setframerate(rate)
        w.writeframes(raw_pcm)


def sha256_of_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()


def create_synthetic_suite(output_dir: Path, wav_dir: Path) -> dict:
    output_dir = Path(output_dir).resolve()
    wav_dir = Path(wav_dir).resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    wav_dir.mkdir(parents=True, exist_ok=True)

    # Garante que arquivos WAV gerados não sejam versionados
    gitignore_file = wav_dir / ".gitignore"
    if not gitignore_file.exists():
        gitignore_file.write_text("*.wav\n", encoding="utf-8")

    rate = 44100
    cases = []

    # 1. fixed_120bpm
    p1 = wav_dir / "fixed_120bpm.wav"
    pcm1 = synthesize_fixed_bpm(rate, bpm=120.0, duration=24.0, first_beat=0.317)
    write_wav(p1, pcm1, rate)
    hash1 = sha256_of_file(p1)
    case1 = {
        "name": "fixed_120bpm",
        "sha256": hash1,
        "sample_rate": rate,
        "duration": 24.0,
        "bpm": 120.0,
        "tempo_type": "fixed",
        "first_beat_time_s": 0.317,
        "first_beat_frame": int(0.317 * rate),
        "downbeat_index": 0,
        "phrases": [
            {"start": 0.317, "end": 8.317, "bars": 4, "type": "INTRO", "confidence": 1.0},
            {"start": 8.317, "end": 24.0, "bars": 8, "type": "CHORUS", "confidence": 1.0},
        ],
        "cues": [
            {"type": "hotcue", "position_frame": int(0.317 * rate), "hotcue": 0, "label": "MIX IN"},
            {"type": "hotcue", "position_frame": int(8.317 * rate), "hotcue": 1, "label": "DROP"},
        ],
        "has_vocal": False,
    }
    (output_dir / "fixed_120bpm.json").write_text(json.dumps(case1, indent=2), encoding="utf-8")
    cases.append(case1)

    # 2. tempo_change_120_to_140bpm
    p2 = wav_dir / "tempo_change_120_to_140bpm.wav"
    pcm2 = synthesize_tempo_change(rate, duration=32.0, split_time=16.0, bpm1=120.0, bpm2=140.0)
    write_wav(p2, pcm2, rate)
    hash2 = sha256_of_file(p2)
    case2 = {
        "name": "tempo_change_120_to_140bpm",
        "sha256": hash2,
        "sample_rate": rate,
        "duration": 32.0,
        "bpm": 120.0,
        "tempo_type": "variable",
        "sections": [
            {"start": 0.5, "end": 16.0, "bpm": 120.0},
            {"start": 16.0 + (60.0 / 140.0), "end": 32.0, "bpm": 140.0},
        ],
        "first_beat_time_s": 0.5,
        "first_beat_frame": int(0.5 * rate),
        "downbeat_index": 0,
        "has_vocal": False,
    }
    (output_dir / "tempo_change_120_to_140bpm.json").write_text(json.dumps(case2, indent=2), encoding="utf-8")
    cases.append(case2)

    # 3. silent_intro_125bpm
    p3 = wav_dir / "silent_intro_125bpm.wav"
    pcm3 = synthesize_silent_intro(rate, duration=24.0, intro_silence=5.0, bpm=125.0)
    write_wav(p3, pcm3, rate)
    hash3 = sha256_of_file(p3)
    case3 = {
        "name": "silent_intro_125bpm",
        "sha256": hash3,
        "sample_rate": rate,
        "duration": 24.0,
        "bpm": 125.0,
        "tempo_type": "fixed",
        "intro_silence_s": 5.0,
        "first_beat_time_s": 5.0,
        "first_beat_frame": int(5.0 * rate),
        "downbeat_index": 0,
        "has_vocal": False,
    }
    (output_dir / "silent_intro_125bpm.json").write_text(json.dumps(case3, indent=2), encoding="utf-8")
    cases.append(case3)

    index_data = {
        "count": len(cases),
        "cases": [
            {
                "name": c["name"],
                "sha256": c["sha256"],
                "duration": c["duration"],
                "bpm": c["bpm"],
                "tempo_type": c["tempo_type"],
            }
            for c in cases
        ],
    }
    (output_dir / "index.json").write_text(json.dumps(index_data, indent=2), encoding="utf-8")
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
