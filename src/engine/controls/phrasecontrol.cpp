#include "engine/controls/phrasecontrol.h"
#include <atomic>
#include <cmath>
#include "analyzer/phraseanalysisstore.h"
#include "control/controlobject.h"
#include "control/controlpushbutton.h"
#include "track/track.h"
#include "engine/enginebuffer.h"
#include "moc_phrasecontrol.cpp"

PhraseControl::PhraseControl(const QString& group, UserSettingsPointer config)
        : EngineControl(group, config) {
    m_type = std::make_unique<ControlObject>(ConfigKey(group, "phrase_type"));
    m_index = std::make_unique<ControlObject>(ConfigKey(group, "phrase_index"));
    m_bars = std::make_unique<ControlObject>(ConfigKey(group, "phrase_bars_left"));
    m_valid = std::make_unique<ControlObject>(ConfigKey(group, "phrase_valid"));
    for (auto* control : {m_type.get(), m_index.get(), m_bars.get(), m_valid.get()}) control->setReadOnly();
    m_next = std::make_unique<ControlPushButton>(ConfigKey(group, "phrase_jump_next"));
    m_previous = std::make_unique<ControlPushButton>(ConfigKey(group, "phrase_jump_prev"));
    connect(m_next.get(), &ControlObject::valueChanged, this, [this](double value) { if (value > 0) jump(true); }, Qt::DirectConnection);
    connect(m_previous.get(), &ControlObject::valueChanged, this, [this](double value) { if (value > 0) jump(false); }, Qt::DirectConnection);
}
PhraseControl::~PhraseControl() { QObject::disconnect(m_connection); }
void PhraseControl::trackLoaded(TrackPointer track) {
    QObject::disconnect(m_connection);
    std::shared_ptr<const State> state;
    if (track) state = std::make_shared<const State>(State{PhraseAnalysisStore::trackKey(track), track->getBeats(), PhraseAnalysisStore::instance().get(PhraseAnalysisStore::trackKey(track))});
    std::atomic_store(&m_state, state);
    if (track) {
        m_connection = connect(&PhraseAnalysisStore::instance(), &PhraseAnalysisStore::updated, this,
                [this, weak = TrackWeakPointer(track)](const QString& location) {
                    auto old = std::atomic_load(&m_state);
                    if (!old || old->location != location) return;
                    auto track = weak.lock();
                    if (!track) return;
                    std::shared_ptr<const State> updated = std::make_shared<const State>(State{
                            location, track->getBeats(), PhraseAnalysisStore::instance().get(location)});
                    std::atomic_compare_exchange_strong(&m_state, &old, updated);
                }, Qt::DirectConnection);
    }
    m_type->forceSet(0); m_index->forceSet(-1); m_bars->forceSet(0); m_valid->forceSet(0);
}
std::shared_ptr<const vilelabox::PhraseAnalysis> PhraseControl::analysis() const {
    const auto state = std::atomic_load(&m_state);
    return state ? state->analysis : nullptr;
}
void PhraseControl::process(double rate, mixxx::audio::FramePos position, int bufferSize) {
    Q_UNUSED(rate); Q_UNUSED(bufferSize);
    const auto state = std::atomic_load(&m_state);
    const auto result = state ? state->analysis : nullptr;
    const auto info = frameInfo();
    int index = result && info.sampleRate.isValid() ? result->indexAt(position.value() / info.sampleRate) : -1;
    m_valid->forceSet(result && !result->phrases.isEmpty());
    m_index->forceSet(index);
    if (index < 0) { m_type->forceSet(0); m_bars->forceSet(0); return; }
    const auto& phrase = result->phrases[index];
    m_type->forceSet(static_cast<int>(phrase.type));
    double left = 0;
    if (state->beats) {
        auto current = state->beats->iteratorFrom(position);
        if (*current > position) --current;
        const auto end = state->beats->iteratorFrom(mixxx::audio::FramePos(phrase.end * info.sampleRate));
        left = std::ceil((end - current) / 4.0 - 1e-6);
    }
    m_bars->forceSet(std::max(0.0, left));
}
void PhraseControl::jump(bool next) {
    const auto result = analysis();
    const auto info = frameInfo();
    if (!result || result->phrases.isEmpty() || !info.sampleRate.isValid()) return;
    const double seconds = info.currentPosition.value() / info.sampleRate;
    int index = result->indexAt(seconds);
    if (index < 0) index = next ? 0 : result->phrases.size() - 1;
    else if (next) ++index;
    if (index >= result->phrases.size()) return;
    // Preserve the position within the bar, just like bar_jump.
    const auto track = getEngineBuffer()->getLoadedTrack();
    const auto beats = track ? track->getBeats() : nullptr;
    auto target = mixxx::audio::FramePos(result->phrases[index].start * info.sampleRate);
    if (beats) {
        const auto phase = beats->barPhase(info.currentPosition);
        if (phase) {
            auto it = beats->iteratorFrom(target);
            target = *(it + static_cast<int>(*phase)) + (*phase - std::floor(*phase)) * it.beatLengthFrames();
        }
    }
    seekExact(target);
}
