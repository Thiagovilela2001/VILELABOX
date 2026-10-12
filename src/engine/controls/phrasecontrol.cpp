#include "engine/controls/phrasecontrol.h"
#include <QThread>
#include <algorithm>
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
    m_autoCues = std::make_unique<ControlPushButton>(ConfigKey(group, "auto_phrase_cues"), true,
            config->getValue(ConfigKey("[Analyzer]", "AutoPhraseCues"), false) ? 1.0 : 0.0);
    m_autoCues->setButtonMode(ControlPushButton::TOGGLE);
    m_undoCues = std::make_unique<ControlPushButton>(ConfigKey(group, "auto_phrase_cues_undo"));
    connect(m_undoCues.get(), &ControlObject::valueChanged, this, [this](double value) { if (value > 0) m_cueGenerator.undo(); }, Qt::DirectConnection);
    m_next = std::make_unique<ControlPushButton>(ConfigKey(group, "phrase_jump_next"));
    m_previous = std::make_unique<ControlPushButton>(ConfigKey(group, "phrase_jump_prev"));
    connect(m_next.get(), &ControlObject::valueChanged, this, [this](double value) { if (value > 0) jump(true); }, Qt::DirectConnection);
    connect(m_previous.get(), &ControlObject::valueChanged, this, [this](double value) { if (value > 0) jump(false); }, Qt::DirectConnection);
}
PhraseControl::~PhraseControl() { QObject::disconnect(m_connection); }
void PhraseControl::trackLoaded(TrackPointer track) {
    // Audio callback: no allocation, no locks, no queued calls. The GUI thread
    // prepares the phrase state (trackLoadedOnGuiThread / store updates).
    m_audioTrack.store(track.get());
    m_type->forceSet(0); m_index->forceSet(-1); m_bars->forceSet(0); m_valid->forceSet(0);
}
void PhraseControl::trackLoadedOnGuiThread() {
    if (!m_connection) {
        // Late results of a previous track are filtered in rebuild() by
        // comparing locations with the track that is loaded now.
        m_connection = connect(&PhraseAnalysisStore::instance(), &PhraseAnalysisStore::updated,
                this, [this](const QString&) { rebuild(); });
    }
    rebuild();
}
void PhraseControl::rebuild() {
    const auto track = getEngineBuffer()->getLoadedTrack();
    if (track.get() != m_cueTrack) {
        // A different track: new cue history, cues not generated yet.
        m_cueTrack = track.get();
        m_cuesApplied = false;
        m_cueGenerator.setTrack(track);
    }
    if (!track) {
        publish(nullptr);
        return;
    }
    auto state = std::make_unique<State>();
    state->track = track.get();
    state->location = PhraseAnalysisStore::trackKey(track);
    state->beats = track->getBeats();
    state->analysis = PhraseAnalysisStore::instance().get(state->location);
    const bool hasAnalysis = state->analysis != nullptr;
    publish(std::move(state));
    if (hasAnalysis) applyAutoCues(track);
}
void PhraseControl::publish(std::unique_ptr<const State> state) {
    m_current.store(state.get());
    if (m_owned) m_retired.push_back(std::move(m_owned));
    m_owned = std::move(state);
    // Free old states the audio thread is not reading. A state stays retired
    // while it is the audio thread's hazard pointer.
    const State* inUse = m_hazard.load();
    m_retired.erase(std::remove_if(m_retired.begin(), m_retired.end(),
                            [inUse](const auto& old) { return old.get() != inUse; }),
            m_retired.end());
}
const PhraseControl::State* PhraseControl::acquireForAudio() const {
    const State* state;
    do {
        state = m_current.load();
        m_hazard.store(state);
    } while (state != m_current.load());
    return state && state->track == m_audioTrack.load() ? state : nullptr;
}
void PhraseControl::applyAutoCues(const TrackPointer& track) {
    if (m_cuesApplied || !m_autoCues->toBool() || !m_owned || !m_owned->analysis) return;
    if (m_cueGenerator.apply(*m_owned->analysis, track)) m_cuesApplied = true;
}
std::shared_ptr<const vilelabox::PhraseAnalysis> PhraseControl::analysis() const {
    return m_owned ? m_owned->analysis : nullptr;
}
const vilelabox::PhraseAnalysis* PhraseControl::analysisForAudio() const {
    const State* state = acquireForAudio();
    return state ? state->analysis.get() : nullptr;
}
void PhraseControl::process(double rate, mixxx::audio::FramePos position, int bufferSize) {
    Q_UNUSED(rate); Q_UNUSED(bufferSize);
    const State* state = acquireForAudio();
    const vilelabox::PhraseAnalysis* result = state ? state->analysis.get() : nullptr;
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
    // Controllers set the control from their own thread; the state is GUI-owned.
    if (QThread::currentThread() != thread()) {
        QMetaObject::invokeMethod(this, [this, next] { jump(next); }, Qt::QueuedConnection);
        return;
    }
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
