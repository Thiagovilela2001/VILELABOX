#pragma once
#include <vector>
#include "analyzer/analyzer.h"
#include "preferences/usersettings.h"
#include "track/phrases.h"

class AnalyzerPhrases : public Analyzer {
  public:
    explicit AnalyzerPhrases(UserSettingsPointer config);
    bool initialize(const AnalyzerTrack& track, mixxx::audio::SampleRate rate, SINT length) override;
    bool processSamples(const CSAMPLE* samples, SINT count) override;
    void storeResults(TrackPointer track) override;
    void cleanup() override;
  private:
    void finishBlock();
    UserSettingsPointer m_config;
    int m_rate = 0;
    qint64 m_frames = 0;
    QString m_cache;
    std::vector<double> m_block;
    std::vector<double> m_previousSpectrum;
    QVector<vilelabox::PhraseFeature> m_features;
};
