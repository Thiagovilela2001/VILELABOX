#pragma once
#include "engine/controls/enginecontrol.h"
#include <memory>
class ControlObject;
class ControlProxy;
class ControlPushButton;
class MixPointControl : public EngineControl {
    Q_OBJECT
  public:
    MixPointControl(const QString& group, UserSettingsPointer config);
    ~MixPointControl() override;
    void trackLoaded(TrackPointer track) override;
    void process(double rate, mixxx::audio::FramePos position, int bufferSize) override;
    void notifySeek(mixxx::audio::FramePos position) override;
  private:
    void arm(bool silent);
    void cancel(int error = 0);
    void select(int direction);
    void refreshPoints();
    std::unique_ptr<ControlPushButton> m_link, m_silent, m_exit, m_next, m_previous;
    std::unique_ptr<ControlObject> m_armed, m_left, m_error, m_in, m_out;
    std::unique_ptr<ControlProxy> m_play;
    EngineBuffer* m_source = nullptr;
    TrackWeakPointer m_sourceTrack;
    mixxx::audio::FramePos m_previousPosition, m_trigger, m_entry;
    int m_sourceSeekGeneration = 0;
    bool m_silentLaunch = false;
};
