#include "engine/controls/mixpointcontrol.h"
#include <cmath>
#include "control/controlobject.h"
#include "control/controlpushbutton.h"
#include "control/controlproxy.h"
#include "engine/enginebuffer.h"
#include "engine/controls/phrasecontrol.h"
#include "track/track.h"
#include "track/cue.h"
#include "moc_mixpointcontrol.cpp"

MixPointControl::MixPointControl(const QString& group, UserSettingsPointer config) : EngineControl(group, config) {
    m_play = std::make_unique<ControlProxy>(group, "play", this);
    m_link = std::make_unique<ControlPushButton>(ConfigKey(group, "mix_point_link"));
    m_silent = std::make_unique<ControlPushButton>(ConfigKey(group, "mix_point_link_silent"));
    m_exit = std::make_unique<ControlPushButton>(ConfigKey(group, "mix_point_link_exit"));
    m_next = std::make_unique<ControlPushButton>(ConfigKey(group, "mix_point_select_next"));
    m_previous = std::make_unique<ControlPushButton>(ConfigKey(group, "mix_point_select_prev"));
    m_armed = std::make_unique<ControlObject>(ConfigKey(group, "mix_point_link_armed"));
    m_left = std::make_unique<ControlObject>(ConfigKey(group, "mix_point_bars_to_go"));
    m_error = std::make_unique<ControlObject>(ConfigKey(group, "mix_point_error"));
    m_in = std::make_unique<ControlObject>(ConfigKey(group, "mix_point_in_position"));
    m_out = std::make_unique<ControlObject>(ConfigKey(group, "mix_point_out_position"));
    for (auto* control : {m_armed.get(), m_left.get(), m_error.get(), m_in.get(), m_out.get()}) control->setReadOnly();
    connect(m_link.get(), &ControlObject::valueChanged, this, [this](double v) { if (v > 0) arm(false); }, Qt::DirectConnection);
    connect(m_silent.get(), &ControlObject::valueChanged, this, [this](double v) { if (v > 0) arm(true); }, Qt::DirectConnection);
    connect(m_exit.get(), &ControlObject::valueChanged, this, [this](double v) { if (v > 0) cancel(); }, Qt::DirectConnection);
    connect(m_next.get(), &ControlObject::valueChanged, this, [this](double v) { if (v > 0) select(1); }, Qt::DirectConnection);
    connect(m_previous.get(), &ControlObject::valueChanged, this, [this](double v) { if (v > 0) select(-1); }, Qt::DirectConnection);
}
MixPointControl::~MixPointControl() = default;
void MixPointControl::cancel(int error) {
    m_source = nullptr; m_sourceTrack.reset(); m_armed->forceSet(0); m_left->forceSet(0); m_error->forceSet(error);
}
void MixPointControl::trackLoaded(TrackPointer track) {
    Q_UNUSED(track);
    cancel(); m_in->forceSet(-1); m_out->forceSet(-1);
}
void MixPointControl::refreshPoints() {
    if (m_in->get() >= 0 && m_out->get() >= 0) return;
    const auto track = getEngineBuffer()->getLoadedTrack();
    if (!track) return;
    const auto phrases = getEngineBuffer()->phraseControl()->analysis();
    if (m_in->get() < 0) {
        auto cue = track->findCueByType(mixxx::CueType::Intro);
        auto position = cue ? cue->getPosition() : track->getMainCuePosition();
        if (position.isValid()) m_in->forceSet(position.toEngineSamplePos());
    }
    if (m_out->get() < 0) {
        auto cue = track->findCueByType(mixxx::CueType::Outro);
        auto position = cue ? cue->getPosition() : mixxx::audio::kInvalidFramePos;
        if (!position.isValid() && phrases && !phrases->phrases.isEmpty()) position = mixxx::audio::FramePos(phrases->phrases.last().start * track->getSampleRate());
        if (position.isValid()) m_out->forceSet(position.toEngineSamplePos());
    }
}
void MixPointControl::select(int direction) {
    cancel(); refreshPoints();
    const auto track = getEngineBuffer()->getLoadedTrack();
    const auto phrases = getEngineBuffer()->phraseControl()->analysis();
    if (!track || !phrases || phrases->phrases.isEmpty()) return;
    auto* point = m_play->toBool() ? m_out.get() : m_in.get();
    const double seconds = point->get() / (2 * track->getSampleRate());
    int index = phrases->indexAt(seconds);
    index = qBound(0, index + direction, static_cast<int>(phrases->phrases.size()) - 1);
    point->forceSet(phrases->phrases[index].start * track->getSampleRate() * 2);
}
void MixPointControl::arm(bool silent) {
    if (m_armed->toBool()) { cancel(); return; }
    refreshPoints();
    auto* source = pickSyncTarget();
    if (!source || source == getEngineBuffer() || m_play->toBool()) { cancel(1); return; }
    auto sourceTrack = source->getLoadedTrack();
    auto own = getEngineBuffer()->getLoadedTrack();
    if (!sourceTrack || !own || !own->getBeats() || !sourceTrack->getBeats() || m_in->get() < 0) { cancel(2); return; }
    const auto ownBpm = own->getBpm() * getEngineBuffer()->getRateRatio();
    const auto sourceBpm = sourceTrack->getBpm() * source->getRateRatio();
    if (ownBpm <= 0 || std::abs(ownBpm - sourceBpm) > 0.05) { cancel(3); return; }
    auto trigger = mixxx::audio::FramePos::fromEngineSamplePosMaybeInvalid(ControlObject::get(ConfigKey(source->getGroup(), "mix_point_out_position")));
    if (!trigger.isValid()) {
        auto cue = sourceTrack->findCueByType(mixxx::CueType::Outro);
        if (cue) trigger = cue->getPosition();
        else {
            auto phrases = source->phraseControl()->analysis();
            if (phrases && !phrases->phrases.isEmpty()) trigger = mixxx::audio::FramePos(phrases->phrases.last().start * sourceTrack->getSampleRate());
        }
    }
    if (!trigger.isValid() || trigger <= source->getExactPlayPos()) { cancel(2); return; }
    m_sourceSeekGeneration = source->seekGeneration();
    m_source = source; m_sourceTrack = sourceTrack; m_trigger = trigger;
    m_previousPosition = source->getExactPlayPos();
    m_entry = mixxx::audio::FramePos::fromEngineSamplePos(m_in->get());
    m_silentLaunch = silent; m_error->forceSet(0); m_armed->forceSet(1);
}
void MixPointControl::notifySeek(mixxx::audio::FramePos position) { Q_UNUSED(position); if (m_source) cancel(); }
void MixPointControl::process(double rate, mixxx::audio::FramePos position, int bufferSize) {
    Q_UNUSED(rate); Q_UNUSED(position);
    if (!m_source) { refreshPoints(); return; }
    auto track = m_sourceTrack.lock();
    if (!track || m_sourceSeekGeneration != m_source->seekGeneration() ||
            track != m_source->getLoadedTrack() || m_play->toBool() ||
            ControlObject::get(ConfigKey(m_source->getGroup(), "play")) == 0) { cancel(4); return; }
    auto current = m_source->getExactPlayPos();
    const double delta = current - m_previousPosition;
    // Forward discontinuities and reverse seeks invalidate the armed link.
    const double maximum = bufferSize * 4 * std::max(1.0, std::abs(m_source->getRateRatio()));
    if (delta < -1 || delta > maximum) { cancel(4); return; }
    const auto beats = track->getBeats();
    if (!beats) { cancel(2); return; }
    const double ownBpm = getEngineBuffer()->getLoadedTrack()->getBpm() * getEngineBuffer()->getRateRatio();
    if (std::abs(ownBpm - track->getBpm() * m_source->getRateRatio()) > 0.05) { cancel(3); return; }
    const auto here = beats->iteratorFrom(current), end = beats->iteratorFrom(m_trigger);
    m_left->forceSet(std::max(0.0, std::ceil((end - here) / 4.0)));
    if (current >= m_trigger) {
        const auto entry = m_entry;
        const bool silent = m_silentLaunch;
        cancel();
        if (silent) ControlObject::set(ConfigKey(getGroup(), "volume"), 0);
        seekExact(entry);
        m_play->set(1);
    } else m_previousPosition = current;
}
