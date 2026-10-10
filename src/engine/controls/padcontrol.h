#pragma once
#include <array>
#include <memory>
#include <vector>
#include "engine/controls/enginecontrol.h"
class ControlObject;
class ControlProxy;
class ControlPushButton;
class PadControl : public EngineControl {
    Q_OBJECT
  public:
    PadControl(const QString& group, UserSettingsPointer config);
    ~PadControl() override;
    void trackLoaded(TrackPointer track) override;
    void trackBeatsUpdated(mixxx::BeatsPointer beats) override { m_beats = beats; }
    void process(double rate, mixxx::audio::FramePos position, int bufferSize) override;
    void processAudio(CSAMPLE* samples, int count);
    const CSAMPLE* echoBufferForTesting() const { return m_delay.data(); }
  private:
    void slice(int index, bool pressed);
    void leaveSlice();
    void releaseFx();
    std::unique_ptr<ControlPushButton> m_enable, m_loop, m_release, m_stop;
    std::array<std::unique_ptr<ControlPushButton>, 8> m_slices;
    std::array<std::unique_ptr<ControlPushButton>, 13> m_keys, m_keyboard;
    std::unique_ptr<ControlObject> m_keyboardCue;
    std::unique_ptr<ControlProxy> m_slip, m_loopEnabled, m_keylock, m_pitch, m_play, m_bpm, m_sampleRate;
    int m_active = -1;
    mixxx::audio::FramePos m_window;
    mixxx::BeatsPointer m_beats;
    // Allocated once in the constructor (decks only): trackLoaded() and
    // processAudio() run in the audio callback and must not allocate.
    std::vector<CSAMPLE> m_delay;
    size_t m_delayWrite = 0;
    qint64 m_releaseSamples = 0, m_releaseTotal = 0;
};
