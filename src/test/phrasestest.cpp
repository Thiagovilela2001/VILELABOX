#include <gtest/gtest.h>
#include <cmath>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QTemporaryDir>
#include <QFile>
#include <QFileInfo>
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
#include "library/related/relatedtracks.h"
TEST(RelatedTracksTest, CamelotNeighborsAndRelativeModes) {
    for (int i = 1; i <= 24; ++i) {
        auto key = KeyUtils::keyFromNumericValue(i);
        const auto compatible = KeyUtils::getCompatibleKeys(key);
        for (int j = 1; j <= 24; ++j) {
            auto candidate = KeyUtils::keyFromNumericValue(j);
            const bool expected = compatible.contains(candidate) &&
                    (KeyUtils::keyIsMajor(key) == KeyUtils::keyIsMajor(candidate) ||
                     KeyUtils::keyToOpenKeyNumber(key) == KeyUtils::keyToOpenKeyNumber(candidate));
            EXPECT_EQ(expected, vilelabox::relatedScore(120, 120, key, candidate).score > 0);
        }
    }
}
TEST(RelatedTracksTest, HalfDoubleTempoAndSixPercentLimit) {
    const auto key = KeyUtils::keyFromNumericValue(1);
    EXPECT_GT(vilelabox::relatedScore(120, 60, key, key).score, 0);
    EXPECT_GT(vilelabox::relatedScore(120, 240, key, key).score, 0);
    EXPECT_GT(vilelabox::relatedScore(120, 127.2, key, key).score, 0);
    EXPECT_EQ(0, vilelabox::relatedScore(120, 128, key, key).score);
    EXPECT_EQ(0, vilelabox::relatedScore(0, 120, key, key).score);
    EXPECT_GT(vilelabox::relatedScore(120, 120, key, key, 5, 5).score, vilelabox::relatedScore(120, 120, key, key, 5, 9).score);
}
TEST(RelatedTracksTest, ExplicitCamelotEightANeighbors) {
    const auto aMinor = KeyUtils::guessKeyFromText("Am");
    for (const auto& name : {"Am", "Dm", "Em", "C"}) {
        EXPECT_GT(vilelabox::relatedScore(120, 120, aMinor, KeyUtils::guessKeyFromText(name)).score, 0) << name;
    }
    for (const auto& name : {"F#", "G", "F"}) EXPECT_EQ(0, vilelabox::relatedScore(120, 120, aMinor, KeyUtils::guessKeyFromText(name)).score);
}
#include "analyzer/analyzerenergy.h"
TEST_F(AnalyzerPhrasesTest, EnergyIsCachedSeparatelyAndInvalidatedByFileChange) {
    QTemporaryDir directory;
    QFile source(directory.filePath("energy.wav")); ASSERT_TRUE(source.open(QIODevice::WriteOnly)); source.write("first"); source.close();
    auto track = Track::newTemporary(source.fileName());
    config()->setValue(ConfigKey("[Analyzer]", "AnalyzeRelatedTracks"), true);
    AnalyzerEnergy analyzer(config());
    ASSERT_TRUE(analyzer.initialize(AnalyzerTrack(track), mixxx::audio::SampleRate(44100), 44100));
    std::vector<CSAMPLE> silence(44100 * 2, 0);
    ASSERT_TRUE(analyzer.processSamples(silence.data(), silence.size()));
    analyzer.storeResults(track); analyzer.cleanup();
    EXPECT_DOUBLE_EQ(1, vilelabox::loadEnergies(config()->getSettingsPath()).value(vilelabox::energyIdentity(source.fileName())));
    EXPECT_FALSE(analyzer.initialize(AnalyzerTrack(track), mixxx::audio::SampleRate(44100), 44100));
    ASSERT_TRUE(source.open(QIODevice::Append)); source.write("changed"); source.close();
    EXPECT_TRUE(analyzer.initialize(AnalyzerTrack(track), mixxx::audio::SampleRate(44100), 44100));
}
TEST_F(AnalyzerPhrasesTest, AutomaticCuesToleratePhraseShiftsFromGridEdits) {
    auto track = Track::newTemporary(QStringLiteral("auto-phrase-shift.wav"));
    track->setAudioProperties(mixxx::audio::ChannelCount(2), mixxx::audio::SampleRate(44100),
            mixxx::audio::Bitrate(1411), mixxx::Duration::fromSeconds(176));
    track->trySetBeats(grid());
    AutoPhraseCues generator;
    generator.setTrack(track);
    ASSERT_TRUE(generator.apply(vilelabox::segmentPhrases(sequence(), 176, grid())));
    const int created = track->getCuePoints().size();
    ASSERT_GT(created, 0);
    // Same track loaded again after nudging the grid by 10 ms.
    const auto nudged = mixxx::Beats::fromConstTempo(mixxx::audio::SampleRate(44100),
            mixxx::audio::FramePos(441), mixxx::Bpm(120))->withDownbeat(mixxx::audio::FramePos(441));
    track->trySetBeats(nudged);
    generator.setTrack(track);
    ASSERT_TRUE(generator.apply(vilelabox::segmentPhrases(sequence(), 176, nudged)));
    EXPECT_EQ(created, track->getCuePoints().size());
}
TEST_F(AnalyzerPhrasesTest, GridEditsWriteCacheOnceAfterTheySettle) {
    QTemporaryDir directory;
    QFile source(directory.filePath("debounce.wav")); ASSERT_TRUE(source.open(QIODevice::WriteOnly)); source.write("x"); source.close();
    auto track = Track::newTemporary(source.fileName()); track->trySetBeats(grid());
    const auto cache = PhraseAnalysisStore::cachePath(directory.path(), source.fileName());
    const auto derived = PhraseAnalysisStore::derivedPath(cache);
    PhraseAnalysisStore::instance().flushPendingSaves(); // leftovers of other tests
    vilelabox::PhraseAnalysis analysis; analysis.features = sequence(); analysis.duration = 176;
    // The analysis wrote the immutable features once.
    ASSERT_TRUE(PhraseAnalysisStore::saveFeatures(cache, analysis));
    QFile featuresFile(cache); ASSERT_TRUE(featuresFile.open(QIODevice::ReadOnly));
    const QByteArray features = featuresFile.readAll(); featuresFile.close();
    const auto modified = QFileInfo(cache).lastModified();
    PhraseAnalysisStore::instance().set(track, analysis, cache);
    EXPECT_FALSE(QFile::exists(derived));
    for (int i = 1; i <= 20; ++i) {
        const auto position = mixxx::audio::FramePos(i * 10);
        track->trySetBeats(mixxx::Beats::fromConstTempo(mixxx::audio::SampleRate(44100), position,
                mixxx::Bpm(120))->withDownbeat(position));
        QCoreApplication::processEvents();
    }
    EXPECT_FALSE(QFile::exists(derived));
    EXPECT_EQ(1, PhraseAnalysisStore::instance().pendingSaveCount());
    PhraseAnalysisStore::instance().flushPendingSaves();
    EXPECT_EQ(0, PhraseAnalysisStore::instance().pendingSaveCount());
    EXPECT_TRUE(QFile::exists(derived));
    // Grid edits never rewrite the features.
    ASSERT_TRUE(featuresFile.open(QIODevice::ReadOnly));
    EXPECT_EQ(features, featuresFile.readAll());
    EXPECT_EQ(modified, QFileInfo(cache).lastModified());
    vilelabox::PhraseAnalysis loaded;
    ASSERT_TRUE(PhraseAnalysisStore::load(cache, &loaded));
    EXPECT_EQ(88, loaded.features.size());
}
#include <QElapsedTimer>
#include <QSignalSpy>
#include <QTest>
#include <QSqlDatabase>
#include <QSqlQuery>
#include <QUuid>
namespace {
// Minimal Mixxx library schema with `count` tracks; every 4th is 120 BPM in
// C major (compatible with the reference), the rest are incompatible.
QString makeLibrary(const QString& directory, int count) {
    const QString path = QDir(directory).filePath(QStringLiteral("library-%1.sqlite").arg(count));
    const QString name = QUuid::createUuid().toString();
    {
        auto db = QSqlDatabase::addDatabase("QSQLITE", name);
        db.setDatabaseName(path);
        EXPECT_TRUE(db.open());
        QSqlQuery q(db);
        q.exec("CREATE TABLE track_locations (id INTEGER PRIMARY KEY, location TEXT, fs_deleted INTEGER)");
        q.exec("CREATE TABLE library (id INTEGER PRIMARY KEY, artist TEXT, title TEXT, bpm REAL, key_id INTEGER, location INTEGER, mixxx_deleted INTEGER)");
        db.transaction();
        QSqlQuery insertLocation(db), insertTrack(db);
        insertLocation.prepare("INSERT INTO track_locations VALUES (?, ?, 0)");
        insertTrack.prepare("INSERT INTO library VALUES (?, ?, ?, ?, ?, ?, 0)");
        for (int i = 1; i <= count; ++i) {
            const bool compatible = i % 4 == 0;
            insertLocation.addBindValue(i); insertLocation.addBindValue(QStringLiteral("/music/%1.mp3").arg(i)); insertLocation.exec();
            insertTrack.addBindValue(i); insertTrack.addBindValue("artist"); insertTrack.addBindValue(QStringLiteral("t%1").arg(i));
            // Compatible tracks spread over 117.6–122.4 BPM so scores differ.
            insertTrack.addBindValue(compatible ? 117.6 + (i % 97) * 0.05 : 97.0);
            insertTrack.addBindValue(compatible ? 1 : 7);
            insertTrack.addBindValue(i); insertTrack.exec();
        }
        db.commit();
        db.close();
    }
    QSqlDatabase::removeDatabase(name);
    return path;
}
vilelabox::RelatedReference cMajorReference(double bpm = 120) {
    vilelabox::RelatedReference reference;
    reference.location = QStringLiteral("/music/4.mp3");
    reference.bpm = bpm;
    reference.key = mixxx::track::io::key::C_MAJOR;
    return reference;
}
} // namespace
TEST(RelatedTracksTest, SearchRespectsLimitOrderAndExcludesReference) {
    QTemporaryDir directory;
    const auto path = makeLibrary(directory.path(), 400);
    const auto rows = vilelabox::findRelated(path, directory.path(), cMajorReference(), 25);
    ASSERT_EQ(25, rows.size());
    for (int i = 1; i < rows.size(); ++i) EXPECT_GE(rows[i - 1].score.score, rows[i].score.score);
    for (const auto& row : rows) EXPECT_NE(4, row.id); // the reference itself
    // Half tempo still matches (60 BPM reference against ~120 BPM tracks).
    EXPECT_EQ(25, vilelabox::findRelated(path, directory.path(), cMajorReference(60), 25).size());
    // Nothing compatible at 97 BPM in C major.
    EXPECT_TRUE(vilelabox::findRelated(path, directory.path(), cMajorReference(150), 25).isEmpty());
}
TEST(RelatedTracksTest, OnlyTheLatestSearchIsReported) {
    QTemporaryDir directory;
    const auto path = makeLibrary(directory.path(), 20000);
    vilelabox::RelatedSearch search(path, directory.path());
    QSignalSpy spy(&search, &vilelabox::RelatedSearch::finished);
    search.start(cMajorReference(120), 100); // superseded before it finishes
    search.start(cMajorReference(150), 100); // no compatible tracks
    ASSERT_TRUE(spy.wait(10000));
    QTest::qWait(300);
    ASSERT_EQ(1, spy.count());
    EXPECT_TRUE(spy.at(0).at(0).value<QVector<vilelabox::RelatedRow>>().isEmpty());
}
TEST(RelatedTracksTest, SearchTimesForLibrarySizes) {
    QTemporaryDir directory;
    for (int size : {1000, 10000, 50000}) {
        const auto path = makeLibrary(directory.path(), size);
        QElapsedTimer timer;
        timer.start();
        const auto rows = vilelabox::findRelated(path, directory.path(), cMajorReference(), 100);
        const auto elapsed = timer.elapsed();
        qInfo() << "RELATED-TIMING" << size << "tracks:" << elapsed << "ms," << rows.size() << "rows";
        EXPECT_EQ(100, rows.size());
        EXPECT_LT(elapsed, 10000);
    }
}
TEST(RelatedTracksTest, EnergyLookupByIdentity) {
    QTemporaryDir directory;
    vilelabox::saveEnergy(directory.path(), "/music/a.mp3", 7.5);
    EXPECT_DOUBLE_EQ(7.5, vilelabox::loadEnergy(directory.path(), "/music/a.mp3").value_or(0));
    EXPECT_FALSE(vilelabox::loadEnergy(directory.path(), "/music/b.mp3").has_value());
}
TEST_F(AnalyzerPhrasesTest, OlderPhraseWriteNeverReplacesNewer) {
    QTemporaryDir directory;
    const auto cache = directory.filePath("ordered.json");
    auto newer = std::make_shared<vilelabox::PhraseAnalysis>(vilelabox::segmentPhrases(sequence(), 176, grid()));
    auto older = std::make_shared<vilelabox::PhraseAnalysis>(*newer);
    older->phrases.clear();
    auto& store = PhraseAnalysisStore::instance();
    store.writeDerived("ordered-track", cache, newer, grid(), 1000005);
    store.writeDerived("ordered-track", cache, older, grid(), 1000003); // arrives late
    QFile file(PhraseAnalysisStore::derivedPath(cache)); ASSERT_TRUE(file.open(QIODevice::ReadOnly));
    const auto object = QJsonDocument::fromJson(file.readAll()).object();
    EXPECT_EQ(1000005, object.value("sequence").toInteger());
    EXPECT_EQ(newer->phrases.size(), object.value("phrases").toArray().size());
}
TEST_F(AnalyzerPhrasesTest, ReadsSingleFileCachesFromEarlierVersions) {
    QTemporaryDir directory;
    const auto cache = directory.filePath("legacy.json");
    const auto result = vilelabox::segmentPhrases(sequence(), 176, grid());
    // The first release wrote features, phrases and grid into one file.
    QJsonArray features;
    for (const auto& f : result.features) {
        QJsonArray row{f.start, f.end, f.rms, f.bass, f.flux};
        for (auto v : f.chroma) row.append(v);
        for (auto v : f.timbre) row.append(v);
        features.append(row);
    }
    QFile file(cache); ASSERT_TRUE(file.open(QIODevice::WriteOnly));
    file.write(QJsonDocument(QJsonObject{{"version", 1}, {"duration", 176}, {"features", features},
            {"phrases", QJsonArray{QJsonArray{0, 32, 16, 1, 0.65}}}, {"grid", QJsonObject{}}}).toJson());
    file.close();
    vilelabox::PhraseAnalysis loaded;
    ASSERT_TRUE(PhraseAnalysisStore::load(cache, &loaded));
    EXPECT_EQ(88, loaded.features.size());
}
TEST_F(AnalyzerPhrasesTest, MemoryKeepsTracksInUseAndTheMostRecent) {
    auto& store = PhraseAnalysisStore::instance();
    QVector<TrackPointer> tracks;
    vilelabox::PhraseAnalysis small; small.features = sequence().mid(0, 8); small.duration = 16;
    // The first track is "loaded in a deck": something holds its result.
    tracks.push_back(Track::newTemporary(QStringLiteral("/tmp/vilelabox-memory-deck.wav")));
    store.set(tracks.back(), small, {});
    const auto inUse = store.get(PhraseAnalysisStore::trackKey(tracks.back()));
    ASSERT_TRUE(inUse);
    for (int i = 0; i < 80; ++i) {
        tracks.push_back(Track::newTemporary(QStringLiteral("/tmp/vilelabox-memory-%1.wav").arg(i)));
        store.set(tracks.back(), small, {});
    }
    EXPECT_LE(store.cachedTrackCount(), PhraseAnalysisStore::kMaxCachedTracks + 1);
    EXPECT_EQ(inUse, store.get(PhraseAnalysisStore::trackKey(tracks.front()))); // kept
    EXPECT_FALSE(store.get(PhraseAnalysisStore::trackKey(tracks.at(1)))); // oldest unused: dropped
    tracks.push_back(Track::newTemporary(QStringLiteral("/tmp/vilelabox-memory-last.wav")));
    store.set(tracks.back(), small, {});
    EXPECT_TRUE(store.get(PhraseAnalysisStore::trackKey(tracks.back())));
    // Measured footprint of the features: about 13 MB per hour of audio.
    const double bytesPerHour = sizeof(vilelabox::PhraseFeature) * 20.0 * 3600;
    qInfo() << "PHRASE-MEMORY" << sizeof(vilelabox::PhraseFeature) << "bytes per 50 ms block,"
            << bytesPerHour / 1e6 << "MB per hour of audio";
    EXPECT_LT(bytesPerHour, 20e6);
}
TEST_F(AnalyzerPhrasesTest, FailedPhraseWriteKeepsTheResultInMemory) {
    QTemporaryDir directory;
    QDir(directory.path()).mkdir("locked");
    const QString locked = directory.filePath("locked");
    QFile::setPermissions(locked, QFile::ReadOwner | QFile::ExeOwner);
    auto track = Track::newTemporary(QStringLiteral("/tmp/vilelabox-unwritable.wav"));
    track->trySetBeats(grid());
    vilelabox::PhraseAnalysis analysis; analysis.features = sequence(); analysis.duration = 176;
    const QString cache = QDir(locked).filePath("cache.json");
    PhraseAnalysisStore::instance().set(track, analysis, cache);
    track->trySetBeats(grid()->withDownbeat(mixxx::audio::FramePos(22050)));
    QCoreApplication::processEvents();
    PhraseAnalysisStore::instance().flushPendingSaves();
    EXPECT_FALSE(QFile::exists(PhraseAnalysisStore::derivedPath(cache)));
    EXPECT_TRUE(PhraseAnalysisStore::instance().get(PhraseAnalysisStore::trackKey(track)));
    QFile::setPermissions(locked, QFile::ReadOwner | QFile::WriteOwner | QFile::ExeOwner);
}
#include "track/mixpoints.h"
namespace {
vilelabox::MixTrack mixTrack(const vilelabox::PhraseAnalysis& phrases) {
    vilelabox::MixTrack track;
    track.phrases = &phrases;
    track.beats = grid(); // 120 BPM, bar 1 at 0 s: one bar every 2 s
    track.duration = 176;
    return track;
}
} // namespace
TEST(MixPointsTest, EntryIsIntroCueElseBarOne) {
    const auto phrases = vilelabox::segmentPhrases(sequence(), 176, grid());
    auto track = mixTrack(phrases);
    EXPECT_DOUBLE_EQ(0, vilelabox::mixInPoint(track).value());
    track.introCue = 10.3;
    EXPECT_DOUBLE_EQ(10, vilelabox::mixInPoint(track).value()); // snapped to the bar
}
TEST(MixPointsTest, IntroLengthEndsAtFirstDropOrVocal) {
    const auto phrases = vilelabox::segmentPhrases(sequence(), 176, grid());
    auto track = mixTrack(phrases);
    EXPECT_EQ(16, vilelabox::introBars(track, 0)); // first CHORUS at 32 s
    track.vocals = {{20.0, 30.0}};
    EXPECT_EQ(8, vilelabox::introBars(track, 0)); // vocal at bar 11 -> 8 bars
    track.phrases = nullptr; track.vocals.clear();
    EXPECT_EQ(0, vilelabox::introBars(track, 0)); // unknown
}
TEST(MixPointsTest, ExitAfterTheLastVocalOrOutroCue) {
    const auto phrases = vilelabox::segmentPhrases(sequence(), 176, grid());
    auto track = mixTrack(phrases);
    auto range = vilelabox::mixOutRange(track).value();
    EXPECT_DOUBLE_EQ(112, range.start); // no OUTRO, no vocals: 32 bars before the end
    EXPECT_DOUBLE_EQ(176, range.end);
    track.vocals = {{40.0, 130.0}};
    range = vilelabox::mixOutRange(track).value();
    EXPECT_DOUBLE_EQ(144, range.start); // first 8-bar boundary after the vocals (bar 73)
    track.outroCue = 150.4;
    EXPECT_DOUBLE_EQ(150, vilelabox::mixOutRange(track)->start);
}
TEST(MixPointsTest, TriggerLandsIncomingDropOnOutroEnd) {
    const auto phrases = vilelabox::segmentPhrases(sequence(), 176, grid());
    const auto track = mixTrack(phrases);
    const vilelabox::MixOutRange range{144, 176};
    EXPECT_DOUBLE_EQ(144, vilelabox::mixTrigger(track, range, 16, 0)); // 32 s of intro = outro
    EXPECT_DOUBLE_EQ(128, vilelabox::mixTrigger(track, range, 24, 0)); // longer intro starts earlier
    EXPECT_DOUBLE_EQ(144, vilelabox::mixTrigger(track, range, 0, 0));  // unknown intro: at the exit
    EXPECT_DOUBLE_EQ(144, vilelabox::mixTrigger(track, range, 24, 130)); // already past: the exit
    EXPECT_DOUBLE_EQ(160, vilelabox::mixTrigger(track, range, 24, 150)); // past the exit: next phrase
}
