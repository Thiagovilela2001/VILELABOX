#include "track/phrases.h"
#include <algorithm>
#include <cmath>
#include <limits>

namespace vilelabox {
int PhraseAnalysis::indexAt(double seconds) const {
    auto it = std::upper_bound(phrases.cbegin(), phrases.cend(), seconds,
            [](double time, const Phrase& phrase) { return time < phrase.start; });
    if (it == phrases.cbegin() || seconds >= (it - 1)->end) return -1;
    return static_cast<int>(it - phrases.cbegin() - 1);
}

PhraseAnalysis segmentPhrases(const QVector<PhraseFeature>& features,
        double duration, const mixxx::BeatsPointer& beats,
        const QVector<QPair<double, double>>& vocals) {
    PhraseAnalysis result;
    result.features = features;
    result.duration = duration;
    if (!beats || !std::isfinite(duration) || duration <= 0 || features.isEmpty()) return result;
    const double rate = beats->getSampleRate();
    if (rate <= 0) return result;
    result.lowConfidence = !beats->hasDownbeat();
    auto first = beats->iteratorFrom(mixxx::audio::kStartFramePos);
    // Anchor candidates to bar 1, including grids with a negative first beat.
    if (beats->hasDownbeat()) {
        auto position = beats->barPosition(*first);
        if (!position) return result;
        first += (1 - position->bar) * 4 + (1 - position->beat);
        while (*first < mixxx::audio::kStartFramePos) first += 4;
    }
    QVector<PhraseFeature> bars;
    int featureIndex = 0;
    for (auto it = first; (*it).value() / rate < duration; it += 4) {
        PhraseFeature bar;
        bar.start = std::max(0.0, (*it).value() / rate);
        bar.end = std::min(duration, (*(it + 4)).value() / rate);
        if (bar.end <= bar.start || bars.size() >= 100000) break;
        while (featureIndex < features.size() && features[featureIndex].end <= bar.start) ++featureIndex;
        double weight = 0;
        for (int j = featureIndex; j < features.size() && features[j].start < bar.end; ++j) {
            const auto& f = features[j];
            const double w = std::max(0.0, std::min(bar.end, f.end) - std::max(bar.start, f.start));
            bar.rms += f.rms * w; bar.bass += f.bass * w; bar.flux += f.flux * w;
            for (int k = 0; k < 12; ++k) bar.chroma[k] += f.chroma[k] * w;
            for (int k = 0; k < 6; ++k) bar.timbre[k] += f.timbre[k] * w;
            weight += w;
        }
        if (weight > 0) {
            bar.rms /= weight; bar.bass /= weight; bar.flux /= weight;
            for (auto& v : bar.chroma) v /= weight;
            for (auto& v : bar.timbre) v /= weight;
        }
        bars.push_back(bar);
    }
    if (bars.isEmpty()) return result;
    double maxEnergy = 1e-12, maxBass = 1e-12;
    for (const auto& b : bars) { maxEnergy = std::max(maxEnergy, b.rms); maxBass = std::max(maxBass, b.bass); }
    if (maxEnergy < 1e-8) return result;
    // Local self-similarity entries, evaluated only inside the Foote kernel.
    // This avoids an unbounded quadratic matrix on long tracks.
    auto similarity = [&](int a, int b) {
        double distance = std::abs(bars[a].rms - bars[b].rms) / maxEnergy +
                std::abs(bars[a].bass - bars[b].bass) / maxBass;
        for (int k = 0; k < 12; ++k) distance += std::abs(bars[a].chroma[k] - bars[b].chroma[k]) / 12;
        for (int k = 0; k < 6; ++k) distance += std::abs(bars[a].timbre[k] - bars[b].timbre[k]) / 6;
        return std::exp(-distance * 2);
    };
    QVector<double> novelty(bars.size() + 1, 0);
    for (int boundary = 4; boundary + 4 <= bars.size(); boundary += 4) {
        double score = 0;
        for (int a = -4; a < 4; ++a) for (int b = -4; b < 4; ++b) {
            score += ((a < 0) == (b < 0) ? 1 : -1) * similarity(boundary + a, boundary + b);
        }
        novelty[boundary] = std::max(0.0, score / 32);
    }
    // Vocals starting or stopping are structural changes: they add to the
    // novelty of the bar line where they happen.
    if (!vocals.isEmpty()) {
        QVector<double> vocalBar(bars.size(), 0);
        for (int k = 0; k < bars.size(); ++k) {
            for (const auto& segment : vocals) {
                vocalBar[k] += std::max(0.0, std::min(bars[k].end, segment.second) - std::max(bars[k].start, segment.first));
            }
            vocalBar[k] /= std::max(1e-9, bars[k].end - bars[k].start);
        }
        // Eight bars on each side: pauses between lines are not changes, a verse
        // starting or ending is.
        for (int boundary = 4; boundary + 4 <= bars.size(); boundary += 4) {
            double before = 0, after = 0;
            int nBefore = 0, nAfter = 0;
            for (int k = 0; k < 8; ++k) {
                if (boundary - 1 - k >= 0) { before += vocalBar[boundary - 1 - k]; ++nBefore; }
                if (boundary + k < bars.size()) { after += vocalBar[boundary + k]; ++nAfter; }
            }
            const double change = std::abs(after / std::max(1, nAfter) - before / std::max(1, nBefore));
            novelty[boundary] += change > 0.5 ? 0.4 * change : 0;
        }
    }
    // Candidates: clear changes on 4-bar lines, and every 8-bar line so that
    // very long sections can still be split.
    QVector<int> candidates{0};
    for (int i = 4; i + 4 <= bars.size(); i += 4) if (novelty[i] >= 0.12 || i % 8 == 0) candidates.push_back(i);
    candidates.push_back(bars.size());
    // Sections longer than 32 bars cost a little per extra 8 bars.
    // and sections shorter than 8 bars (half a phrase) cost more.
    const auto lengthCost = [](int length) {
        return 0.05 * std::max(0.0, (length - 32) / 8.0) + (length < 8 ? 0.1 : 0.0);
    };
    QVector<double> cost(candidates.size(), -std::numeric_limits<double>::infinity());
    QVector<int> previous(candidates.size(), -1);
    cost[0] = 0;
    for (int j = 1; j < candidates.size(); ++j) for (int i = 0; i < j; ++i) {
        const int length = candidates[j] - candidates[i];
        if (length < 4 && !(i == 0 && j == candidates.size() - 1)) continue;
        const double reward = (j + 1 == candidates.size() ? 0 :
                novelty[candidates[j]] - 0.15 + (candidates[j] % 8 == 0 ? 0.025 : 0)) - lengthCost(length);
        if (cost[i] + reward > cost[j]) { cost[j] = cost[i] + reward; previous[j] = i; }
    }
    QVector<int> boundaries;
    for (int j = candidates.size() - 1; j >= 0; j = previous[j]) {
        boundaries.prepend(candidates[j]);
        if (j == 0) break;
    }
    if (boundaries.size() < 2) return result;
    // Levels are measured against the track's own spread (10th to 90th
    // percentile of its bars), so a loud, flat master does not turn every
    // phrase into a chorus.
    const auto spread = [&bars](auto value) {
        QVector<double> values;
        for (const auto& bar : bars) values.push_back(value(bar));
        std::sort(values.begin(), values.end());
        const auto at = [&values](double q) { return values[static_cast<int>(q * (values.size() - 1))]; };
        return std::pair<double, double>{at(0.1), std::max(at(0.9), at(0.1) + 1e-9)};
    };
    const auto rmsSpread = spread([](const PhraseFeature& b) { return b.rms; });
    const auto bassSpread = spread([](const PhraseFeature& b) { return b.bass; });
    const auto level = [](double x, std::pair<double, double> range) {
        return std::clamp((x - range.first) / (range.second - range.first), 0.0, 1.0);
    };
    struct Segment { double loud = 0, slope = 0, vocal = 0; };
    QVector<Segment> segments;
    for (int j = 0; j + 1 < boundaries.size(); ++j) {
        const int start = boundaries[j], end = boundaries[j + 1];
        Segment s;
        double firstHalf = 0, secondHalf = 0;
        for (int k = start; k < end; ++k) {
            s.loud += 0.5 * level(bars[k].rms, rmsSpread) + 0.5 * level(bars[k].bass, bassSpread);
            (k < (start + end) / 2 ? firstHalf : secondHalf) += level(bars[k].rms, rmsSpread);
        }
        s.loud /= end - start;
        const int half = std::max(1, (end - start) / 2);
        s.slope = secondHalf / std::max(1, end - start - half) - firstHalf / half;
        const double from = j == 0 ? 0 : bars[start].start, to = bars[end - 1].end;
        for (const auto& segment : vocals) {
            s.vocal += std::max(0.0, std::min(to, segment.second) - std::max(from, segment.first));
        }
        s.vocal /= std::max(1e-9, to - from);
        segments.push_back(s);
        result.phrases.push_back({from, to, end - start, PhraseType::Verse, 0});
    }
    constexpr double kHigh = 0.6, kLow = 0.35;
    bool chorusSeen = false;
    const int last = result.phrases.size() - 1;
    for (int j = 0; j <= last; ++j) {
        auto& phrase = result.phrases[j];
        const auto& s = segments[j];
        const bool high = s.loud >= kHigh;
        const bool low = s.loud <= kLow;
        const bool nextHigh = j < last && segments[j + 1].loud >= kHigh;
        // Distance from the nearest threshold: how clear the decision is.
        double margin = std::min(std::abs(s.loud - kHigh), std::abs(s.loud - kLow));
        if (high) {
            phrase.type = PhraseType::Chorus;
            chorusSeen = true;
        } else if (j == 0 && (s.vocal < 0.4 || phrase.bars <= 16)) {
            // An intro has little singing; a long sung opening is a verse.
            phrase.type = PhraseType::Intro;
        } else if (j == last && (s.vocal < 0.4 || phrase.bars <= 16)) {
            phrase.type = PhraseType::Outro;
        } else if (nextHigh && s.slope > 0.1) {
            phrase.type = PhraseType::Up;
            margin = std::max(margin, std::min(0.3, s.slope));
        } else if (low && chorusSeen) {
            phrase.type = PhraseType::Down;
        } else {
            phrase.type = PhraseType::Verse;
        }
        // A sung verse is more certain than an instrumental middle section.
        if (phrase.type == PhraseType::Verse && s.vocal > 0.5) margin += 0.1;
        phrase.confidence = result.lowConfidence ? 0.4 : std::clamp(0.55 + margin, 0.55, 0.95);
        if (phrase.confidence < 0.5) phrase.type = PhraseType::Unknown;
    }
    // Label hysteresis, keeping every boundary: a short phrase close to the
    // threshold takes the label of the neighbours on both sides.
    for (int j = 1; j < result.phrases.size() - 1; ++j) {
        auto& phrase = result.phrases[j];
        const auto before = result.phrases[j - 1].type, after = result.phrases[j + 1].type;
        if (phrase.bars > 8 || before != after || phrase.type == before) continue;
        const bool nearThreshold = std::abs(segments[j].loud - kHigh) < 0.1;
        if (nearThreshold && (before == PhraseType::Chorus || before == PhraseType::Verse) &&
                (phrase.type == PhraseType::Chorus || phrase.type == PhraseType::Verse)) {
            phrase.type = before;
        }
    }
    const int merged = result.phrases.size() - 1;
    // A build-up split in several phrases: extend UP backwards while the
    // energy keeps rising into it.
    for (int j = merged; j > 0; --j) {
        auto& previous = result.phrases[j - 1];
        if (result.phrases[j].type == PhraseType::Up && previous.type == PhraseType::Verse &&
                segments[j - 1].slope > 0.1 && segments[j - 1].loud < segments[j].loud) {
            previous.type = PhraseType::Up;
        }
    }
    return result;
}
} // namespace vilelabox
