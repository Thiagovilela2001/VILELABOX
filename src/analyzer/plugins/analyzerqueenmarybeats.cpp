#include <dsp/onsets/DetectionFunction.h>
#include <dsp/tempotracking/DownBeat.h>
#include <algorithm>
#include <cmath>
#include <dsp/tempotracking/TempoTrackV2.h>

// Class header comes after library includes here since our preprocessor
// definitions interfere with qm-dsp's headers.
#include "analyzer/plugins/analyzerqueenmarybeats.h"

#include "analyzer/constants.h"

namespace mixxx {
namespace {

// This determines the resolution of the resulting BeatMap.
// ~12 ms (86 Hz) is a fair compromise between accuracy and analysis speed,
// also matching the preferred window/step sizes from BeatTrack VAMP.
// For a 44.1 kHz track, we go in 512 sample steps
// TODO: kStepSecs and the waveform sample rate of 441
// (defined in AnalyzerWaveform::initialize) do not align well and thus
// generate interference. Currently we are at this odd factor: 441 * 0.01161 = 5.12.
// This should be adjusted to be an integer.
constexpr float kStepSecs = 0.01161f;
// results in 43 Hz @ 44.1 kHz / 47 Hz @ 48 kHz / 47 Hz @ 96 kHz
constexpr int kMaximumBinSizeHz = 50; // Hz

DFConfig makeDetectionFunctionConfig(int stepSizeFrames, int windowSize) {
    // These are the defaults for the VAMP beat tracker plugin we used in Mixxx
    // 2.0.
    DFConfig config;
    config.DFType = DF_COMPLEXSD;
    config.stepSize = stepSizeFrames;
    config.frameLength = windowSize;
    config.dbRise = 3;
    config.adaptiveWhitening = false;
    config.whiteningRelaxCoeff = -1;
    config.whiteningFloor = -1;
    return config;
}

} // namespace

AnalyzerQueenMaryBeats::AnalyzerQueenMaryBeats(bool precision)
        : m_precision(precision),
          m_windowSize(0),
          m_stepSizeFrames(0) {
}

AnalyzerQueenMaryBeats::~AnalyzerQueenMaryBeats() {
}

bool AnalyzerQueenMaryBeats::initialize(mixxx::audio::SampleRate sampleRate) {
    m_detectionResults.clear();
    m_resultBeats.clear();
    m_resultDownbeat = {};
    m_transients.clear();
    m_audioBlockFill = 0;
    m_envelopeFill = 0;
    m_envelopeEnergy = 0;
    m_previousEnergy = 0;
    m_envelopeStep = std::max(1, static_cast<int>(sampleRate / 1000));
    if (m_precision) {
        size_t factor = 16;
        while (sampleRate / factor > 4000 && factor < 64) {
            factor *= 2;
        }
        m_pDownbeat = std::make_unique<DownBeat>(sampleRate, factor, m_audioBlock.size());
        m_pDownbeat->setBeatsPerBar(4);
    }
    m_sampleRate = sampleRate;
    m_stepSizeFrames = static_cast<int>(m_sampleRate * kStepSecs);
    m_windowSize = MathUtilities::nextPowerOfTwo(m_sampleRate / kMaximumBinSizeHz);
    m_pDetectionFunction = std::make_unique<DetectionFunction>(
            makeDetectionFunctionConfig(m_stepSizeFrames, m_windowSize));
    qDebug() << "input sample rate is " << m_sampleRate << ", step size is " << m_stepSizeFrames;

    m_helper.initialize(
            m_windowSize, m_stepSizeFrames, [this](double* pWindow, size_t) {
                // TODO(rryan) reserve?
                m_detectionResults.push_back(
                        m_pDetectionFunction->processTimeDomain(pWindow));
                return true;
            });
    return true;
}

bool AnalyzerQueenMaryBeats::processSamples(const CSAMPLE* pIn, SINT iLen) {
    DEBUG_ASSERT(iLen % kAnalysisChannels == 0);
    if (!m_pDetectionFunction) {
        return false;
    }

    if (m_precision) {
        for (SINT i = 0; i < iLen; i += kAnalysisChannels) {
            const float mono = (pIn[i] + pIn[i + 1]) * 0.5f;
            m_audioBlock[m_audioBlockFill++] = mono;
            if (m_audioBlockFill == m_audioBlock.size()) {
                m_pDownbeat->pushAudioBlock(m_audioBlock.data());
                m_audioBlockFill = 0;
            }
            m_envelopeEnergy += mono * mono;
            if (++m_envelopeFill == m_envelopeStep) {
                const double energy = m_envelopeEnergy / m_envelopeStep;
                m_transients.push_back(std::max(0.0, energy - m_previousEnergy));
                m_previousEnergy = energy;
                m_envelopeFill = 0;
                m_envelopeEnergy = 0;
            }
        }
    }
    return m_helper.processStereoSamples(pIn, iLen);
}

bool AnalyzerQueenMaryBeats::finalize() {
    m_helper.finalize();

    int nonZeroCount = static_cast<int>(m_detectionResults.size());
    while (nonZeroCount > 0 && m_detectionResults.at(nonZeroCount - 1) <= 0.0) {
        --nonZeroCount;
    }

    std::vector<double> df;
    std::vector<double> beatPeriod;
    std::vector<double> tempi;
    const auto required_size = std::max(0, nonZeroCount - 2);
    df.reserve(required_size);
    beatPeriod.reserve(required_size);

    // skip first 2 results as it might have detect noise as onset
    // that's how vamp does and seems works best this way
    for (int i = 2; i < nonZeroCount; ++i) {
        df.push_back(m_detectionResults.at(i));
        beatPeriod.push_back(0.0);
    }

    if (m_precision && (df.size() < 512 ||
            std::none_of(df.begin(), df.end(), [](double value) { return value > 1e-10; }))) {
        m_pDetectionFunction.reset();
        m_pDownbeat.reset();
        return true;
    }
    TempoTrackV2 tt(m_sampleRate, m_stepSizeFrames);
    tt.calculateBeatPeriod(df, beatPeriod, tempi);

    std::vector<double> beats;
    tt.calculateBeats(df, beatPeriod, beats);

    m_resultBeats.reserve(static_cast<int>(beats.size()));
    for (size_t i = 0; i < beats.size(); ++i) {
        // we add the halve m_stepSizeFrames here, because the beat
        // is detected between the two samples.
        const auto result = mixxx::audio::FramePos(
                (beats.at(i) * m_stepSizeFrames) + m_stepSizeFrames / 2);
        m_resultBeats.push_back(result);
    }

    if (m_precision) {
        refineBeats();
    }
    m_pDetectionFunction.reset();
    m_pDownbeat.reset();
    return true;
}

void AnalyzerQueenMaryBeats::refineBeats() {
    for (auto& beat : m_resultBeats) {
        const double center = beat.value() / m_envelopeStep;
        const int radius = std::max(1, static_cast<int>(m_sampleRate * 0.025 / m_envelopeStep));
        const int first = std::max(0, static_cast<int>(center) - radius);
        const int last = std::min(static_cast<int>(m_transients.size()) - 1,
                static_cast<int>(center) + radius);
        double bestScore = 1e-10;
        int best = -1;
        for (int bin = first; bin <= last; ++bin) {
            const double score = m_transients[bin] / (1.0 + std::abs(bin - center) / 10.0);
            if (score > bestScore) {
                bestScore = score;
                best = bin;
            }
        }
        if (best >= 0) {
            beat = audio::FramePos(static_cast<double>(best) * m_envelopeStep);
        }
    }
    if (m_resultBeats.size() < 16 || !m_pDownbeat) {
        return;
    }
    if (m_audioBlockFill) {
        std::fill(m_audioBlock.begin() + m_audioBlockFill, m_audioBlock.end(), 0.0f);
        m_pDownbeat->pushAudioBlock(m_audioBlock.data());
    }
    std::vector<double> beats;
    for (const auto& beat : m_resultBeats) {
        beats.push_back(beat.value() / m_audioBlock.size());
    }
    size_t length = 0;
    const float* audio = m_pDownbeat->getBufferedAudio(length);
    std::vector<int> downbeats;
    m_pDownbeat->findDownBeats(audio, length, beats, downbeats);
    std::vector<double> differences;
    m_pDownbeat->getBeatSD(differences);
    std::array<double, 4> scores{};
    std::array<int, 4> counts{};
    for (size_t i = 0; i < differences.size(); ++i) {
        if (std::isfinite(differences[i])) {
            scores[(i + 1) % 4] += differences[i];
            ++counts[(i + 1) % 4];
        }
    }
    for (size_t phase = 0; phase < scores.size(); ++phase) {
        if (counts[phase]) {
            scores[phase] /= counts[phase];
        }
    }
    std::sort(scores.begin(), scores.end());
    if (!downbeats.empty() && scores[3] > 1e-6 && scores[3] > scores[2] * 1.1) {
        m_resultDownbeat = m_resultBeats[downbeats.front()];
    }
}

} // namespace mixxx
