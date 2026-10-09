#!/usr/bin/env python3
"""Local vocal classifier. No audio or analysis is sent over the network."""
import argparse
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


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--raw', required=True)
    parser.add_argument('--sample-rate', type=int, required=True)
    parser.add_argument('--model', default=str(Path(__file__).with_name('model.json')))
    args = parser.parse_args()
    print(json.dumps(analyze(args.raw, args.sample_rate, args.model), allow_nan=False))

if __name__ == '__main__':
    main()
