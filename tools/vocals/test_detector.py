import json
import sys
import tempfile
import unittest
from pathlib import Path
import numpy as np
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'res/vocals'))
from vocal_detector import analyze, features, predict, segments
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

if __name__ == '__main__': unittest.main()
