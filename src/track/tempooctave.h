#pragma once
#include <QVector>
#include <algorithm>
#include <cmath>
#include "audio/frame.h"
#include "audio/types.h"

namespace vilelabox {
/// Moves a detected beat list into the preferred BPM range (as rekordbox's
/// analysis range does): below `minBpm` the beats are doubled (a beat in the
/// middle of each pair), above `maxBpm` every other beat is dropped. When
/// halving, the half that contains `downbeat` is kept, so bar 1 survives.
inline QVector<mixxx::audio::FramePos> normalizeTempoOctave(QVector<mixxx::audio::FramePos> beats,
        mixxx::audio::SampleRate rate, double minBpm, double maxBpm,
        mixxx::audio::FramePos downbeat = mixxx::audio::kInvalidFramePos) {
    if (beats.size() < 4 || !rate.isValid() || minBpm <= 0 || maxBpm < minBpm * 1.9) return beats;
    for (int pass = 0; pass < 2; ++pass) {
        QVector<double> intervals;
        for (int i = 1; i < beats.size(); ++i) intervals.push_back(beats[i] - beats[i - 1]);
        std::nth_element(intervals.begin(), intervals.begin() + intervals.size() / 2, intervals.end());
        const double median = intervals[intervals.size() / 2];
        if (median <= 0) return beats;
        const double bpm = 60.0 * rate / median;
        if (bpm < minBpm) {
            QVector<mixxx::audio::FramePos> doubled;
            doubled.reserve(beats.size() * 2);
            for (int i = 0; i < beats.size(); ++i) {
                doubled.push_back(beats[i]);
                if (i + 1 < beats.size()) doubled.push_back(beats[i] + (beats[i + 1] - beats[i]) / 2);
            }
            beats = doubled;
        } else if (bpm > maxBpm) {
            int parity = 0;
            if (downbeat.isValid()) {
                const auto closest = std::min_element(beats.cbegin(), beats.cend(),
                        [downbeat](auto a, auto b) { return std::abs(a - downbeat) < std::abs(b - downbeat); });
                parity = static_cast<int>(closest - beats.cbegin()) % 2;
            }
            QVector<mixxx::audio::FramePos> halved;
            for (int i = parity; i < beats.size(); i += 2) halved.push_back(beats[i]);
            beats = halved;
        } else {
            break;
        }
    }
    return beats;
}
} // namespace vilelabox
