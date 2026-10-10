#pragma once
#include <array>
#include <atomic>
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
    // Audio callback: ends the release (stop or fade back in).
    void finishRelease(CSAMPLE* samples, int from, int count);
    // Release FX types, as in rekordbox: BREAK (vinyl brake), ECHO, SPIN.
    enum ReleaseType { kBreak = 0, kEcho = 1, kSpin = 2 };
    std::unique_ptr<ControlPushButton> m_enable, m_loop, m_release, m_stop;
    std::unique_ptr<ControlPushButton> m_releaseTypeButtons[3], m_releaseBeatsHalve, m_releaseBeatsDouble;
    std::unique_ptr<ControlObject> m_releaseType, m_releaseBeats, m_releaseActive;
    std::array<std::unique_ptr<ControlPushButton>, 8> m_slices;
    std::array<std::unique_ptr<ControlPushButton>, 13> m_keys, m_keyboard;
    std::unique_ptr<ControlObject> m_keyboardCue;
    std::unique_ptr<ControlProxy> m_slip, m_loopEnabled, m_keylock, m_pitch, m_play, m_bpm, m_sampleRate,
            m_scratch2, m_scratch2Enable;
    int m_active = -1;
    mixxx::audio::FramePos m_window;
    mixxx::BeatsPointer m_beats;
    // Allocated once in the constructor (decks only): trackLoaded() and
    // processAudio() run in the audio callback and must not allocate.
    std::vector<CSAMPLE> m_delay;
    size_t m_delayWrite = 0;
    // Written by releaseFx() (GUI thread), consumed by the audio callback.
    std::atomic<qint64> m_releaseSamples{0}, m_releaseTotal{0};
    std::atomic<int> m_activeType{kEcho};
    // Audio callback only.
    bool m_scratching = false;
    double m_startRate = 1;
    qint64 m_fadeSamples = 0, m_fadeTotal = 0;
};
