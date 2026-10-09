#include "analyzer/analyzerenergy.h"
#include <algorithm>
#include <cmath>
#include "library/related/relatedtracks.h"
#include "track/track.h"
bool AnalyzerEnergy::initialize(const AnalyzerTrack& track, mixxx::audio::SampleRate rate, SINT length) {
    cleanup();
    if (!rate.isValid() || length <= 0 || !m_config->getValue(ConfigKey("[Analyzer]", "AnalyzeRelatedTracks"), false)) return false;
    if (vilelabox::loadEnergies(m_config->getSettingsPath()).contains(vilelabox::energyIdentity(track.getTrack()->getLocation()))) return false;
    m_rate = static_cast<int>(rate);
    m_state = ebur128_init(2, m_rate, EBUR128_MODE_I);
    return m_state != nullptr;
}
bool AnalyzerEnergy::processSamples(const CSAMPLE* samples, SINT count) {
    if (!m_state || count < 0 || count % 2) return false;
    if (ebur128_add_frames_float(m_state, samples, count / 2) != EBUR128_SUCCESS) return false;
    const double alpha = 1 - std::exp(-2 * 3.141592653589793 * 150 / m_rate);
    for (SINT i = 0; i < count; i += 2) {
        const double sample = (samples[i] + samples[i + 1]) * 0.5;
        if (!std::isfinite(sample)) return false;
        m_low += alpha * (sample - m_low);
        m_bass += m_low * m_low; m_total += sample * sample; m_blockEnergy += sample * sample;
        ++m_frames;
        if (++m_blockFrames >= m_rate / 20) {
            const double energy = m_blockEnergy / m_blockFrames;
            if (energy > 1e-7 && energy > m_previousEnergy * 1.5) ++m_attacks;
            m_previousEnergy = energy; m_blockFrames = 0; m_blockEnergy = 0;
        }
    }
    return true;
}
void AnalyzerEnergy::storeResults(TrackPointer track) {
    double lufs = 0;
    if (!m_state || m_frames <= 0 || ebur128_loudness_global(m_state, &lufs) != EBUR128_SUCCESS) return;
    const double loudness = std::isfinite(lufs) ? std::clamp((lufs + 36) / 30, 0.0, 1.0) : 0;
    const double attacks = std::clamp(m_attacks / (static_cast<double>(m_frames) / m_rate) / 8, 0.0, 1.0);
    const double bass = m_total > 1e-12 ? std::clamp(m_bass / m_total, 0.0, 1.0) : 0;
    vilelabox::saveEnergy(m_config->getSettingsPath(), track->getLocation(), 1 + 9 * (0.6 * loudness + 0.25 * attacks + 0.15 * bass));
}
void AnalyzerEnergy::cleanup() {
    if (m_state) ebur128_destroy(&m_state);
    m_rate = 0; m_blockFrames = 0; m_frames = 0; m_attacks = 0;
    m_low = 0; m_bass = 0; m_total = 0; m_blockEnergy = 0; m_previousEnergy = 0;
}
