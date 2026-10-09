#pragma once
#include <ebur128.h>
#include "analyzer/analyzer.h"
#include "preferences/usersettings.h"
class AnalyzerEnergy : public Analyzer {
  public:
    explicit AnalyzerEnergy(UserSettingsPointer config) : m_config(config) {}
    ~AnalyzerEnergy() override { cleanup(); }
    bool initialize(const AnalyzerTrack& track, mixxx::audio::SampleRate rate, SINT length) override;
    bool processSamples(const CSAMPLE* samples, SINT count) override;
    void storeResults(TrackPointer track) override;
    void cleanup() override;
  private:
    UserSettingsPointer m_config;
    ebur128_state* m_state = nullptr;
    int m_rate = 0, m_blockFrames = 0;
    qint64 m_frames = 0, m_attacks = 0;
    double m_low = 0, m_bass = 0, m_total = 0, m_blockEnergy = 0, m_previousEnergy = 0;
};
