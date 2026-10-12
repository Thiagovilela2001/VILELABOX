#pragma once
#include "engine/controls/enginecontrol.h"
#include <atomic>
#include <map>
#include <memory>
class ControlObject;
class ControlProxy;
class ControlPushButton;
class MixPointControl : public EngineControl {
    Q_OBJECT
  public:
    MixPointControl(const QString& group, UserSettingsPointer config);
    ~MixPointControl() override;
    // Audio callback: cancels and resets; points are computed on the GUI thread.
    void trackLoaded(TrackPointer track) override;
    /// GUI thread, after the engine loaded or ejected a track.
    void trackLoadedOnGuiThread();
    void process(double rate, mixxx::audio::FramePos position, int bufferSize) override;
    void notifySeek(mixxx::audio::FramePos position) override;
  private:
    void arm(bool silent);
    void cancel(int error = 0);
    void select(int direction);
    // GUI thread only: reads cues and phrases, which lock the track.
    void refreshPoints();
    std::unique_ptr<ControlPushButton> m_link, m_silent, m_exit, m_next, m_previous;
    std::unique_ptr<ControlObject> m_armed, m_left, m_error, m_in, m_out;
    std::unique_ptr<ControlProxy> m_play, m_bpm, m_volume;
    // Proxies of each source deck, created in arm() and never destroyed while
    // this control exists, so process() can never read a deleted proxy.
    struct SourceProxies {
        std::unique_ptr<ControlProxy> play, bpm;
    };
    std::map<QString, SourceProxies> m_sourceProxies;
    ControlProxy* m_sourcePlay = nullptr;
    ControlProxy* m_sourceBpm = nullptr;
    // Written last in arm() (release) and read first in process() (acquire).
    std::atomic<EngineBuffer*> m_source{nullptr};
    TrackWeakPointer m_sourceTrack;
    // Captured in arm() (GUI thread) before m_source is published.
    mixxx::BeatsPointer m_sourceBeats, m_ownBeats;
    QMetaObject::Connection m_cuesConnection, m_phrasesConnection;
    mixxx::audio::FramePos m_previousPosition, m_trigger, m_entry;
    int m_sourceSeekGeneration = 0;
    bool m_silentLaunch = false;
    // Points picked with select_next/prev survive cue and phrase updates.
    bool m_inSelected = false, m_outSelected = false;
};
