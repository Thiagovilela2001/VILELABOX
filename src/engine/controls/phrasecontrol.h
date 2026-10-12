#pragma once
#include <atomic>
#include <memory>
#include <vector>
#include "engine/controls/enginecontrol.h"
#include "track/phrases.h"
#include "track/autophrasecues.h"
class ControlObject;
class ControlPushButton;
class PhraseControl : public EngineControl {
    Q_OBJECT
  public:
    PhraseControl(const QString& group, UserSettingsPointer config);
    ~PhraseControl() override;
    // Audio callback: only records which track is loaded.
    void trackLoaded(TrackPointer track) override;
    void process(double rate, mixxx::audio::FramePos position, int bufferSize) override;
    /// GUI thread, after the engine loaded or ejected a track.
    void trackLoadedOnGuiThread();
    /// GUI thread only.
    std::shared_ptr<const vilelabox::PhraseAnalysis> analysis() const;
    /// Engine thread only. Valid until the next call on this control from the
    /// engine thread; never owned, so the audio thread never frees it.
    const vilelabox::PhraseAnalysis* analysisForAudio() const;
  private:
    struct State {
        const Track* track = nullptr; // identity only, never dereferenced
        QString location;
        mixxx::BeatsPointer beats;
        std::shared_ptr<const vilelabox::PhraseAnalysis> analysis;
    };
    void jump(bool next);
    // GUI thread: builds the state of the loaded track and publishes it.
    void rebuild();
    void publish(std::unique_ptr<const State> state);
    void applyAutoCues(const TrackPointer& track);
    // Hazard-pointer read for the engine thread.
    const State* acquireForAudio() const;

    // Published state (GUI writes, audio reads) and the pointer the audio
    // thread is using; together they let the GUI thread free old states safely.
    std::atomic<const State*> m_current{nullptr};
    mutable std::atomic<const State*> m_hazard{nullptr};
    // The track the engine has loaded; a state for another track is ignored.
    std::atomic<const Track*> m_audioTrack{nullptr};
    // GUI thread only.
    std::unique_ptr<const State> m_owned;
    std::vector<std::unique_ptr<const State>> m_retired;
    const Track* m_cueTrack = nullptr;
    // Set once cues were generated from a usable analysis of the loaded track,
    // so later grid edits (which recalculate phrases) do not add cues again.
    bool m_cuesApplied = false;

    QMetaObject::Connection m_connection;
    std::unique_ptr<ControlObject> m_type, m_index, m_bars, m_valid;
    std::unique_ptr<ControlPushButton> m_next, m_previous, m_autoCues, m_undoCues;
    AutoPhraseCues m_cueGenerator;
};
