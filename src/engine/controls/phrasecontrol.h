#pragma once
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
};
