#include <memory>

#include "test/mockedenginebackendtest.h"
#include "track/beats.h"

class BeatsTranslateTest : public MockedEngineBackendTest {
};

TEST_F(BeatsTranslateTest, SimpleTranslateMatch) {
    ControlObject::set(ConfigKey(m_sGroup1, QStringLiteral("quantize")), 0);
    ControlObject::set(ConfigKey(m_sGroup2, QStringLiteral("quantize")), 0);
    // Set up BeatGrids for decks 1 and 2.
    const auto bpm = mixxx::Bpm(60.0);
    constexpr auto firstBeat = mixxx::audio::kStartFramePos;
    auto grid1 = mixxx::Beats::fromConstTempo(
            m_pTrack1->getSampleRate(), firstBeat, bpm);
    m_pTrack1->trySetBeats(grid1);
    ASSERT_DOUBLE_EQ(firstBeat.value(),
            grid1->findClosestBeat(mixxx::audio::kStartFramePos).value());

    auto grid2 = mixxx::Beats::fromConstTempo(
            m_pTrack2->getSampleRate(), firstBeat, bpm);
    m_pTrack2->trySetBeats(grid2);
    ASSERT_DOUBLE_EQ(firstBeat.value(),
            grid2->findClosestBeat(mixxx::audio::kStartFramePos).value());

    // Seek deck 1 forward a bit.
    const auto seekPosition = mixxx::audio::FramePos{1111.0};
    m_pChannel1->getEngineBuffer()->seekAbs(seekPosition);
    ProcessBuffer();
    EXPECT_TRUE(m_pChannel1->getEngineBuffer()->getVisualPlayPos() > 0);

    // Make both decks playing.
    ControlObject::getControl(m_sGroup1, "play")->set(1.0);
    ControlObject::getControl(m_sGroup2, "play")->set(1.0);
    ProcessBuffer();
    // Manually set the "bpm" control... I would like to figure out why this
    // doesn't get set naturally, but this will do for now.
    auto pBpm1 = std::make_unique<ControlProxy>(m_sGroup1, "bpm");
    auto pBpm2 = std::make_unique<ControlProxy>(m_sGroup1, "bpm");
    pBpm1->set(bpm.value());
    pBpm2->set(bpm.value());
    ProcessBuffer();

    // Push the button on deck 2.
    auto pTranslateMatchAlignment = std::make_unique<ControlProxy>(
        m_sGroup2, "beats_translate_match_alignment");
    pTranslateMatchAlignment->set(1.0);
    ProcessBuffer();

    // Deck 1 is +seekPosition away from its closest beat (which is at 0).
    // Deck 2 was left at 0. We translated grid 2 so that it is also +seekPosition
    // away from its closest beat, so that beat should be at -seekPosition.
    mixxx::BeatsPointer pBeats = m_pTrack2->getBeats();
    EXPECT_FRAMEPOS_EQ(seekPosition * -1, pBeats->findClosestBeat(mixxx::audio::kStartFramePos));
}

TEST_F(BeatsTranslateTest, BeatsUndoTest) {
    const auto bpm60 = mixxx::Bpm(60.0);
    const auto origin123 = mixxx::audio::FramePos{123.0};
    auto grid = mixxx::Beats::fromConstTempo(
            m_pTrack1->getSampleRate(), mixxx::audio::kStartFramePos, bpm60);
    m_pTrack1->trySetBeats(grid);
    EXPECT_FRAMEPOS_EQ(mixxx::audio::kStartFramePos, mixxx::audio::kStartFramePos);

    auto pBpm = std::make_unique<ControlProxy>(m_sGroup1, "bpm");
    auto pBeatsTranslateCurPos =
            std::make_unique<ControlProxy>(m_sGroup1, "beats_translate_curpos");
    auto pBeatsUndo = std::make_unique<ControlProxy>(m_sGroup1, "beats_undo_adjustment");
    pBpm->set(bpm60.value());
    m_pChannel1->getEngineBuffer()->seekAbs(origin123);
    ProcessBuffer(); // first process to schedule seek in a stopped deck
    ProcessBuffer(); // then seek
    // Beats undo ignores changes done in quick succession (ignore delay is 800 millis),
    // so after the each set of beat changes, the newest item in the undo stack
    QTest::qSleep(810);

    // Populate the beats_undo stack with 10 "beats_translate_curpos" actions at
    // different positions.
    // should be the state from before the sequence.
    // In order to have a before state, we translate the beats to 123.
    pBeatsTranslateCurPos->set(1.0);
    pBeatsTranslateCurPos->set(0.0);
    // Now sleep so only the steps of the sequence are recognized as quick actions.
    auto newSeekPosition = m_pChannel1->getEngineBuffer()->getExactPlayPos();
    EXPECT_FRAMEPOS_EQ(origin123, newSeekPosition);
    QTest::qSleep(810);

    for (int i = 0; i < 5; i++) {
        newSeekPosition += 5.0;
        m_pChannel1->getEngineBuffer()->seekAbs(newSeekPosition);
        ProcessBuffer(); // first process to schedule seek in a stopped deck
        ProcessBuffer(); // then seek
        pBeatsTranslateCurPos->set(1.0);
        pBeatsTranslateCurPos->set(0.0);
    }

    // We should be at 148 now
    newSeekPosition = m_pChannel1->getEngineBuffer()->getExactPlayPos();
    const auto pos148 = mixxx::audio::FramePos{148.0};
    EXPECT_FRAMEPOS_EQ(newSeekPosition, pos148);

    // Sleep to keep the current state in the undo stack.
    QTest::qSleep(810);

    for (int i = 0; i < 5; i++) {
        newSeekPosition += 5.0;
        m_pChannel1->getEngineBuffer()->seekAbs(newSeekPosition);
        ProcessBuffer(); // first process to schedule seek in a stopped deck
        ProcessBuffer(); // then seek
        pBeatsTranslateCurPos->set(1.0);
        pBeatsTranslateCurPos->set(0.0);
    }
    // After this quick sequence we're at frame 173.
    newSeekPosition = m_pChannel1->getEngineBuffer()->getExactPlayPos();
    EXPECT_FRAMEPOS_EQ(newSeekPosition, mixxx::audio::FramePos{173.0});

    pBeatsUndo->set(1.0);
    pBeatsUndo->set(0.0);
    mixxx::BeatsPointer pBeats = m_pTrack1->getBeats();
    EXPECT_FRAMEPOS_EQ(pos148, pBeats->findClosestBeat(mixxx::audio::kStartFramePos));

    // Undo once more to get back to 123.
    pBeatsUndo->set(1.0);
    pBeatsUndo->set(0.0);
    pBeats = m_pTrack1->getBeats();
    EXPECT_FRAMEPOS_EQ(origin123, pBeats->findClosestBeat(mixxx::audio::kStartFramePos));
}

TEST_F(BeatsTranslateTest, SetDownbeatCountsSeeksAndClears) {
    const auto beats = mixxx::Beats::fromConstTempo(m_pTrack1->getSampleRate(),
            mixxx::audio::kStartFramePos, mixxx::Bpm(120.0));
    ASSERT_TRUE(m_pTrack1->trySetBeats(beats));
    const auto anchor = *(beats->cfirstmarker() + 2);
    m_pChannel1->getEngineBuffer()->seekAbs(anchor);
    ProcessBuffer();
    ProcessBuffer();
    ControlObject::set(ConfigKey(m_sGroup1, "beats_set_first_downbeat"), 1);
    ControlObject::set(ConfigKey(m_sGroup1, "beats_set_first_downbeat"), 0);
    ProcessBuffer();
    ProcessBuffer();
    ASSERT_TRUE(m_pTrack1->getBeats()->hasDownbeat());
    EXPECT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup1, "downbeat_valid")));
    EXPECT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup1, "bar_number")));
    EXPECT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup1, "beat_in_bar")));
    m_pChannel1->getEngineBuffer()->seekAbs(*(beats->cfirstmarker() + 7));
    ProcessBuffer();
    ProcessBuffer();
    EXPECT_FRAMEPOS_EQ(*(beats->cfirstmarker() + 7),
            m_pChannel1->getEngineBuffer()->getExactPlayPos());
    EXPECT_DOUBLE_EQ(2, ControlObject::get(ConfigKey(m_sGroup1, "bar_number")));
    EXPECT_DOUBLE_EQ(2, ControlObject::get(ConfigKey(m_sGroup1, "beat_in_bar")));
    ControlObject::set(ConfigKey(m_sGroup1, "beats_clear_first_downbeat"), 1);
    ControlObject::set(ConfigKey(m_sGroup1, "beats_clear_first_downbeat"), 0);
    ProcessBuffer();
    ProcessBuffer();
    EXPECT_FALSE(m_pTrack1->getBeats()->hasDownbeat());
    EXPECT_DOUBLE_EQ(0, ControlObject::get(ConfigKey(m_sGroup1, "downbeat_valid")));
    EXPECT_DOUBLE_EQ(0, ControlObject::get(ConfigKey(m_sGroup1, "bar_number")));
    EXPECT_DOUBLE_EQ(0, ControlObject::get(ConfigKey(m_sGroup1, "beat_in_bar")));
}

TEST_F(BeatsTranslateTest, DownbeatRespectsBpmLock) {
    const auto beats = mixxx::Beats::fromConstTempo(m_pTrack1->getSampleRate(),
            mixxx::audio::kStartFramePos, mixxx::Bpm(120.0));
    ASSERT_TRUE(m_pTrack1->trySetBeats(beats));
    m_pTrack1->setBpmLocked(true);
    ControlObject::set(ConfigKey(m_sGroup1, "beats_set_first_downbeat"), 1);
    ControlObject::set(ConfigKey(m_sGroup1, "beats_set_first_downbeat"), 0);
    EXPECT_FALSE(m_pTrack1->getBeats()->hasDownbeat());
}

TEST_F(BeatsTranslateTest, DownbeatCanBeRestoredWithGridUndo) {
    const auto beats = mixxx::Beats::fromConstTempo(m_pTrack1->getSampleRate(),
            mixxx::audio::kStartFramePos, mixxx::Bpm(120.0));
    ASSERT_TRUE(m_pTrack1->trySetBeats(beats));
    ControlObject::set(ConfigKey(m_sGroup1, "beats_set_first_downbeat"), 1);
    ControlObject::set(ConfigKey(m_sGroup1, "beats_set_first_downbeat"), 0);
    ASSERT_TRUE(m_pTrack1->getBeats()->hasDownbeat());
    QTest::qSleep(810);
    ControlObject::set(ConfigKey(m_sGroup1, "beats_clear_first_downbeat"), 1);
    ControlObject::set(ConfigKey(m_sGroup1, "beats_clear_first_downbeat"), 0);
    ASSERT_FALSE(m_pTrack1->getBeats()->hasDownbeat());
    ControlObject::set(ConfigKey(m_sGroup1, "beats_undo_adjustment"), 1);
    ControlObject::set(ConfigKey(m_sGroup1, "beats_undo_adjustment"), 0);
    EXPECT_TRUE(m_pTrack1->getBeats()->hasDownbeat());
}

namespace {

mixxx::BeatsPointer barGrid(TrackPointer pTrack) {
    return mixxx::Beats::fromConstTempo(pTrack->getSampleRate(),
            mixxx::audio::kStartFramePos,
            mixxx::Bpm(120.0))
            ->withDownbeat(mixxx::audio::kStartFramePos);
}

} // namespace

class BarSyncTest : public BeatsTranslateTest {
  protected:
    void SetUp() override {
        BeatsTranslateTest::SetUp();
        ControlObject::set(ConfigKey(m_sGroup1, "quantize"), 1);
        ControlObject::set(ConfigKey(m_sGroup2, "quantize"), 1);
        ASSERT_TRUE(m_pTrack1->trySetBeats(barGrid(m_pTrack1)));
        ASSERT_TRUE(m_pTrack2->trySetBeats(barGrid(m_pTrack2)));
        m_pBeats = m_pTrack1->getBeats();
        // Deck 1 plays from the start of beat 3 of bar 1.
        m_pChannel1->getEngineBuffer()->seekAbs(*(m_pBeats->cfirstmarker() + 2));
        ProcessBuffer();
        ControlObject::set(ConfigKey(m_sGroup1, "play"), 1);
        ProcessBuffer();
        ASSERT_DOUBLE_EQ(3, ControlObject::get(ConfigKey(m_sGroup1, "beat_in_bar")));
        // Deck 2 is cued on bar 3, beat 1.
        m_pChannel2->getEngineBuffer()->seekAbs(*(m_pBeats->cfirstmarker() + 8));
        ProcessBuffer();
        ASSERT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup2, "beat_in_bar")));
    }

    void playDeck2() {
        ControlObject::set(ConfigKey(m_sGroup2, "play"), 1);
        ProcessBuffer();
        ProcessBuffer();
    }

    mixxx::BeatsPointer m_pBeats;
};

TEST_F(BarSyncTest, PlayAlignsBarsWithOtherDeck) {
    EXPECT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup2, "bar_sync")));
    playDeck2();
    EXPECT_DOUBLE_EQ(ControlObject::get(ConfigKey(m_sGroup1, "beat_in_bar")),
            ControlObject::get(ConfigKey(m_sGroup2, "beat_in_bar")));
    // Moved back two beats, so the cued downbeat meets the next downbeat.
    EXPECT_DOUBLE_EQ(2, ControlObject::get(ConfigKey(m_sGroup2, "bar_number")));
    EXPECT_NEAR(ControlObject::get(ConfigKey(m_sGroup1, "beat_distance")),
            ControlObject::get(ConfigKey(m_sGroup2, "beat_distance")),
            1e-3);
}

TEST_F(BarSyncTest, DisabledBarSyncOnlyMatchesBeats) {
    ControlObject::set(ConfigKey(m_sGroup2, "bar_sync"), 0);
    playDeck2();
    EXPECT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup2, "beat_in_bar")));
    EXPECT_DOUBLE_EQ(3, ControlObject::get(ConfigKey(m_sGroup2, "bar_number")));
}

TEST_F(BarSyncTest, WithoutReferenceOnlyMatchesBeats) {
    ASSERT_TRUE(m_pTrack2->trySetBeats(m_pTrack2->getBeats()->withoutDownbeat()));
    ProcessBuffer();
    playDeck2();
    EXPECT_DOUBLE_EQ(0, ControlObject::get(ConfigKey(m_sGroup2, "downbeat_valid")));
    // Only the beat phase is matched: deck 2 stays near its cued beat.
    EXPECT_NEAR((*(m_pBeats->cfirstmarker() + 8)).value(),
            m_pChannel2->getEngineBuffer()->getExactPlayPos().value(),
            0.25 * (*(m_pBeats->cfirstmarker() + 1) - *m_pBeats->cfirstmarker()));
}

TEST_F(BarSyncTest, SeekWhilePlayingKeepsBeatOnlyQuantize) {
    playDeck2();
    // A quantized jump during playback (e.g. a hotcue) must not move bars.
    m_pChannel2->getEngineBuffer()->seekAbs(*(m_pBeats->cfirstmarker() + 4));
    ProcessBuffer();
    ProcessBuffer();
    EXPECT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup2, "beat_in_bar")));
    EXPECT_DOUBLE_EQ(2, ControlObject::get(ConfigKey(m_sGroup2, "bar_number")));
}

class BarJumpLoopTest : public BeatsTranslateTest {
  protected:
    void SetUp() override {
        BeatsTranslateTest::SetUp();
        ControlObject::set(ConfigKey(m_sGroup1, "quantize"), 0);
        ASSERT_TRUE(m_pTrack1->trySetBeats(barGrid(m_pTrack1)));
        m_pBeats = m_pTrack1->getBeats();
        ControlObject::set(ConfigKey(m_sGroup1, "bar_size"), 1);
    }

    void seekToBeat(int beat, double fraction = 0.25) {
        auto it = m_pBeats->cfirstmarker() + beat;
        const auto position = *it + fraction * (*(it + 1) - *it);
        m_pChannel1->getEngineBuffer()->seekAbs(position);
        ProcessBuffer();
    }

    void press(const char* item) {
        ControlObject::set(ConfigKey(m_sGroup1, item), 1);
        ControlObject::set(ConfigKey(m_sGroup1, item), 0);
        ProcessBuffer();
    }

    double beatPosition(int beat) {
        return (*(m_pBeats->cfirstmarker() + beat)).toEngineSamplePos();
    }

    mixxx::BeatsPointer m_pBeats;
};

TEST_F(BarJumpLoopTest, BarSizeHalvesAndDoublesWithinLimits) {
    ControlObject::set(ConfigKey(m_sGroup1, "bar_size"), 4);
    press("bar_size_halve");
    EXPECT_DOUBLE_EQ(2, ControlObject::get(ConfigKey(m_sGroup1, "bar_size")));
    press("bar_size_halve");
    press("bar_size_halve");
    EXPECT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup1, "bar_size")));
    ControlObject::set(ConfigKey(m_sGroup1, "bar_size"), 32);
    press("bar_size_double");
    press("bar_size_double");
    EXPECT_DOUBLE_EQ(64, ControlObject::get(ConfigKey(m_sGroup1, "bar_size")));
}

TEST_F(BarJumpLoopTest, BarJumpKeepsBeatInBar) {
    seekToBeat(2);
    ASSERT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup1, "bar_number")));
    ASSERT_DOUBLE_EQ(3, ControlObject::get(ConfigKey(m_sGroup1, "beat_in_bar")));
    press("bar_jump_forward");
    ProcessBuffer();
    EXPECT_DOUBLE_EQ(2, ControlObject::get(ConfigKey(m_sGroup1, "bar_number")));
    EXPECT_DOUBLE_EQ(3, ControlObject::get(ConfigKey(m_sGroup1, "beat_in_bar")));
    press("bar_jump_backward");
    ProcessBuffer();
    EXPECT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup1, "bar_number")));
    EXPECT_DOUBLE_EQ(3, ControlObject::get(ConfigKey(m_sGroup1, "beat_in_bar")));
}

TEST_F(BarJumpLoopTest, BarLoopStartsOnBeatOneOfCurrentBar) {
    seekToBeat(6);
    press("bar_loop_toggle");
    EXPECT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup1, "loop_enabled")));
    EXPECT_DOUBLE_EQ(beatPosition(4), ControlObject::get(ConfigKey(m_sGroup1, "loop_start_position")));
    EXPECT_DOUBLE_EQ(beatPosition(8), ControlObject::get(ConfigKey(m_sGroup1, "loop_end_position")));
    press("bar_loop_toggle");
    EXPECT_DOUBLE_EQ(0, ControlObject::get(ConfigKey(m_sGroup1, "loop_enabled")));
}

TEST_F(BarJumpLoopTest, BarLoopWithoutReferenceStartsOnCurrentBeat) {
    ASSERT_TRUE(m_pTrack1->trySetBeats(m_pBeats->withoutDownbeat()));
    ProcessBuffer();
    seekToBeat(6);
    press("bar_loop_toggle");
    EXPECT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup1, "loop_enabled")));
    EXPECT_DOUBLE_EQ(beatPosition(6), ControlObject::get(ConfigKey(m_sGroup1, "loop_start_position")));
    EXPECT_DOUBLE_EQ(beatPosition(10), ControlObject::get(ConfigKey(m_sGroup1, "loop_end_position")));
}

// Every control used by the GRID panel in the RbxFlowBars skin.
class GridPanelTest : public BeatsTranslateTest,
                      public ::testing::WithParamInterface<bool> {
  protected:
    void SetUp() override {
        BeatsTranslateTest::SetUp();
        ControlObject::set(ConfigKey(m_sGroup1, "quantize"), 0);
        auto beats = mixxx::Beats::fromConstTempo(m_pTrack1->getSampleRate(),
                mixxx::audio::kStartFramePos,
                mixxx::Bpm(120.0));
        if (GetParam()) {
            beats = beats->withDownbeat(mixxx::audio::kStartFramePos);
        }
        ASSERT_TRUE(m_pTrack1->trySetBeats(beats));
        ProcessBuffer();
    }

    void press(const char* item) {
        // Space presses apart so each one gets its own undo entry.
        QTest::qSleep(810);
        ControlObject::set(ConfigKey(m_sGroup1, item), 1);
        ControlObject::set(ConfigKey(m_sGroup1, item), 0);
        ProcessBuffer();
    }

    mixxx::audio::FramePos firstBeat() {
        return *m_pTrack1->getBeats()->cfirstmarker();
    }
    double fileBpm() {
        return ControlObject::get(ConfigKey(m_sGroup1, "file_bpm"));
    }
    bool hasDownbeat() {
        return m_pTrack1->getBeats()->hasDownbeat();
    }
};

TEST_P(GridPanelTest, AllButtonsEditTheGrid) {
    const auto origin = firstBeat();
    EXPECT_DOUBLE_EQ(120, fileBpm());

    press("beats_translate_later");
    const auto later = firstBeat();
    EXPECT_GT(later, origin);
    press("beats_translate_earlier");
    EXPECT_LT(firstBeat(), later);

    press("beats_adjust_faster");
    EXPECT_DOUBLE_EQ(120.01, fileBpm());
    press("beats_adjust_slower");
    press("beats_adjust_slower");
    EXPECT_DOUBLE_EQ(119.99, fileBpm());

    // GRID: nearest beat moves to the play position.
    const auto seek = mixxx::audio::FramePos(origin.value() + 3000);
    m_pChannel1->getEngineBuffer()->seekAbs(seek);
    ProcessBuffer();
    const auto playPos = m_pChannel1->getEngineBuffer()->getExactPlayPos();
    press("beats_translate_curpos");
    EXPECT_NEAR(playPos.value(),
            m_pTrack1->getBeats()->findClosestBeat(playPos).value(),
            1.0);

    press("beats_undo_adjustment");
    EXPECT_DOUBLE_EQ(119.99, fileBpm());
    EXPECT_EQ(GetParam(), hasDownbeat());

    // LOCK blocks edits, unlocking allows them again.
    // LOCK is a toggle: the skin button writes the new state.
    ControlObject::set(ConfigKey(m_sGroup1, "bpmlock"), 1);
    ProcessBuffer();
    EXPECT_TRUE(m_pTrack1->isBpmLocked());
    const auto locked = firstBeat();
    press("beats_translate_later");
    press("beats_adjust_faster");
    EXPECT_EQ(locked, firstBeat());
    EXPECT_DOUBLE_EQ(119.99, fileBpm());
    ControlObject::set(ConfigKey(m_sGroup1, "bpmlock"), 0);
    ProcessBuffer();
    EXPECT_FALSE(m_pTrack1->isBpmLocked());
    press("beats_adjust_faster");
    EXPECT_DOUBLE_EQ(120, fileBpm());
    EXPECT_EQ(GetParam(), hasDownbeat());
}

TEST_P(GridPanelTest, TapSetsFileBpm) {
    for (int i = 0; i < 8; ++i) {
        ControlObject::set(ConfigKey(m_sGroup1, "bpm_tap"), 1);
        ControlObject::set(ConfigKey(m_sGroup1, "bpm_tap"), 0);
        QTest::qSleep(500); // 120 BPM taps
    }
    ProcessBuffer();
    EXPECT_NEAR(120, fileBpm(), 2);
    EXPECT_EQ(GetParam(), hasDownbeat());
}

INSTANTIATE_TEST_SUITE_P(WithAndWithoutBar1, GridPanelTest, ::testing::Bool());

class AutoDownbeatTest : public BeatsTranslateTest {
  protected:
    // Loads a new track into deck 1; like a real load, the analysis
    // delivers the grid (and the main cue) afterwards.
    TrackPointer loadTrackWithGrid(mixxx::BeatsPointer pBeats,
            double cueFrames = -1,
            bool locked = false) {
        TrackPointer pTrack = m_pMixerDeck1->loadFakeTrack(false, 0.0);
        ProcessBuffer();
        if (cueFrames >= 0) {
            pTrack->setMainCuePosition(mixxx::audio::FramePos(cueFrames));
        }
        pTrack->setBpmLocked(locked);
        if (pBeats) {
            EXPECT_TRUE(pTrack->trySetBeats(pBeats));
        }
        ProcessBuffer();
        return pTrack;
    }
    mixxx::BeatsPointer grid() {
        return mixxx::Beats::fromConstTempo(mixxx::audio::SampleRate(44100),
                mixxx::audio::FramePos(1000),
                mixxx::Bpm(120.0));
    }
    static constexpr double kBeatFrames = 22050;
    void press(const char* item) {
        ControlObject::set(ConfigKey(m_sGroup1, item), 1);
        ControlObject::set(ConfigKey(m_sGroup1, item), 0);
        ProcessBuffer();
    }
};

TEST_F(AutoDownbeatTest, MarksBarOneOnTheMainCue) {
    ControlObject::set(ConfigKey(m_sGroup1, "auto_first_downbeat"), 1);
    // Cue a little after the third beat.
    const auto pTrack = loadTrackWithGrid(grid(), 1000 + 2 * kBeatFrames + 50);
    auto pBeats = pTrack->getBeats();
    ASSERT_TRUE(pBeats->hasDownbeat());
    const auto bar = pBeats->barPosition(*(pBeats->cfirstmarker() + 2));
    ASSERT_TRUE(bar);
    EXPECT_EQ(1, bar->bar);
    EXPECT_EQ(1, bar->beat);
    EXPECT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup1, "downbeat_valid")));
}

TEST_F(AutoDownbeatTest, WithoutCueUsesFirstBeat) {
    ControlObject::set(ConfigKey(m_sGroup1, "auto_first_downbeat"), 1);
    const auto pTrack = loadTrackWithGrid(grid());
    auto pBeats = pTrack->getBeats();
    ASSERT_TRUE(pBeats->hasDownbeat());
    const auto bar = pBeats->barPosition(*pBeats->cfirstmarker());
    ASSERT_TRUE(bar);
    EXPECT_EQ(1, bar->bar);
    EXPECT_EQ(1, bar->beat);
}

TEST_F(AutoDownbeatTest, KeepsExistingReference) {
    ControlObject::set(ConfigKey(m_sGroup1, "auto_first_downbeat"), 1);
    const auto pTrack = loadTrackWithGrid(
            grid()->withDownbeat(*(grid()->cfirstmarker() + 1)), 1000);
    auto pBeats = pTrack->getBeats();
    const auto bar = pBeats->barPosition(*(pBeats->cfirstmarker() + 1));
    ASSERT_TRUE(bar);
    EXPECT_EQ(1, bar->bar);
    EXPECT_EQ(1, bar->beat);
}

TEST_F(AutoDownbeatTest, SkipsLockedTracks) {
    ControlObject::set(ConfigKey(m_sGroup1, "auto_first_downbeat"), 1);
    // A library track that already has a locked grid when it is loaded.
    TrackPointer pTrack = Track::newTemporary();
    pTrack->setAudioProperties(mixxx::kEngineChannelCount,
            mixxx::audio::SampleRate(44100),
            mixxx::audio::Bitrate(),
            mixxx::Duration::fromSeconds(10));
    ASSERT_TRUE(pTrack->trySetBeats(grid()));
    pTrack->setBpmLocked(true);
    ASSERT_TRUE(pTrack->isBpmLocked());
    m_pChannel1->getEngineBuffer()->loadFakeTrack(pTrack, false);
    ProcessBuffer();
    ProcessBuffer();
    EXPECT_TRUE(pTrack->isBpmLocked());
    EXPECT_FALSE(pTrack->getBeats()->hasDownbeat());
}

TEST_F(AutoDownbeatTest, ClearingByHandSticks) {
    ControlObject::set(ConfigKey(m_sGroup1, "auto_first_downbeat"), 1);
    const auto pTrack = loadTrackWithGrid(grid(), 1000);
    ASSERT_TRUE(pTrack->getBeats()->hasDownbeat());
    press("beats_clear_first_downbeat");
    EXPECT_FALSE(pTrack->getBeats()->hasDownbeat());
    // Later grid edits must not bring it back.
    press("beats_translate_later");
    EXPECT_FALSE(pTrack->getBeats()->hasDownbeat());
}

TEST_F(AutoDownbeatTest, CanBeSwitchedOff) {
    ControlObject::set(ConfigKey(m_sGroup1, "auto_first_downbeat"), 0);
    const auto pTrack = loadTrackWithGrid(grid(), 1000);
    EXPECT_FALSE(pTrack->getBeats()->hasDownbeat());
}

TEST_F(AutoDownbeatTest, MarksAnalyzedLibraryTrackOnLoad) {
    ControlObject::set(ConfigKey(m_sGroup1, "auto_first_downbeat"), 1);
    // Grid and cue already stored, as for a track analyzed earlier.
    TrackPointer pTrack = Track::newTemporary();
    pTrack->setAudioProperties(mixxx::kEngineChannelCount,
            mixxx::audio::SampleRate(44100),
            mixxx::audio::Bitrate(),
            mixxx::Duration::fromSeconds(10));
    ASSERT_TRUE(pTrack->trySetBeats(grid()));
    pTrack->setMainCuePosition(mixxx::audio::FramePos(1000 + kBeatFrames));
    ASSERT_FALSE(pTrack->getBeats()->hasDownbeat());
    m_pChannel1->getEngineBuffer()->loadFakeTrack(pTrack, false);
    ProcessBuffer();
    ProcessBuffer();
    auto pBeats = pTrack->getBeats();
    ASSERT_TRUE(pBeats->hasDownbeat());
    const auto bar = pBeats->barPosition(*(pBeats->cfirstmarker() + 1));
    ASSERT_TRUE(bar);
    EXPECT_EQ(1, bar->bar);
    EXPECT_EQ(1, bar->beat);
    EXPECT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup1, "downbeat_valid")));
}

#include "analyzer/phraseanalysisstore.h"
#include "track/cue.h"
TEST_F(BarSyncTest, PhraseSyncMatchesRemainingPhraseBeatsOnPlay) {
    vilelabox::PhraseFeature f;
    f.start = 0; f.end = 128; f.rms = 1; f.bass = 1;
    for (const auto& track : {m_pTrack1, m_pTrack2}) {
        vilelabox::PhraseAnalysis analysis; analysis.features = {f}; analysis.duration = 128;
        PhraseAnalysisStore::instance().set(track, analysis, {});
    }
    ControlObject::set(ConfigKey(m_sGroup2, "phrase_sync"), 1);
    playDeck2();
    EXPECT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup2, "bar_number")));
    EXPECT_DOUBLE_EQ(ControlObject::get(ConfigKey(m_sGroup1, "beat_in_bar")), ControlObject::get(ConfigKey(m_sGroup2, "beat_in_bar")));
    EXPECT_DOUBLE_EQ(ControlObject::get(ConfigKey(m_sGroup1, "phrase_bars_left")), ControlObject::get(ConfigKey(m_sGroup2, "phrase_bars_left")));
}
TEST_F(BarSyncTest, MixPointArmsLaunchesAndCancelsOnStop) {
    const auto trigger = *(m_pBeats->cfirstmarker() + 4);
    m_pTrack1->createAndAddCue(mixxx::CueType::Outro, Cue::kNoHotCue, trigger, *(m_pBeats->cfirstmarker() + 8));
    m_pTrack2->createAndAddCue(mixxx::CueType::Intro, Cue::kNoHotCue, mixxx::audio::kStartFramePos, *(m_pBeats->cfirstmarker() + 4));
    ProcessBuffer();
    ControlObject::set(ConfigKey(m_sGroup2, "mix_point_link"), 1);
    ControlObject::set(ConfigKey(m_sGroup2, "mix_point_link"), 0);
    ASSERT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup2, "mix_point_link_armed")));
    for (int i = 0; i < 500 && !ControlObject::get(ConfigKey(m_sGroup2, "play")); ++i) ProcessBuffer();
    EXPECT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup2, "play")));
    EXPECT_GE(m_pChannel1->getEngineBuffer()->getExactPlayPos(), trigger);
    EXPECT_DOUBLE_EQ(0, ControlObject::get(ConfigKey(m_sGroup2, "mix_point_link_armed")));
}
TEST_F(BarSyncTest, MixPointRejectsMissingExitAndDifferentTempo) {
    ControlObject::set(ConfigKey(m_sGroup2, "mix_point_link"), 1);
    ControlObject::set(ConfigKey(m_sGroup2, "mix_point_link"), 0);
    EXPECT_DOUBLE_EQ(0, ControlObject::get(ConfigKey(m_sGroup2, "mix_point_link_armed")));
    m_pTrack1->createAndAddCue(mixxx::CueType::Outro, Cue::kNoHotCue, *(m_pBeats->cfirstmarker() + 4), mixxx::audio::kInvalidFramePos);
    m_pTrack2->trySetBeats(mixxx::Beats::fromConstTempo(m_pTrack2->getSampleRate(), mixxx::audio::kStartFramePos, mixxx::Bpm(130)));
    ProcessBuffer();
    ControlObject::set(ConfigKey(m_sGroup2, "mix_point_link"), 1);
    EXPECT_DOUBLE_EQ(0, ControlObject::get(ConfigKey(m_sGroup2, "mix_point_link_armed")));
    EXPECT_DOUBLE_EQ(3, ControlObject::get(ConfigKey(m_sGroup2, "mix_point_error")));
}
TEST_F(BarSyncTest, MixPointCancelsEvenSmallSourceSeeksAndStops) {
    m_pTrack1->createAndAddCue(mixxx::CueType::Outro, Cue::kNoHotCue, *(m_pBeats->cfirstmarker() + 8), mixxx::audio::kInvalidFramePos);
    m_pTrack2->createAndAddCue(mixxx::CueType::Intro, Cue::kNoHotCue, mixxx::audio::kStartFramePos, mixxx::audio::kInvalidFramePos);
    ProcessBuffer();
    auto arm = [this] { ControlObject::set(ConfigKey(m_sGroup2, "mix_point_link"), 1); ControlObject::set(ConfigKey(m_sGroup2, "mix_point_link"), 0); };
    arm(); ASSERT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup2, "mix_point_link_armed")));
    m_pChannel1->getEngineBuffer()->queueNewPlaypos(m_pChannel1->getEngineBuffer()->getExactPlayPos() + 1, EngineBuffer::SEEK_EXACT);
    ProcessBuffer(); ProcessBuffer();
    EXPECT_DOUBLE_EQ(0, ControlObject::get(ConfigKey(m_sGroup2, "mix_point_link_armed")));
    arm(); ASSERT_DOUBLE_EQ(1, ControlObject::get(ConfigKey(m_sGroup2, "mix_point_link_armed")));
    ControlObject::set(ConfigKey(m_sGroup1, "play"), 0); ProcessBuffer();
    EXPECT_DOUBLE_EQ(0, ControlObject::get(ConfigKey(m_sGroup2, "mix_point_link_armed")));
}
