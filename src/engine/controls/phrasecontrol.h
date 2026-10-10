#pragma once
#include <atomic>
#include <memory>
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
    void trackLoaded(TrackPointer track) override;
    void process(double rate, mixxx::audio::FramePos position, int bufferSize) override;
    std::shared_ptr<const vilelabox::PhraseAnalysis> analysis() const;
  private:
    void jump(bool next);
    // Runs on the GUI thread: cue creation locks the track and allocates.
    void applyAutoCues(const TrackWeakPointer& weak);
    struct State {
        QString location;
        mixxx::BeatsPointer beats;
        std::shared_ptr<const vilelabox::PhraseAnalysis> analysis;
    };
    std::shared_ptr<const State> m_state;
    QMetaObject::Connection m_connection;
    std::unique_ptr<ControlObject> m_type, m_index, m_bars, m_valid;
    std::unique_ptr<ControlPushButton> m_next, m_previous, m_autoCues, m_undoCues;
    AutoPhraseCues m_cueGenerator;
    // Set once cues were generated from a usable analysis of the loaded track,
    // so later grid edits (which recalculate phrases) do not add cues again.
    std::atomic<bool> m_cuesApplied{false};
};
