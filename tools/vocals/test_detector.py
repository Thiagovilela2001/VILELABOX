import json
import sys
import tempfile
import unittest
from pathlib import Path
import numpy as np
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'res/vocals'))
from vocal_detector import analyze, features, predict, segments
import vocal_detector
ROOT = Path(__file__).resolve().parents[2]

class DetectorTests(unittest.TestCase):
    def test_segment_boundaries_and_silence_gaps(self):
        p = np.zeros(64); p[8:20] = 1; p[32:48] = 1
        detected = segments(p, 64 * .256, .9)
        self.assertEqual(len(detected), 2)
        self.assertLess(abs(detected[0][0] - 8 * .256), .52)
        self.assertLess(abs(detected[0][1] - 20 * .256), .52)
        self.assertLess(detected[0][1], detected[1][0])
        self.assertEqual(segments(np.zeros(30), 7.68), [])

    def test_silent_audio_at_both_deck_sample_rates(self):
        for rate in [44100, 48000]:
            with tempfile.NamedTemporaryFile() as raw:
                np.zeros(rate * 2, dtype='<f4').tofile(raw.name)
                output = analyze(raw.name, rate, ROOT/'res/vocals/model.json')
                self.assertEqual(output['duration'], 2)
                self.assertEqual(output['segments'], [])

    def test_exported_model_matches_training_predictions(self):
        data = np.load(ROOT/'tools/vocals/validation_features.npz')
        model = json.loads((ROOT/'res/vocals/model.json').read_text())
        np.testing.assert_allclose(predict(data['X'], model), data['p'], atol=3e-6)
        self.assertTrue(np.isfinite(features(np.zeros(16384))).all())

class DemucsEngineTests(unittest.TestCase):
    def test_frames_merge_word_gaps_and_drop_blips(self):
        frame = vocal_detector.FRAME
        active = [False] * 200
        for i in list(range(20, 60)) + list(range(70, 110)):  # 0.5 s pause between
            active[i] = True
        active[150] = True  # a single 50 ms blip
        result = vocal_detector.frames_to_segments(active, 200 * frame)
        self.assertEqual(result, [[round(20 * frame, 4), round(110 * frame, 4)]])

    @unittest.skipUnless(vocal_detector.demucs_available(), 'demucs not installed')
    def test_no_vocals_in_silence_or_a_steady_tone(self):
        rate = 44100
        t = np.arange(rate * 8) / rate
        signal = np.zeros_like(t)
        signal[rate * 4:] = 0.3 * np.sin(2 * np.pi * 110 * t[rate * 4:])  # bass-like tone
        with tempfile.NamedTemporaryFile(suffix='.f32') as raw:
            signal.astype('<f4').tofile(raw.name)
            result = vocal_detector.analyze_demucs(raw.name, rate)
        self.assertEqual(result['engine'], 'demucs')
        self.assertAlmostEqual(result['duration'], 8.0, places=3)
        self.assertEqual(result['segments'], [])


if __name__ == '__main__': unittest.main()
