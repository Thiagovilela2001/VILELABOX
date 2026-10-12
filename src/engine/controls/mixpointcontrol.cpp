#include "engine/controls/mixpointcontrol.h"
#include <cmath>
#include "analyzer/phraseanalysisstore.h"
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
    m_bpm = std::make_unique<ControlProxy>(group, "bpm", this);
    m_volume = std::make_unique<ControlProxy>(group, "volume", this);
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
    m_in->forceSet(-1); m_out->forceSet(-1);
    connect(m_link.get(), &ControlObject::valueChanged, this, [this](double v) { if (v > 0) arm(false); }, Qt::DirectConnection);
    connect(m_silent.get(), &ControlObject::valueChanged, this, [this](double v) { if (v > 0) arm(true); }, Qt::DirectConnection);
    connect(m_exit.get(), &ControlObject::valueChanged, this, [this](double v) { if (v > 0) cancel(); }, Qt::DirectConnection);
    connect(m_next.get(), &ControlObject::valueChanged, this, [this](double v) { if (v > 0) select(1); }, Qt::DirectConnection);
    connect(m_previous.get(), &ControlObject::valueChanged, this, [this](double v) { if (v > 0) select(-1); }, Qt::DirectConnection);
}
MixPointControl::~MixPointControl() {
    QObject::disconnect(m_cuesConnection);
    QObject::disconnect(m_phrasesConnection);
}
void MixPointControl::cancel(int error) {
    m_source.store(nullptr, std::memory_order_release);
    m_armed->forceSet(0); m_left->forceSet(0); m_error->forceSet(error);
}
void MixPointControl::trackLoaded(TrackPointer track) {
    // Audio callback: no allocation, locks or queued calls here.
    Q_UNUSED(track);
    cancel(); m_in->forceSet(-1); m_out->forceSet(-1);
    m_inSelected = false; m_outSelected = false;
}
void MixPointControl::trackLoadedOnGuiThread() {
    QObject::disconnect(m_cuesConnection);
    const auto track = getEngineBuffer()->getLoadedTrack();
    if (track) {
        m_cuesConnection = connect(track.get(), &Track::cuesUpdated, this, [this] { refreshPoints(); }, Qt::QueuedConnection);
    }
    if (!m_phrasesConnection) {
        m_phrasesConnection = connect(&PhraseAnalysisStore::instance(), &PhraseAnalysisStore::updated,
                this, [this] { refreshPoints(); }, Qt::QueuedConnection);
    }
    refreshPoints();
}
void MixPointControl::refreshPoints() {
    const auto track = getEngineBuffer()->getLoadedTrack();
    if (!track) return;
    const auto phrases = getEngineBuffer()->phraseControl()->analysis();
    if (!m_inSelected) {
        auto cue = track->findCueByType(mixxx::CueType::Intro);
        auto position = cue ? cue->getPosition() : track->getMainCuePosition();
        m_in->forceSet(position.isValid() ? position.toEngineSamplePos() : -1);
    }
    if (!m_outSelected) {
        auto cue = track->findCueByType(mixxx::CueType::Outro);
        auto position = cue ? cue->getPosition() : mixxx::audio::kInvalidFramePos;
        if (!position.isValid() && phrases && !phrases->phrases.isEmpty()) position = mixxx::audio::FramePos(phrases->phrases.last().start * track->getSampleRate());
        m_out->forceSet(position.isValid() ? position.toEngineSamplePos() : -1);
    }
}
void MixPointControl::select(int direction) {
    cancel(); refreshPoints();
    const auto track = getEngineBuffer()->getLoadedTrack();
    const auto phrases = getEngineBuffer()->phraseControl()->analysis();
    if (!track || !phrases || phrases->phrases.isEmpty()) return;
    const bool exit = m_play->toBool();
    auto* point = exit ? m_out.get() : m_in.get();
    const double seconds = point->get() / (2 * track->getSampleRate());
    int index = phrases->indexAt(seconds);
    index = qBound(0, index + direction, static_cast<int>(phrases->phrases.size()) - 1);
    point->forceSet(phrases->phrases[index].start * track->getSampleRate() * 2);
    (exit ? m_outSelected : m_inSelected) = true;
}
void MixPointControl::arm(bool silent) {
    if (m_armed->toBool()) { cancel(); return; }
    refreshPoints();
    auto* source = pickSyncTarget();
    if (!source || source == getEngineBuffer() || m_play->toBool()) { cancel(1); return; }
    auto sourceTrack = source->getLoadedTrack();
    auto own = getEngineBuffer()->getLoadedTrack();
    const auto sourceBeats = sourceTrack ? sourceTrack->getBeats() : nullptr;
    if (!sourceTrack || !own || !own->getBeats() || !sourceBeats || m_in->get() < 0) { cancel(2); return; }
    auto& proxies = m_sourceProxies[source->getGroup()];
    if (!proxies.play) {
        proxies.play = std::make_unique<ControlProxy>(source->getGroup(), "play", this);
        proxies.bpm = std::make_unique<ControlProxy>(source->getGroup(), "bpm", this);
    }
    m_sourcePlay = proxies.play.get(); m_sourceBpm = proxies.bpm.get();
    const double ownBpm = m_bpm->get();
    if (ownBpm <= 0 || std::abs(ownBpm - m_sourceBpm->get()) > 0.05) { cancel(3); return; }
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
    m_sourceTrack = sourceTrack; m_sourceBeats = sourceBeats; m_trigger = trigger;
    m_ownBeats = own->getBeats();
    m_previousPosition = source->getExactPlayPos();
    m_entry = mixxx::audio::FramePos::fromEngineSamplePos(m_in->get());
    m_silentLaunch = silent; m_error->forceSet(0); m_armed->forceSet(1);
    m_source.store(source, std::memory_order_release);
}
void MixPointControl::notifySeek(mixxx::audio::FramePos position) {
    Q_UNUSED(position);
    if (m_source.load(std::memory_order_acquire)) cancel();
}
void MixPointControl::process(double rate, mixxx::audio::FramePos position, int bufferSize) {
    Q_UNUSED(rate); Q_UNUSED(position);
    // Audio callback: only proxies, atomics and immutable beats; no track locks.
    auto* source = m_source.load(std::memory_order_acquire);
    if (!source) return;
    auto track = m_sourceTrack.lock();
    if (!track || m_sourceSeekGeneration != source->seekGeneration() ||
            track != source->getLoadedTrack() || m_play->toBool() || !m_sourcePlay->toBool()) { cancel(4); return; }
    auto current = source->getExactPlayPos();
    const double delta = current - m_previousPosition;
    // Forward discontinuities and reverse seeks invalidate the armed link.
    const double maximum = bufferSize * 4 * std::max(1.0, std::abs(source->getRateRatio()));
    if (delta < -1 || delta > maximum) { cancel(4); return; }
    if (std::abs(m_bpm->get() - m_sourceBpm->get()) > 0.05) { cancel(3); return; }
    const auto here = m_sourceBeats->iteratorFrom(current), end = m_sourceBeats->iteratorFrom(m_trigger);
    m_left->forceSet(std::max(0.0, std::ceil((end - here) / 4.0)));
    // During the callback the source reports its position at the start of
    // this buffer. Our seek and play take effect at the start of the next
    // one, when the source will be one buffer further (measured advance).
    const double advance = delta > 0 ? delta : bufferSize / 2.0 * std::abs(source->getRateRatio());
    const auto next = current + advance;
    if (next >= m_trigger) {
        // Enter as many beats past the entry as the source is past the exit,
        // so both decks stay in phase.
        auto entry = m_entry;
        const double sourceBeat = m_sourceBeats->iteratorFrom(m_trigger).beatLengthFrames();
        if (m_ownBeats && sourceBeat > 0) {
            const double overshootBeats = (next - m_trigger) / sourceBeat;
            entry += overshootBeats * m_ownBeats->iteratorFrom(entry).beatLengthFrames();
        }
        const bool silent = m_silentLaunch;
        cancel();
        if (silent) m_volume->set(0);
        seekExact(entry);
        m_play->set(1);
    } else m_previousPosition = current;
}
