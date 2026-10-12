#!/usr/bin/env python3
"""Local vocal detector. No audio or analysis is sent over the network.

Two engines:
- demucs (preferred): separates the vocals with the open Demucs model
  (htdemucs, MIT licence, runs locally) and marks the frames where the isolated
  vocal is audible. Used when the `demucs` package is installed.
- classifier (fallback): the small spectral network in model.json.
"""
import argparse
import importlib.util
import json
import math
import os
from pathlib import Path

os.environ.setdefault('OPENBLAS_NUM_THREADS', '1')
os.environ.setdefault('OMP_NUM_THREADS', '1')
import numpy as np
from scipy.signal import resample_poly

RATE = 16000
WINDOW = 16384
STEP = 4096
FFT = 512
HOP = 256


def features(audio):
    audio = np.asarray(audio, dtype=np.float32)
    if len(audio) < WINDOW:
        audio = np.pad(audio, (0, WINDOW - len(audio)))
    audio = audio[:WINDOW]
    rms = np.sqrt(np.mean(audio * audio))
    audio = audio / max(float(rms), 1e-5)
    frames = np.lib.stride_tricks.sliding_window_view(audio, FFT)[::HOP]
    spectrum = np.abs(np.fft.rfft(frames * np.hanning(FFT), axis=1)) ** 2
    mel = spectrum @ mel_filters()
    logmel = np.log(np.maximum(mel, 1e-10))
    logmel -= logmel.mean()
    delta = np.diff(logmel, axis=0)
    flatness = np.exp(np.log(np.maximum(spectrum, 1e-12)).mean(axis=1)) / np.maximum(spectrum.mean(axis=1), 1e-12)
    centroid = (spectrum * np.linspace(0, 1, spectrum.shape[1])).sum(axis=1) / np.maximum(spectrum.sum(axis=1), 1e-12)
    zcr = (frames[:, 1:] * frames[:, :-1] < 0).mean(axis=1)
    return np.concatenate((logmel.mean(axis=0), logmel.std(axis=0), delta.std(axis=0),
        [flatness.mean(), flatness.std(), centroid.mean(), centroid.std(), zcr.mean(), zcr.std()])).astype(np.float32)


_FILTERS = None

def mel_filters():
    global _FILTERS
    if _FILTERS is None:
        mel = np.linspace(2595 * np.log10(1 + 40 / 700), 2595 * np.log10(1 + 7600 / 700), 42)
        hz = 700 * (10 ** (mel / 2595) - 1)
        freq = np.fft.rfftfreq(FFT, 1 / RATE)
        _FILTERS = np.maximum(0, np.minimum((freq[:, None] - hz[:-2]) / (hz[1:-1] - hz[:-2]),
            (hz[2:] - freq[:, None]) / (hz[2:] - hz[1:-1]))).astype(np.float32)
    return _FILTERS


def predict(x, model):
    x = (np.asarray(x) - np.asarray(model['mean'])) / np.asarray(model['scale'])
    for i, (weight, bias) in enumerate(zip(model['weights'], model['biases'])):
        x = x @ np.asarray(weight) + np.asarray(bias)
        x = np.maximum(0, x) if i + 1 < len(model['weights']) else 1 / (1 + np.exp(-np.clip(x, -40, 40)))
    return x.reshape(-1)


def segments(probabilities, duration, threshold=0.65):
    # Three predictions suppress isolated attacks, while allowing short phrases.
    smoothed = np.convolve(probabilities, np.ones(3) / 3, mode='same') if len(probabilities) >= 3 else probabilities
    result = []
    start = None
    for i, p in enumerate(smoothed):
        time = i * STEP / RATE
        if p >= threshold and start is None:
            start = max(0, time - STEP / RATE)
        elif p < threshold - 0.15 and start is not None:
            end = min(duration, time + STEP / RATE)
            if end - start >= 0.4:
                result.append([round(start, 4), round(end, 4)])
            start = None
    if start is not None and duration - start >= 0.4:
        result.append([round(start, 4), round(duration, 4)])
    merged = []
    for start, end in result:
        if merged and start - merged[-1][1] <= 0.4:
            merged[-1][1] = end
        else:
            merged.append([start, end])
    return merged


def analyze(raw, sample_rate, model_path):
    model = json.loads(Path(model_path).read_text())
    source = np.memmap(raw, dtype='<f4', mode='r')
    duration = len(source) / sample_rate
    probabilities = []
    step_seconds = STEP / RATE
    # Bounded memory; each block includes context for all centered windows.
    for first in range(0, max(1, math.ceil(duration / step_seconds)), 128):
        count = min(128, math.ceil(duration / step_seconds) - first)
        start_seconds = first * step_seconds - WINDOW / RATE / 2
        end_seconds = (first + count - 1) * step_seconds + WINDOW / RATE / 2
        a = math.floor(start_seconds * sample_rate)
        b = math.ceil(end_seconds * sample_rate)
        clip = np.asarray(source[max(0, a):min(len(source), b)])
        clip = np.pad(clip, (max(0, -a), max(0, b - len(source))))
        divisor = math.gcd(sample_rate, RATE)
        clip = resample_poly(clip, RATE // divisor, sample_rate // divisor)
        batch = []
        silent = []
        for index in range(count):
            window = clip[index * STEP:index * STEP + WINDOW]
            silent.append(np.sqrt(np.mean(window * window)) < 0.001)
            batch.append(features(window))
        output = predict(batch, model)
        output[np.asarray(silent)] = 0
        probabilities.extend(output.tolist())
    return {'version': model['version'], 'duration': duration,
        'step': step_seconds, 'segments': segments(np.asarray(probabilities), duration, model['threshold'])}


# --- Demucs engine ---------------------------------------------------------
DEMUCS_MODEL = 'htdemucs'
FRAME = 0.05            # seconds per decision frame
VOICE_DB = -42.0        # isolated vocal level (dBFS) that counts as audible
RELATIVE_DB = -24.0     # ...and not buried this far under the full mix
MERGE_GAP = 0.8         # pauses between words up to this long stay one segment
MIN_SEGMENT = 0.4
CHUNK = 30.0            # seconds separated at a time (bounded memory)
CONTEXT = 2.0           # extra audio on each side of a chunk


def demucs_available():
    return importlib.util.find_spec('demucs') is not None and importlib.util.find_spec('torch') is not None


def engine():
    return 'demucs' if demucs_available() else 'classifier'


def frames_to_segments(active, duration):
    result = []
    start = None
    for i, on in enumerate(active):
        time = i * FRAME
        if on and start is None:
            start = time
        elif not on and start is not None:
            result.append([start, time])
            start = None
    if start is not None:
        result.append([start, len(active) * FRAME])
    merged = []
    for start, end in result:
        if merged and start - merged[-1][1] <= MERGE_GAP:
            merged[-1][1] = end
        else:
            merged.append([start, end])
    return [[round(s, 4), round(min(e, duration), 4)] for s, e in merged
            if min(e, duration) - s >= MIN_SEGMENT and s < duration]


def analyze_demucs(raw, sample_rate):
    import torch
    from demucs.apply import apply_model
    from demucs.pretrained import get_model
    threads = int(os.environ.get('VILELABOX_VOCAL_THREADS', '0')) or max(1, (os.cpu_count() or 2) // 2)
    torch.set_num_threads(threads)
    model = get_model(DEMUCS_MODEL)
    model.eval()
    vocal_index = model.sources.index('vocals')
    source = np.memmap(raw, dtype='<f4', mode='r')
    duration = len(source) / sample_rate
    rate = model.samplerate
    divisor = math.gcd(sample_rate, rate)
    hop = int(round(FRAME * rate))
    voice_db, mix_db = [], []
    position = 0.0
    while position < duration:
        a = max(0, int((position - CONTEXT) * sample_rate))
        b = min(len(source), int((position + CHUNK + CONTEXT) * sample_rate))
        clip = np.asarray(source[a:b], dtype=np.float32)
        if sample_rate != rate:
            clip = resample_poly(clip, rate // divisor, sample_rate // divisor).astype(np.float32)
        wav = torch.from_numpy(np.stack([clip, clip]))[None]
        with torch.no_grad():
            vocals = apply_model(model, wav, device='cpu', split=True, overlap=0.1,
                                 progress=False)[0, vocal_index].mean(0).numpy()
        # Keep only the chunk itself, not its context.
        lead = int(round((position - a / sample_rate) * rate))
        length = int(round(min(CHUNK, duration - position) * rate))
        vocals = vocals[lead:lead + length]
        mix = clip[lead:lead + length]
        count = len(vocals) // hop
        if count:
            power = lambda x: 10 * np.log10(np.maximum((x[:count * hop].reshape(count, hop) ** 2).mean(1), 1e-12))
            voice_db.append(power(vocals))
            mix_db.append(power(mix))
        position += CHUNK
    voice = np.concatenate(voice_db) if voice_db else np.zeros(0)
    mix = np.concatenate(mix_db) if mix_db else np.zeros(0)
    active = (voice > VOICE_DB) & (voice - mix > RELATIVE_DB)
    # A 250 ms majority vote removes single-frame flicker.
    if len(active) >= 5:
        active = np.convolve(active.astype(float), np.ones(5) / 5, mode='same') >= 0.5
    return {'version': f'demucs-{DEMUCS_MODEL}-1', 'engine': 'demucs', 'duration': duration,
            'step': FRAME, 'segments': frames_to_segments(active, duration)}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--raw')
    parser.add_argument('--sample-rate', type=int)
    parser.add_argument('--model', default=str(Path(__file__).with_name('model.json')))
    parser.add_argument('--engine', choices=('auto', 'demucs', 'classifier'), default='auto')
    parser.add_argument('--probe', action='store_true', help='print the engine that would be used')
    args = parser.parse_args()
    chosen = engine() if args.engine == 'auto' else args.engine
    if args.probe:
        print(chosen)
        return
    if not args.raw or not args.sample_rate:
        parser.error('--raw and --sample-rate are required')
    if chosen == 'demucs':
        result = analyze_demucs(args.raw, args.sample_rate)
    else:
        result = analyze(args.raw, args.sample_rate, args.model)
        result['engine'] = 'classifier'
    print(json.dumps(result, allow_nan=False))

if __name__ == '__main__':
    main()
