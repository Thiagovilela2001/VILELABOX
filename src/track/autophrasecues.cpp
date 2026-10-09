#include "track/autophrasecues.h"
#include <array>
#include "track/track.h"

void AutoPhraseCues::setTrack(TrackPointer track) {
    QMutexLocker lock(&m_mutex);
    m_track = track;
    m_created.clear();
}
void AutoPhraseCues::apply(const vilelabox::PhraseAnalysis& analysis, TrackPointer expectedTrack) {
    QMutexLocker lock(&m_mutex);
    auto track = m_track.lock();
    if (!track || (expectedTrack && expectedTrack != track) || track->isBpmLocked() || analysis.lowConfidence || analysis.phrases.isEmpty() ||
            !track->getSampleRate().isValid()) return;
    std::array<bool, 8> occupied{};
    const auto cues = track->getCuePoints();
    for (const auto& cue : cues) if (cue->getHotCue() >= 0 && cue->getHotCue() < 8) occupied[cue->getHotCue()] = true;
    const double rate = track->getSampleRate();
    for (const auto& phrase : analysis.phrases) {
        if (phrase.confidence < 0.5) continue;
        auto type = mixxx::CueType::Invalid;
        int slot = Cue::kNoHotCue;
        QString label;
        mixxx::RgbColor color(0xffd447);
        if (phrase.type == vilelabox::PhraseType::Intro) { type = mixxx::CueType::Intro; label = "Intro"; }
        else if (phrase.type == vilelabox::PhraseType::Outro) { type = mixxx::CueType::Outro; label = "Outro"; }
        else if (phrase.type == vilelabox::PhraseType::Chorus || phrase.type == vilelabox::PhraseType::Down || phrase.type == vilelabox::PhraseType::Up) {
            type = mixxx::CueType::HotCue;
            for (int i = 0; i < 8; ++i) if (!occupied[i]) { slot = i; occupied[i] = true; break; }
            if (slot < 0) continue;
            if (phrase.type == vilelabox::PhraseType::Chorus) { label = "Chorus"; color = mixxx::RgbColor(0xff4545); }
            else if (phrase.type == vilelabox::PhraseType::Down) { label = "Down"; color = mixxx::RgbColor(0x498dff); }
            else label = "Up";
            // Reanalysis does not duplicate an existing cue at this boundary.
            bool exists = false;
            for (const auto& cue : track->getCuePoints()) if (cue->getType() == type &&
                    cue->getPosition() == mixxx::audio::FramePos(phrase.start * rate)) { exists = true; break; }
            if (exists) { occupied[slot] = false; continue; }
        }
        if (type == mixxx::CueType::Invalid || (type != mixxx::CueType::HotCue && track->findCueByType(type))) continue;
        const auto start = mixxx::audio::FramePos(phrase.start * rate);
        const auto end = type == mixxx::CueType::HotCue ? mixxx::audio::kInvalidFramePos :
                mixxx::audio::FramePos((type == mixxx::CueType::Outro ? analysis.duration : phrase.end) * rate);
        auto cue = track->createAndAddCue(type, slot, start, end, color);
        cue->setLabel(label);
        m_created.push_back({cue, cue->getStartAndEndPosition(), label, color});
    }
}
void AutoPhraseCues::undo() {
    QMutexLocker lock(&m_mutex);
    auto track = m_track.lock();
    if (!track || track->isBpmLocked()) return;
    const auto current = track->getCuePoints();
    for (const auto& created : m_created) {
        const auto& cue = created.cue;
        if (current.contains(cue) && cue->getPosition() == created.positions.startPosition &&
                cue->getEndPosition() == created.positions.endPosition && cue->getLabel() == created.label &&
                cue->getColor() == created.color) track->removeCue(cue);
    }
    m_created.clear();
}
