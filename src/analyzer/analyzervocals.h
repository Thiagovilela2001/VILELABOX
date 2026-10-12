#pragma once

#include <QTemporaryFile>
#include <functional>
#include <vector>

#include "analyzer/analyzer.h"
#include "analyzer/vocalanalysisstore.h"
#include "preferences/usersettings.h"

class AnalyzerVocals : public Analyzer {
  public:
    /// `isStopping` is polled while the detector runs, so stopping the
    /// analysis (or quitting) does not wait for the detector's timeout.
    explicit AnalyzerVocals(UserSettingsPointer config, std::function<bool()> isStopping = {});
    /// Interpreter: preference, VILELABOX_VOCAL_PYTHON, ~/.venv/bin/python, python3.
    static QString resolvePython(const UserSettingsPointer& config);
    bool initialize(const AnalyzerTrack& track, mixxx::audio::SampleRate rate, SINT length) override;
    bool processSamples(const CSAMPLE* samples, SINT count) override;
    void storeResults(TrackPointer track) override;
    void cleanup() override;
  private:
    void fail(VocalAnalysis::Error error = VocalAnalysis::Error::None, const QString& detail = {});
    std::function<bool()> m_isStopping;
    UserSettingsPointer m_config;
    std::unique_ptr<QTemporaryFile> m_audio;
    std::vector<float> m_mono;
    QString m_location;
    QString m_cache;
    QString m_script;
    QString m_python;
    int m_rate = 0;
};
