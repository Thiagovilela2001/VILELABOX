#include <gtest/gtest.h>

#include <algorithm>
#include <cmath>
#include <vector>

#include "analyzer/plugins/analyzerqueenmarybeats.h"
#include "track/beatfactory.h"

namespace {
std::vector<CSAMPLE> clickTrack(int rate, double bpm, double duration) {
    std::vector<CSAMPLE> samples(static_cast<size_t>(duration * rate) * 2);
    for (double time = 0.317; time < duration - 0.1; time += 60.0 / bpm) {
        const size_t onset = static_cast<size_t>(time * rate);
        for (size_t i = 0; i < static_cast<size_t>(rate * 0.03); ++i) {
            const double t = static_cast<double>(i) / rate;
            const float value = static_cast<float>(std::exp(-t * 250) *
                    std::cos(t * 2 * 3.141592653589793 * 180));
            samples[(onset + i) * 2] = value;
            samples[(onset + i) * 2 + 1] = value;
        }
    }
    return samples;
}

void feed(mixxx::AnalyzerQueenMaryBeats* analyzer, const std::vector<CSAMPLE>& samples) {
    for (size_t offset = 0; offset < samples.size(); offset += 1478) {
        ASSERT_TRUE(analyzer->processSamples(samples.data() + offset,
                std::min(size_t(1478), samples.size() - offset)));
    }
    ASSERT_TRUE(analyzer->finalize());
}
}

TEST(AnalyzerGridTest, PreciseBeatsFollowAudioAtBothSampleRates) {
    double totalImprovement = 0;
    for (const int rate : {44100, 48000}) {
        for (const double bpm : {90.0, 120.0, 150.0}) {
            auto samples = clickTrack(rate, bpm, 24.0);
            mixxx::AnalyzerQueenMaryBeats analyzer(true);
            ASSERT_TRUE(analyzer.initialize(mixxx::audio::SampleRate(rate)));
            feed(&analyzer, samples);
            const auto beats = analyzer.getBeats();
            ASSERT_GT(beats.size(), 16);
            std::vector<double> errors;
            for (const auto beat : beats) {
                const double time = beat.value() / rate;
                if (time < 2.0 || time > 22.0) {
                    continue;
                }
                const double count = std::round((time - 0.317) * bpm / 60.0);
                errors.push_back(std::abs(time - (0.317 + count * 60.0 / bpm)));
            }
            ASSERT_FALSE(errors.empty());
            std::sort(errors.begin(), errors.end());
            EXPECT_LT(errors[errors.size() / 2], 0.003) << rate << " Hz, " << bpm << " BPM";
            mixxx::AnalyzerQueenMaryBeats legacy;
            ASSERT_TRUE(legacy.initialize(mixxx::audio::SampleRate(rate)));
            feed(&legacy, samples);
            std::vector<double> legacyErrors;
            for (const auto beat : legacy.getBeats()) {
                const double time = beat.value() / rate;
                if (time >= 2.0 && time <= 22.0) {
                    const double count = std::round((time - 0.317) * bpm / 60.0);
                    legacyErrors.push_back(std::abs(time - (0.317 + count * 60.0 / bpm)));
                }
            }
            ASSERT_FALSE(legacyErrors.empty());
            std::sort(legacyErrors.begin(), legacyErrors.end());
            totalImprovement += legacyErrors[legacyErrors.size() / 2] - errors[errors.size() / 2];
            RecordProperty("error_ms_" + std::to_string(rate) + "_" + std::to_string(static_cast<int>(bpm)),
                    std::to_string(errors[errors.size() / 2] * 1000));
            const auto grid = BeatFactory::makePreferredBeats(beats, {}, true,
                    mixxx::audio::SampleRate(rate));
            ASSERT_TRUE(grid);
            EXPECT_NEAR(bpm, grid->getBpmInRange(mixxx::audio::kStartFramePos,
                    mixxx::audio::FramePos(rate * 24.0)).value(), 0.1);
        }
    }
    EXPECT_GT(totalImprovement, 0.006);
}

TEST(AnalyzerGridTest, SilenceAndShortAudioDoNotInventBeats) {
    for (const double duration : {0.1, 2.0, 20.0}) {
        mixxx::AnalyzerQueenMaryBeats analyzer(true);
        ASSERT_TRUE(analyzer.initialize(mixxx::audio::SampleRate(44100)));
        feed(&analyzer, std::vector<CSAMPLE>(static_cast<size_t>(duration * 44100) * 2));
        EXPECT_TRUE(analyzer.getBeats().isEmpty());
        EXPECT_FALSE(analyzer.getFirstDownbeat().isValid());
    }
}

TEST(AnalyzerGridTest, ReinitializeClearsPreviousResults) {
    mixxx::AnalyzerQueenMaryBeats analyzer(true);
    ASSERT_TRUE(analyzer.initialize(mixxx::audio::SampleRate(44100)));
    feed(&analyzer, clickTrack(44100, 120.0, 20.0));
    ASSERT_FALSE(analyzer.getBeats().isEmpty());
    ASSERT_TRUE(analyzer.initialize(mixxx::audio::SampleRate(48000)));
    EXPECT_TRUE(analyzer.getBeats().isEmpty());
    EXPECT_FALSE(analyzer.getFirstDownbeat().isValid());
    feed(&analyzer, std::vector<CSAMPLE>(48000 * 2));
    EXPECT_TRUE(analyzer.getBeats().isEmpty());
}

TEST(AnalyzerGridTest, DynamicGridFollowsTempoChange) {
    const int rate = 44100;
    auto samples = clickTrack(rate, 120, 20);
    const auto slower = clickTrack(rate, 90, 20);
    samples.insert(samples.end(), slower.begin(), slower.end());
    mixxx::AnalyzerQueenMaryBeats analyzer(true);
    ASSERT_TRUE(analyzer.initialize(mixxx::audio::SampleRate(rate)));
    feed(&analyzer, samples);
    const auto map = BeatFactory::makePreferredBeats(analyzer.getBeats(), {}, false,
            mixxx::audio::SampleRate(rate));
    ASSERT_TRUE(map);
    EXPECT_NEAR(120, map->getBpmAroundPosition(mixxx::audio::FramePos(rate * 10.0), 8).value(), 2);
    EXPECT_NEAR(90, map->getBpmAroundPosition(mixxx::audio::FramePos(rate * 30.0), 8).value(), 2);
}

// Vocal segmentation must distinguish unavailable analysis from an instrumental
// result and preserve boundaries when a cached track is loaded again.
#include "analyzer/analyzervocals.h"
#include "analyzer/vocalanalysisstore.h"
#include "test/mixxxtest.h"
#include "track/track.h"
#include <QJsonArray>
#include <QTemporaryFile>

TEST(VocalAnalysisTest, ValidatesCacheAndKeepsEndExclusiveIntervals) {
    const QJsonObject valid{{"version", "test"}, {"duration", 8.0},
            {"segments", QJsonArray{QJsonArray{1.0, 3.0}, QJsonArray{5.0, 7.0}}}};
    const auto analysis = VocalAnalysis::fromJson(valid);
    ASSERT_TRUE(analysis);
    EXPECT_FALSE(analysis->hasVocal(0));
    EXPECT_TRUE(analysis->hasVocal(1));
    EXPECT_TRUE(analysis->hasVocal(2.99));
    EXPECT_FALSE(analysis->hasVocal(3));
    EXPECT_TRUE(analysis->hasVocal(6));
    EXPECT_FALSE(analysis->hasVocal(8));
    auto invalid = valid;
    invalid["segments"] = QJsonArray{QJsonArray{1.0, 4.0}, QJsonArray{3.0, 7.0}};
    EXPECT_FALSE(VocalAnalysis::fromJson(invalid));
    invalid["segments"] = QJsonArray{QJsonArray{1.0, 9.0}};
    EXPECT_FALSE(VocalAnalysis::fromJson(invalid));
    invalid["segments"] = QJsonArray{QJsonArray{-1.0, 2.0}};
    EXPECT_FALSE(VocalAnalysis::fromJson(invalid));
    invalid["segments"] = QJsonArray{};
    const auto instrumental = VocalAnalysis::fromJson(invalid);
    ASSERT_TRUE(instrumental);
    EXPECT_EQ(instrumental->status, VocalAnalysis::Status::Ready);
    EXPECT_FALSE(instrumental->hasVocal(2));
}

class AnalyzerVocalsTest : public MixxxTest {};
TEST_F(AnalyzerVocalsTest, PythonPipelineAndCachedReloadOfSilentTrack) {
    QTemporaryFile source;
    ASSERT_TRUE(source.open());
    source.write("silence fixture");
    source.flush();
    const auto track = Track::newTemporary(source.fileName());
    AnalyzerVocals analyzer(config());
    ASSERT_TRUE(analyzer.initialize(AnalyzerTrack(track), mixxx::audio::SampleRate(44100), 88200));
    std::vector<CSAMPLE> silence(44100 * 2 * 2, 0);
    ASSERT_TRUE(analyzer.processSamples(silence.data(), silence.size()));
    analyzer.storeResults(track);
    analyzer.cleanup();
    const auto result = VocalAnalysisStore::instance().get(track->getLocation());
    ASSERT_TRUE(result);
    ASSERT_EQ(result->status, VocalAnalysis::Status::Ready);
    EXPECT_NEAR(result->duration, 2.0, 0.001);
    EXPECT_TRUE(result->segments.empty());
    // No audio pass is needed when the same audio/model/script fingerprint exists.
    EXPECT_FALSE(analyzer.initialize(AnalyzerTrack(track), mixxx::audio::SampleRate(44100), 88200));
    EXPECT_EQ(VocalAnalysisStore::instance().get(track->getLocation())->status,
            VocalAnalysis::Status::Ready);
    analyzer.cleanup();
    // Changed audio must invalidate the result; cancellation is unavailable,
    // never an instrumental classification.
    source.write(" changed");
    source.flush();
    EXPECT_TRUE(analyzer.initialize(AnalyzerTrack(track), mixxx::audio::SampleRate(44100), 88200));
    analyzer.cleanup();
    EXPECT_EQ(VocalAnalysisStore::instance().get(track->getLocation())->status,
            VocalAnalysis::Status::Failed);
}

#include "widget/wvocalstatus.h"
#include "control/controlobject.h"

TEST_F(AnalyzerVocalsTest, VocalIndicatorTracksAnalysisAndEject) {
    ControlObject position(ConfigKey("[VocalWidgetTest]", "playposition"));
    WVocalStatus indicator("[VocalWidgetTest]", nullptr, nullptr);
    const auto track = Track::newTemporary("/tmp/vilelabox-vocal-indicator-test.wav");
    indicator.setTrack(track);
    EXPECT_EQ(indicator.text(), QStringLiteral("VOZ…"));

    auto vocal = std::make_shared<VocalAnalysis>();
    vocal->status = VocalAnalysis::Status::Ready;
    vocal->duration = 2;
    vocal->segments.append(qMakePair(0.0, 1.0));
    VocalAnalysisStore::instance().set(track->getLocation(), vocal);
    QCoreApplication::processEvents();
    EXPECT_EQ(indicator.text(), QStringLiteral("VOCAL"));
    EXPECT_EQ(indicator.getHighlight(), 1);

    position.forceSet(0.75);
    QCoreApplication::processEvents();
    EXPECT_EQ(indicator.text(), QStringLiteral("SEM VOZ"));
    EXPECT_EQ(indicator.getHighlight(), 0);

    auto unavailable = std::make_shared<VocalAnalysis>();
    unavailable->status = VocalAnalysis::Status::Failed;
    VocalAnalysisStore::instance().set(track->getLocation(), unavailable);
    QCoreApplication::processEvents();
    EXPECT_EQ(indicator.text(), QStringLiteral("VOZ ?"));
    indicator.setTrack(TrackPointer());
    EXPECT_EQ(indicator.text(), QStringLiteral("—"));
}

#include <QElapsedTimer>
#include <csignal>
namespace {
// A stand-in interpreter: a shell script that receives the detector's
// arguments (script --raw <audio> --sample-rate <rate>).
QString fakePython(const QTemporaryDir& directory, const QString& name, const QString& body) {
    const QString path = directory.filePath(name);
    QFile file(path);
    EXPECT_TRUE(file.open(QIODevice::WriteOnly));
    file.write(("#!/bin/sh\n" + body + "\n").toUtf8());
    file.close();
    file.setPermissions(QFile::ReadOwner | QFile::WriteOwner | QFile::ExeOwner);
    return path;
}
std::shared_ptr<const VocalAnalysis> runVocals(const UserSettingsPointer& config, const QString& audio,
        std::function<bool()> stopping = {}) {
    const auto track = Track::newTemporary(audio);
    AnalyzerVocals analyzer(config, std::move(stopping));
    if (analyzer.initialize(AnalyzerTrack(track), mixxx::audio::SampleRate(44100), 88200)) {
        std::vector<CSAMPLE> silence(44100 * 2, 0);
        analyzer.processSamples(silence.data(), silence.size());
        analyzer.storeResults(track);
    }
    analyzer.cleanup();
    return VocalAnalysisStore::instance().get(track->getLocation());
}
} // namespace
TEST_F(AnalyzerVocalsTest, DiagnosesEachFailure) {
    QTemporaryDir directory;
    const QString audio = directory.filePath("source.wav");
    QFile source(audio); ASSERT_TRUE(source.open(QIODevice::WriteOnly)); source.write("x"); source.close();
    const ConfigKey python("[Vilelabox]", "VocalPython");
    struct Case { QString body; VocalAnalysis::Error error; QString detail; };
    const QList<Case> cases{
            {"echo \"ModuleNotFoundError: No module named 'numpy'\" >&2; exit 1", VocalAnalysis::Error::DependenciesMissing, "numpy"},
            {"echo boom >&2; exit 3", VocalAnalysis::Error::DetectorFailed, "boom"},
            {"echo not-json; exit 0", VocalAnalysis::Error::InvalidResult, ""},
    };
    int index = 0;
    for (const auto& c : cases) {
        config()->set(python, ConfigValue(fakePython(directory, QString("py%1").arg(index++), c.body)));
        const auto result = runVocals(config(), audio);
        ASSERT_TRUE(result);
        EXPECT_EQ(VocalAnalysis::Status::Failed, result->status);
        EXPECT_EQ(c.error, result->error) << c.body.toStdString();
        if (!c.detail.isEmpty()) EXPECT_TRUE(result->detail.contains(c.detail)) << result->detail.toStdString();
        EXPECT_FALSE(result->describe().isEmpty());
    }
    config()->set(python, ConfigValue(directory.filePath("no-such-python")));
    const auto missing = runVocals(config(), audio);
    ASSERT_TRUE(missing);
    EXPECT_EQ(VocalAnalysis::Error::PythonMissing, missing->error);
}
TEST_F(AnalyzerVocalsTest, TimeoutKillsTheDetector) {
    QTemporaryDir directory;
    const QString audio = directory.filePath("source.wav");
    QFile source(audio); ASSERT_TRUE(source.open(QIODevice::WriteOnly)); source.write("x"); source.close();
    config()->set(ConfigKey("[Vilelabox]", "VocalPython"), ConfigValue(fakePython(directory, "slow", "exec sleep 30")));
    config()->setValue(ConfigKey("[Vilelabox]", "VocalTimeoutSeconds"), 1);
    QElapsedTimer timer; timer.start();
    const auto result = runVocals(config(), audio);
    EXPECT_LT(timer.elapsed(), 5000);
    ASSERT_TRUE(result);
    EXPECT_EQ(VocalAnalysis::Error::Timeout, result->error);
}
TEST_F(AnalyzerVocalsTest, CancellingStopsTheDetectorWithoutWaitingForTimeout) {
    QTemporaryDir directory;
    const QString audio = directory.filePath("source.wav");
    QFile source(audio); ASSERT_TRUE(source.open(QIODevice::WriteOnly)); source.write("x"); source.close();
    const QString pidFile = directory.filePath("pid"), rawFile = directory.filePath("raw");
    config()->set(ConfigKey("[Vilelabox]", "VocalPython"), ConfigValue(fakePython(directory, "hang",
            QString("echo $$ > '%1'; echo \"$3\" > '%2'; exec sleep 60").arg(pidFile, rawFile))));
    QElapsedTimer timer; timer.start();
    const auto result = runVocals(config(), audio, [&timer] { return timer.elapsed() > 300; });
    EXPECT_LT(timer.elapsed(), 3000); // the default timeout is 120 s
    ASSERT_TRUE(result);
    EXPECT_EQ(VocalAnalysis::Error::Cancelled, result->error);
    QFile pid(pidFile); ASSERT_TRUE(pid.open(QIODevice::ReadOnly));
    const int processId = pid.readAll().trimmed().toInt();
    ASSERT_GT(processId, 0);
    EXPECT_NE(0, ::kill(processId, 0)); // the detector process is gone
    QFile raw(rawFile); ASSERT_TRUE(raw.open(QIODevice::ReadOnly));
    EXPECT_FALSE(QFile::exists(QString::fromUtf8(raw.readAll().trimmed()))); // temporary audio removed
}
