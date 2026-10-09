#include "engine/controls/padcontrol.h"
#include <algorithm>
#include <cmath>
#include "control/controlobject.h"
#include "control/controlproxy.h"
#include "control/controlpushbutton.h"
#include "engine/enginebuffer.h"
#include "track/track.h"
#include "moc_padcontrol.cpp"
PadControl::PadControl(const QString& group, UserSettingsPointer config) : EngineControl(group, config) {
    m_slip = std::make_unique<ControlProxy>(group, "slip_enabled", this);
    m_loopEnabled = std::make_unique<ControlProxy>(group, "loop_enabled", this);
    m_keylock = std::make_unique<ControlProxy>(group, "keylock", this);
    m_pitch = std::make_unique<ControlProxy>(group, "pitch_adjust", this);
    m_play = std::make_unique<ControlProxy>(group, "play", this);
    m_bpm = std::make_unique<ControlProxy>(group, "bpm", this);
    m_enable = std::make_unique<ControlPushButton>(ConfigKey(group, "slicer_enable"));
    m_enable->setButtonMode(ControlPushButton::TOGGLE);
    m_loop = std::make_unique<ControlPushButton>(ConfigKey(group, "slicer_loop"));
    m_loop->setButtonMode(ControlPushButton::TOGGLE);
    connect(m_enable.get(), &ControlObject::valueChanged, this, [this](double v) { if (v == 0) leaveSlice(); }, Qt::DirectConnection);
    m_keyboardCue = std::make_unique<ControlObject>(ConfigKey(group, "keyboard_hotcue"));
    m_keyboardCue->set(1);
    for (int i = 0; i < 8; ++i) {
        m_slices[i] = std::make_unique<ControlPushButton>(ConfigKey(group, QString("slicer_slice_%1").arg(i + 1)));
        connect(m_slices[i].get(), &ControlObject::valueChanged, this, [this, i](double v) { slice(i, v > 0); }, Qt::DirectConnection);
    }
    for (int i = 0; i < 13; ++i) {
        m_keys[i] = std::make_unique<ControlPushButton>(ConfigKey(group, QString("keyshift_%1").arg(i - 6)));
        m_keyboard[i] = std::make_unique<ControlPushButton>(ConfigKey(group, QString("keyboard_%1").arg(i - 6)));
        auto transpose = [this, i](double v) { if (v > 0) { m_keylock->set(1); m_pitch->set(i - 6); } };
        connect(m_keys[i].get(), &ControlObject::valueChanged, this, transpose, Qt::DirectConnection);
        connect(m_keyboard[i].get(), &ControlObject::valueChanged, this, [this, group, transpose](double v) {
            transpose(v);
            const int cue = static_cast<int>(m_keyboardCue->get());
            if (v > 0 && cue >= 1 && cue <= 8) {
                const auto key = ConfigKey(group, QString("hotcue_%1_goto").arg(cue));
                ControlObject::set(key, 1); ControlObject::set(key, 0);
            }
        }, Qt::DirectConnection);
    }
    m_release = std::make_unique<ControlPushButton>(ConfigKey(group, "release_fx"));
    m_stop = std::make_unique<ControlPushButton>(ConfigKey(group, "release_fx_stop"), true, 0.0);
    m_stop->setButtonMode(ControlPushButton::TOGGLE);
    connect(m_release.get(), &ControlObject::valueChanged, this, [this](double v) { if (v > 0) releaseFx(); }, Qt::DirectConnection);
}
PadControl::~PadControl() = default;
void PadControl::trackLoaded(TrackPointer track) {
    m_active = -1; m_window = {}; m_beats = track ? track->getBeats() : nullptr;
    m_enable->set(0); m_loop->set(0); m_releaseSamples = 0; m_releaseTotal = 0;
    m_outputRate = std::max(8000, static_cast<int>(ControlObject::get(ConfigKey("[App]", "samplerate"))));
    m_delay.assign(m_outputRate * 8 * 2, 0); m_delayWrite = 0;
}
void PadControl::leaveSlice() {
    if (m_active < 0) return;
    if (m_beats && m_window.isValid()) {
        auto it = m_beats->iteratorFrom(m_window);
        setLoop(*(it + m_active), *(it + m_active + 1), false);
    }
    m_slip->set(0); m_active = -1;
}
void PadControl::slice(int index, bool pressed) {
    if (!pressed) { if (m_active == index) leaveSlice(); return; }
    if (!m_enable->toBool() || !m_beats || !m_window.isValid() ||
            (m_active < 0 && (m_slip->toBool() || m_loopEnabled->toBool()))) return;
    auto it = m_beats->iteratorFrom(m_window);
    const auto start = *(it + index), end = *(it + index + 1);
    if (start < mixxx::audio::kStartFramePos || end > frameInfo().trackEndPosition) return;
    if (m_active < 0) m_slip->set(1);
    setLoop(start, end, true); seekExact(start); m_active = index;
}
void PadControl::process(double rate, mixxx::audio::FramePos position, int bufferSize) {
    Q_UNUSED(rate); Q_UNUSED(bufferSize);
    if (!m_enable->toBool() || !m_beats || m_active >= 0) return;
    if (m_loop->toBool() && m_window.isValid()) {
        auto it = m_beats->iteratorFrom(m_window);
        if (position >= *(it + 8)) seekExact(m_window);
        return;
    }
    const auto bar = m_beats->barPosition(position);
    if (!bar) { m_window = {}; return; }
    auto it = m_beats->iteratorFrom(position);
    if (*it > position) --it;
    const int offset = ((bar->bar - 1) % 2 + 2) % 2 * 4 + bar->beat - 1;
    m_window = *(it - offset);
}
void PadControl::releaseFx() {
    if (!m_play->toBool() || m_bpm->get() <= 0 || m_delay.empty()) return;
    for (int i = 1; i <= 4; ++i) {
        const auto route = ControlObject::getControl(ConfigKey(QString("[EffectRack1_EffectUnit%1]").arg(i),
                QString("group_%1_enable").arg(getGroup())), ControlFlag::NoWarnIfMissing);
        if (route) route->set(0);
    }
    m_releaseTotal = std::llround(m_outputRate * 240 / m_bpm->get()) * 2;
    m_releaseSamples = m_releaseTotal;
}
void PadControl::processAudio(CSAMPLE* samples, int count) {
    if (m_delay.empty()) return;
    const size_t beatSamples = std::min(m_delay.size() - 2,
            std::max(size_t(2), static_cast<size_t>(m_outputRate * 60 / std::max(30.0, m_bpm->get())) * 2));
    for (int i = 0; i < count; ++i) {
        if (m_releaseSamples > 0) {
            const float echo = m_delay[(m_delayWrite + m_delay.size() - beatSamples) % m_delay.size()];
            m_delay[m_delayWrite] = echo * 0.6f;
            samples[i] = echo * static_cast<float>(m_releaseSamples) / m_releaseTotal;
            if (--m_releaseSamples == 0 && m_stop->toBool()) {
                m_play->set(0);
                std::fill(samples + i + 1, samples + count, 0);
                break;
            }
        } else m_delay[m_delayWrite] = samples[i];
        m_delayWrite = (m_delayWrite + 1) % m_delay.size();
    }
}
