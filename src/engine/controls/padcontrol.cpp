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
    m_sampleRate = std::make_unique<ControlProxy>("[App]", "samplerate", this);
    m_scratch2 = std::make_unique<ControlProxy>(group, "scratch2", this);
    m_scratch2Enable = std::make_unique<ControlProxy>(group, "scratch2_enable", this);
    if (group.startsWith(QStringLiteral("[Channel"))) {
        // One beat at 30 BPM (2 s) at the highest supported output rate, stereo.
        constexpr size_t kMaxRate = 192000;
        m_delay.assign(kMaxRate * 21 / 10 * 2, 0);
    }
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
    // The three release types are exit effects; by default the deck stops.
    m_stop = std::make_unique<ControlPushButton>(ConfigKey(group, "release_fx_stop"), true, 1.0);
    m_stop->setButtonMode(ControlPushButton::TOGGLE);
    connect(m_release.get(), &ControlObject::valueChanged, this, [this](double v) { if (v > 0) releaseFx(); }, Qt::DirectConnection);
    m_releaseType = std::make_unique<ControlObject>(ConfigKey(group, "release_fx_type"), true, false, true, kEcho);
    m_releaseBeats = std::make_unique<ControlObject>(ConfigKey(group, "release_fx_beats"), true, false, true, 4.0);
    m_releaseActive = std::make_unique<ControlObject>(ConfigKey(group, "release_fx_active"));
    m_releaseActive->setReadOnly();
    static const char* typeNames[]{"release_fx_break", "release_fx_echo", "release_fx_spin"};
    for (int type = 0; type < 3; ++type) {
        m_releaseTypeButtons[type] = std::make_unique<ControlPushButton>(ConfigKey(group, typeNames[type]));
        connect(m_releaseTypeButtons[type].get(), &ControlObject::valueChanged, this,
                [this, type](double v) { if (v > 0) m_releaseType->set(type); }, Qt::DirectConnection);
    }
    // Adjust Time: 1 to 16 beats in powers of two.
    m_releaseBeatsHalve = std::make_unique<ControlPushButton>(ConfigKey(group, "release_fx_beats_halve"));
    m_releaseBeatsDouble = std::make_unique<ControlPushButton>(ConfigKey(group, "release_fx_beats_double"));
    connect(m_releaseBeatsHalve.get(), &ControlObject::valueChanged, this, [this](double v) {
        if (v > 0) m_releaseBeats->set(std::max(1.0, m_releaseBeats->get() / 2));
    }, Qt::DirectConnection);
    connect(m_releaseBeatsDouble.get(), &ControlObject::valueChanged, this, [this](double v) {
        if (v > 0) m_releaseBeats->set(std::min(16.0, m_releaseBeats->get() * 2));
    }, Qt::DirectConnection);
}
PadControl::~PadControl() = default;
void PadControl::trackLoaded(TrackPointer track) {
    m_active = -1; m_window = {}; m_beats = track ? track->getBeats() : nullptr;
    m_enable->set(0); m_loop->set(0); m_releaseSamples = 0; m_releaseTotal = 0;
    m_fadeSamples = 0;
    if (m_scratching) { m_scratch2Enable->set(0); m_scratching = false; }
    m_releaseActive->forceSet(0);
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
    Q_UNUSED(bufferSize);
    const qint64 remaining = m_releaseSamples.load();
    const int type = m_activeType.load();
    if (remaining > 0 && type != kEcho) {
        if (!m_scratching) {
            m_startRate = rate != 0 ? rate : 1;
            m_scratching = true;
            m_scratch2Enable->set(1);
        }
        const double left = static_cast<double>(remaining) / std::max<qint64>(1, m_releaseTotal.load());
        // BREAK: speed falls linearly to a stop. SPIN: a fast backspin that
        // slows down quadratically.
        m_scratch2->set(type == kBreak ? m_startRate * left : -3.0 * left * left);
    }
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
    // Release the beat effects playing on this deck, as rekordbox does: the
    // units routed here are switched off, the routing itself is kept.
    for (int i = 1; i <= 4; ++i) {
        const QString unit = QString("[EffectRack1_EffectUnit%1]").arg(i);
        const auto route = ControlObject::getControl(ConfigKey(unit, QString("group_%1_enable").arg(getGroup())),
                ControlFlag::NoWarnIfMissing);
        if (!route || !route->toBool()) continue;
        for (const auto& key : {ConfigKey(unit, "enabled"),
                     ConfigKey(QString("[EffectRack1_EffectUnit%1_Effect1]").arg(i), "enabled")}) {
            if (auto* control = ControlObject::getControl(key, ControlFlag::NoWarnIfMissing)) control->set(0);
        }
    }
    const double rate = std::max(8000.0, m_sampleRate->get());
    const double beats = std::clamp(m_releaseBeats->get(), 1.0, 16.0);
    m_activeType = std::clamp(static_cast<int>(m_releaseType->get()), 0, 2);
    const qint64 total = std::llround(rate * 60 * beats / m_bpm->get()) * 2;
    m_releaseTotal = total;
    m_releaseSamples = total;
}
void PadControl::finishRelease(CSAMPLE* samples, int from, int count) {
    if (m_scratching) { m_scratch2Enable->set(0); m_scratching = false; }
    m_releaseActive->forceSet(0);
    if (m_stop->toBool()) {
        m_play->set(0);
        std::fill(samples + from, samples + count, 0);
        return;
    }
    // Bring the deck back in over half a beat instead of a hard cut.
    const double rate = std::max(8000.0, m_sampleRate->get());
    m_fadeTotal = std::max<qint64>(2, std::llround(rate * 30 / std::max(30.0, m_bpm->get())) * 2);
    m_fadeSamples = m_fadeTotal;
}
void PadControl::processAudio(CSAMPLE* samples, int count) {
    if (m_delay.empty()) return;
    const double rate = std::max(8000.0, m_sampleRate->get());
    const size_t beatSamples = std::min(m_delay.size() - 2,
            std::max(size_t(2), static_cast<size_t>(rate * 60 / std::max(30.0, m_bpm->get())) * 2));
    qint64 remaining = m_releaseSamples.load();
    const qint64 total = std::max<qint64>(1, m_releaseTotal.load());
    const bool echo = m_activeType.load() == kEcho;
    if (remaining > 0) m_releaseActive->forceSet(1);
    for (int i = 0; i < count; ++i) {
        if (remaining > 0 && echo) {
            // ECHO: the dry signal is cut, the last beat repeats and decays.
            const float wet = m_delay[(m_delayWrite + m_delay.size() - beatSamples) % m_delay.size()];
            m_delay[m_delayWrite] = wet * 0.6f;
            samples[i] = wet * static_cast<float>(remaining) / total;
        } else {
            // BREAK and SPIN change the playback speed in process(); the audio
            // passes through. Also feeds the echo buffer.
            if (m_fadeSamples > 0) {
                samples[i] *= 1.0f - static_cast<float>(m_fadeSamples) / m_fadeTotal;
                --m_fadeSamples;
            }
            m_delay[m_delayWrite] = samples[i];
        }
        m_delayWrite = (m_delayWrite + 1) % m_delay.size();
        if (remaining > 0 && --remaining == 0) {
            m_releaseSamples = 0;
            finishRelease(samples, i + 1, count);
            if (m_stop->toBool()) return;
        }
    }
    if (remaining > 0) m_releaseSamples = remaining;
}
