#include "analyzer/analyzerphrases.h"
#include <algorithm>
#include <cmath>
#include <complex>
#include "analyzer/phraseanalysisstore.h"
#include "track/track.h"

namespace {
constexpr double kPi = 3.14159265358979323846;
// Iterative radix-2 FFT; no external or proprietary models.
void fft(std::vector<std::complex<double>>& data) {
    const size_t n = data.size();
    for (size_t i = 1, j = 0; i < n; ++i) {
        size_t bit = n >> 1;
        for (; j & bit; bit >>= 1) j ^= bit;
        j ^= bit;
        if (i < j) std::swap(data[i], data[j]);
    }
    for (size_t length = 2; length <= n; length <<= 1) {
        const auto step = std::polar(1.0, -2 * kPi / length);
        for (size_t i = 0; i < n; i += length) {
            std::complex<double> phase(1, 0);
            for (size_t j = 0; j < length / 2; ++j) {
                const auto even = data[i + j], odd = data[i + j + length / 2] * phase;
                data[i + j] = even + odd; data[i + j + length / 2] = even - odd;
                phase *= step;
            }
        }
    }
}
}
AnalyzerPhrases::AnalyzerPhrases(UserSettingsPointer config) : m_config(std::move(config)) {}
bool AnalyzerPhrases::initialize(const AnalyzerTrack& track, mixxx::audio::SampleRate rate, SINT length) {
    cleanup();
    const bool force = track.getOptions().forcePhrases.value_or(false);
    if ((!force && !m_config->getValue(ConfigKey("[Analyzer]", "AnalyzePhrases"), false)) ||
            !rate.isValid() || length <= 0) return false;
    m_rate = static_cast<int>(rate);
    m_cache = PhraseAnalysisStore::cachePath(m_config->getSettingsPath() + "/analysis/phrases", track.getTrack()->getLocation());
    vilelabox::PhraseAnalysis cached;
    if (!force && PhraseAnalysisStore::load(m_cache, &cached)) {
        PhraseAnalysisStore::instance().set(track.getTrack(), std::move(cached), m_cache);
        return false;
    }
    return true;
}
bool AnalyzerPhrases::processSamples(const CSAMPLE* samples, SINT count) {
    if (m_rate <= 0 || count < 0 || count % 2) return false;
    for (SINT i = 0; i < count; i += 2) {
        const double mono = (samples[i] + samples[i + 1]) * 0.5;
        if (!std::isfinite(mono)) return false;
        m_block.push_back(mono);
        ++m_frames;
        if (m_block.size() >= static_cast<size_t>(m_rate / 20)) finishBlock();
    }
    return true;
}
void AnalyzerPhrases::finishBlock() {
    if (m_block.empty()) return;
    vilelabox::PhraseFeature feature;
    feature.start = static_cast<double>(m_frames - m_block.size()) / m_rate;
    feature.end = static_cast<double>(m_frames) / m_rate;
    size_t size = 1;
    while (size < m_block.size()) size <<= 1;
    std::vector<std::complex<double>> spectrum(size);
    for (size_t i = 0; i < m_block.size(); ++i) {
        feature.rms += m_block[i] * m_block[i];
        spectrum[i] = m_block[i] * (0.5 - 0.5 * std::cos(2 * kPi * i / std::max(size_t(1), m_block.size() - 1)));
    }
    feature.rms = std::sqrt(feature.rms / m_block.size());
    fft(spectrum);
    if (m_previousSpectrum.size() != size / 2) m_previousSpectrum.assign(size / 2, 0);
    double total = 0;
    for (size_t i = 1; i < size / 2; ++i) {
        const double frequency = static_cast<double>(i) * m_rate / size;
        const double power = std::norm(spectrum[i]) / (size * m_block.size());
        total += power;
        if (frequency < 150) feature.bass += power;
        feature.flux += std::max(0.0, std::sqrt(power) - m_previousSpectrum[i]);
        m_previousSpectrum[i] = std::sqrt(power);
        int band = frequency < 150 ? 0 : frequency < 400 ? 1 : frequency < 1000 ? 2 : frequency < 2500 ? 3 : frequency < 6000 ? 4 : 5;
        feature.timbre[band] += power;
        if (frequency >= 55 && frequency < 5000) {
            int midi = static_cast<int>(std::lround(69 + 12 * std::log2(frequency / 440)));
            feature.chroma[((midi % 12) + 12) % 12] += power;
        }
    }
    feature.bass = std::sqrt(feature.bass);
    double chromaTotal = 0;
    for (auto v : feature.chroma) chromaTotal += v;
    if (chromaTotal > 1e-12) for (auto& v : feature.chroma) v /= chromaTotal;
    if (total > 1e-12) for (auto& v : feature.timbre) v /= total;
    m_features.push_back(feature);
    m_block.clear();
}
void AnalyzerPhrases::storeResults(TrackPointer track) {
    finishBlock();
    vilelabox::PhraseAnalysis result;
    result.features = std::move(m_features);
    result.duration = static_cast<double>(m_frames) / m_rate;
    PhraseAnalysisStore::instance().set(track, std::move(result), m_cache, true);
}
void AnalyzerPhrases::cleanup() {
    m_rate = 0; m_frames = 0; m_cache.clear(); m_block.clear(); m_previousSpectrum.clear(); m_features.clear();
}
