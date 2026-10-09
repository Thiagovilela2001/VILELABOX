#include <gtest/gtest.h>
#include <cmath>
#include <QJsonDocument>
#include <QJsonObject>
#include <QTemporaryDir>
#include <QFile>
#include "track/phrases.h"
#include "analyzer/phraseanalysisstore.h"
#include "analyzer/analyzerphrases.h"
#include "track/track.h"
#include "test/mixxxtest.h"

namespace {
QVector<vilelabox::PhraseFeature> sequence() {
    QVector<vilelabox::PhraseFeature> features;
    for (int bar = 0; bar < 88; ++bar) {
        const bool high = (bar >= 16 && bar < 48) || bar >= 56;
        vilelabox::PhraseFeature f;
        f.start = bar * 2; f.end = (bar + 1) * 2;
        f.rms = high ? 1 : 0.2; f.bass = high ? 1 : 0.01;
        f.chroma[high ? 0 : 7] = 1;
        f.timbre[high ? 0 : 5] = 1;
        features.push_back(f);
    }
    return features;
}
mixxx::BeatsPointer grid() {
    return mixxx::Beats::fromConstTempo(mixxx::audio::SampleRate(44100),
            mixxx::audio::kStartFramePos, mixxx::Bpm(120))->withDownbeat(mixxx::audio::kStartFramePos);
}
}
TEST(PhrasesTest, SyntheticSectionsAndEndExclusiveLookup) {
    const auto result = vilelabox::segmentPhrases(sequence(), 176, grid());
    ASSERT_EQ(4, result.phrases.size());
    const vilelabox::PhraseType expected[]{vilelabox::PhraseType::Intro, vilelabox::PhraseType::Chorus,
            vilelabox::PhraseType::Down, vilelabox::PhraseType::Chorus};
    const double starts[]{0, 32, 96, 112};
    for (int i = 0; i < 4; ++i) { EXPECT_EQ(expected[i], result.phrases[i].type); EXPECT_DOUBLE_EQ(starts[i], result.phrases[i].start); }
    EXPECT_EQ(0, result.indexAt(31.99)); EXPECT_EQ(1, result.indexAt(32)); EXPECT_EQ(-1, result.indexAt(176));
}
TEST(PhrasesTest, MissingGridAndDownbeat) {
    EXPECT_TRUE(vilelabox::segmentPhrases(sequence(), 176, {}).phrases.isEmpty());
    auto result = vilelabox::segmentPhrases(sequence(), 176, grid()->withoutDownbeat());
    EXPECT_TRUE(result.lowConfidence);
    for (const auto& phrase : result.phrases) EXPECT_EQ(vilelabox::PhraseType::Unknown, phrase.type);
}
TEST(PhrasesTest, GridEditsReuseFeatures) {
    for (const auto beats : {grid()->withDownbeat(mixxx::audio::FramePos(22050)),
                *grid()->tryScale(mixxx::Beats::BpmScale::Double), *grid()->tryScale(mixxx::Beats::BpmScale::Halve)}) {
        const auto result = vilelabox::segmentPhrases(sequence(), 176, beats);
        EXPECT_FALSE(result.phrases.isEmpty());
        EXPECT_EQ(88, result.features.size());
        for (const auto& phrase : result.phrases) {
            auto position = beats->barPosition(mixxx::audio::FramePos(phrase.start * 44100));
            ASSERT_TRUE(position);
            if (phrase.start > 0) EXPECT_EQ(1, position->beat);
        }
    }
}
TEST(PhrasesTest, CacheRejectsVersionAndChangedFile) {
    QTemporaryDir directory;
    const auto source = directory.filePath("source.wav");
    QFile file(source); ASSERT_TRUE(file.open(QIODevice::WriteOnly)); file.write("a"); file.close();
    const auto path = PhraseAnalysisStore::cachePath(directory.path(), source);
    const auto result = vilelabox::segmentPhrases(sequence(), 176, grid());
    ASSERT_TRUE(PhraseAnalysisStore::save(path, result, grid()));
    vilelabox::PhraseAnalysis loaded;
    ASSERT_TRUE(PhraseAnalysisStore::load(path, &loaded));
    EXPECT_EQ(88, loaded.features.size());
    ASSERT_TRUE(file.open(QIODevice::Append)); file.write("b"); file.close();
    EXPECT_NE(path, PhraseAnalysisStore::cachePath(directory.path(), source));
    QFile cache(path); ASSERT_TRUE(cache.open(QIODevice::ReadOnly));
    auto object = QJsonDocument::fromJson(cache.readAll()).object(); cache.close();
    object["version"] = -1;
    ASSERT_TRUE(cache.open(QIODevice::WriteOnly | QIODevice::Truncate)); cache.write(QJsonDocument(object).toJson()); cache.close();
    EXPECT_FALSE(PhraseAnalysisStore::load(path, &loaded));
}
class AnalyzerPhrasesTest : public MixxxTest {};
TEST_F(AnalyzerPhrasesTest, StreamingSilenceCacheAndGridRecalculation) {
    QTemporaryDir directory;
    QFile source(directory.filePath("audio.wav")); ASSERT_TRUE(source.open(QIODevice::WriteOnly)); source.write("fixture"); source.close();
    auto track = Track::newTemporary(source.fileName()); track->trySetBeats(grid());
    config()->setValue(ConfigKey("[Analyzer]", "AnalyzePhrases"), true);
    AnalyzerPhrases analyzer(config());
    ASSERT_TRUE(analyzer.initialize(AnalyzerTrack(track), mixxx::audio::SampleRate(44100), 44100 * 16));
    std::vector<CSAMPLE> samples(44100 * 2, 0);
    for (int i = 0; i < 16; ++i) ASSERT_TRUE(analyzer.processSamples(samples.data(), samples.size()));
    analyzer.storeResults(track); analyzer.cleanup();
    auto result = PhraseAnalysisStore::instance().get(track->getLocation());
    ASSERT_TRUE(result); EXPECT_DOUBLE_EQ(16, result->duration); EXPECT_FALSE(result->features.isEmpty());
    EXPECT_FALSE(analyzer.initialize(AnalyzerTrack(track), mixxx::audio::SampleRate(44100), 44100 * 16));
    track->trySetBeats(grid()->withDownbeat(mixxx::audio::FramePos(22050)));
    QCoreApplication::processEvents();
    result = PhraseAnalysisStore::instance().get(track->getLocation());
    ASSERT_TRUE(result); EXPECT_DOUBLE_EQ(16, result->duration);
}
TEST_F(AnalyzerPhrasesTest, SyntheticAudioFindsFourSectionsAcrossChunkBoundaries) {
    QTemporaryDir directory;
    QFile source(directory.filePath("synthetic.wav")); ASSERT_TRUE(source.open(QIODevice::WriteOnly)); source.write("synthetic"); source.close();
    auto track = Track::newTemporary(source.fileName());
    track->trySetBeats(mixxx::Beats::fromConstTempo(mixxx::audio::SampleRate(8000),
            mixxx::audio::kStartFramePos, mixxx::Bpm(120))->withDownbeat(mixxx::audio::kStartFramePos));
    config()->setValue(ConfigKey("[Analyzer]", "AnalyzePhrases"), true);
    AnalyzerPhrases analyzer(config());
    ASSERT_TRUE(analyzer.initialize(AnalyzerTrack(track), mixxx::audio::SampleRate(8000), 8000 * 176));
    std::vector<CSAMPLE> samples(738 * 2);
    for (int frame = 0; frame < 8000 * 176;) {
        const int count = std::min(738, 8000 * 176 - frame);
        for (int j = 0; j < count; ++j) {
            double t = (frame + j) / 8000.0;
            bool chorus = (t >= 32 && t < 96) || t >= 112;
            const float sample = 0.1 * std::sin(t * 2 * 3.141592653589793 * 2000) +
                    (chorus ? 0.8 * std::sin(t * 2 * 3.141592653589793 * 60) : 0);
            samples[2 * j] = samples[2 * j + 1] = sample;
        }
        ASSERT_TRUE(analyzer.processSamples(samples.data(), count * 2));
        frame += count;
    }
    analyzer.storeResults(track); analyzer.cleanup();
    auto result = PhraseAnalysisStore::instance().get(track->getLocation());
    ASSERT_TRUE(result); ASSERT_EQ(4, result->phrases.size());
    EXPECT_DOUBLE_EQ(32, result->phrases[1].start);
    EXPECT_DOUBLE_EQ(96, result->phrases[2].start);
    EXPECT_DOUBLE_EQ(112, result->phrases[3].start);
}
#include "track/autophrasecues.h"
TEST_F(AnalyzerPhrasesTest, AutomaticCuesPreserveUserSlotsLockAndUndoEdits) {
    auto track = Track::newTemporary(QStringLiteral("auto-phrase-cues.wav"));
    track->setAudioProperties(mixxx::audio::ChannelCount(2), mixxx::audio::SampleRate(44100),
            mixxx::audio::Bitrate(1411), mixxx::Duration::fromSeconds(176));
    auto user = track->createAndAddCue(mixxx::CueType::HotCue, 0, mixxx::audio::FramePos(100), mixxx::audio::kInvalidFramePos);
    user->setLabel("User");
    AutoPhraseCues generator;
    generator.setTrack(track);
    const auto result = vilelabox::segmentPhrases(sequence(), 176, grid());
    track->setBpmLocked(true); generator.apply(result); EXPECT_EQ(1, track->getCuePoints().size());
    track->setBpmLocked(false);
    generator.apply(result, Track::newTemporary(QStringLiteral("another-track.wav")));
    EXPECT_EQ(1, track->getCuePoints().size());
    generator.apply(result);
    ASSERT_EQ(5, track->getCuePoints().size());
    EXPECT_EQ("User", user->getLabel()); EXPECT_EQ(mixxx::audio::FramePos(100), user->getPosition());
    generator.apply(result); EXPECT_EQ(5, track->getCuePoints().size());
    auto intro = track->findCueByType(mixxx::CueType::Intro); ASSERT_TRUE(intro); intro->setLabel("My Intro");
    generator.undo(); EXPECT_EQ(2, track->getCuePoints().size());
    EXPECT_TRUE(track->getCuePoints().contains(user)); EXPECT_TRUE(track->getCuePoints().contains(intro));
    generator.setTrack(track); generator.apply({}); EXPECT_EQ(2, track->getCuePoints().size());
}
