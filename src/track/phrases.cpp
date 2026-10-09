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
    QVector<int> candidates{0};
    for (int i = 4; i + 4 <= bars.size(); i += 4) if (novelty[i] >= 0.12) candidates.push_back(i);
    candidates.push_back(bars.size());
    QVector<double> cost(candidates.size(), -std::numeric_limits<double>::infinity());
    QVector<int> previous(candidates.size(), -1);
    cost[0] = 0;
    for (int j = 1; j < candidates.size(); ++j) for (int i = 0; i < j; ++i) {
        const int length = candidates[j] - candidates[i];
        if (length < 4 && !(i == 0 && j == candidates.size() - 1)) continue;
        const double reward = j + 1 == candidates.size() ? 0 :
                novelty[candidates[j]] - 0.15 + (candidates[j] % 8 == 0 ? 0.025 : 0);
        if (cost[i] + reward > cost[j]) { cost[j] = cost[i] + reward; previous[j] = i; }
    }
    QVector<int> boundaries;
    for (int j = candidates.size() - 1; j >= 0; j = previous[j]) {
        boundaries.prepend(candidates[j]);
        if (j == 0) break;
    }
    if (boundaries.size() < 2) return result;
    QVector<double> energies, basses;
    for (int j = 0; j + 1 < boundaries.size(); ++j) {
        int start = boundaries[j], end = boundaries[j + 1];
        double energy = 0, bass = 0;
        for (int k = start; k < end; ++k) { energy += bars[k].rms; bass += bars[k].bass; }
        energies.push_back(energy / (end - start) / maxEnergy);
        basses.push_back(bass / (end - start) / maxBass);
        result.phrases.push_back({j == 0 ? 0 : bars[start].start, bars[end - 1].end,
                end - start, PhraseType::Verse, result.lowConfidence ? 0.4 : 0.65});
    }
    QVector<double> sortedBass;
    for (const auto& bar : bars) sortedBass.push_back(bar.bass / maxBass);
    std::sort(sortedBass.begin(), sortedBass.end());
    const double medianBass = (sortedBass[(sortedBass.size() - 1) / 2] + sortedBass[sortedBass.size() / 2]) * 0.5;
    for (int j = 0; j < result.phrases.size(); ++j) {
        auto& phrase = result.phrases[j];
        bool vocal = false;
        for (const auto& segment : vocals) if (segment.first < phrase.end && segment.second > phrase.start) { vocal = true; break; }
        if (energies[j] > 0.65 && basses[j] > 0.65) {
            phrase.type = PhraseType::Chorus;
            phrase.confidence = result.lowConfidence ? 0.45 : (vocal ? 0.9 : 0.8);
        } else if (j == 0 && basses[j] < medianBass - 1e-8) phrase.type = PhraseType::Intro;
        else if (j == result.phrases.size() - 1 && basses[j] < medianBass - 1e-8) phrase.type = PhraseType::Outro;
        else if (j > 0 && basses[j] < basses[j - 1] * 0.55) phrase.type = PhraseType::Down;
        else if (j + 1 < energies.size() && energies[j + 1] > 0.65 && basses[j + 1] > 0.65 && energies[j + 1] > energies[j] * 1.25) phrase.type = PhraseType::Up;
        if (phrase.confidence < 0.5) phrase.type = PhraseType::Unknown;
    }
    return result;
}
} // namespace vilelabox
