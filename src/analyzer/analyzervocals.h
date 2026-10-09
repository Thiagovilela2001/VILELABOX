#pragma once

#include <QTemporaryFile>
#include <vector>

#include "analyzer/analyzer.h"
#include "preferences/usersettings.h"

class AnalyzerVocals : public Analyzer {
  public:
    explicit AnalyzerVocals(UserSettingsPointer config);
    bool initialize(const AnalyzerTrack& track, mixxx::audio::SampleRate rate, SINT length) override;
    bool processSamples(const CSAMPLE* samples, SINT count) override;
    void storeResults(TrackPointer track) override;
    void cleanup() override;
  private:
    void fail();
    UserSettingsPointer m_config;
    std::unique_ptr<QTemporaryFile> m_audio;
    std::vector<float> m_mono;
    QString m_location;
    QString m_cache;
    QString m_script;
    QString m_python;
    int m_rate = 0;
};
