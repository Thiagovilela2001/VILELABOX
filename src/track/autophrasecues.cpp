#include "track/autophrasecues.h"
#include <algorithm>
#include <array>
#include <cmath>
#include "analyzer/vocalanalysisstore.h"
#include "track/mixpoints.h"
#include "track/track.h"

void AutoPhraseCues::setTrack(TrackPointer track) {
    QMutexLocker lock(&m_mutex);
    m_track = track;
    m_created.clear();
}
bool AutoPhraseCues::apply(const vilelabox::PhraseAnalysis& analysis, TrackPointer expectedTrack) {
    QMutexLocker lock(&m_mutex);
    auto track = m_track.lock();
    if (!track || (expectedTrack && expectedTrack != track) || track->isBpmLocked() || analysis.lowConfidence || analysis.phrases.isEmpty() ||
            !track->getSampleRate().isValid()) return false;
    const double rate = track->getSampleRate();
    const auto beats = track->getBeats();
    if (!beats) return false;

    // The same points Mix Point Link uses, plus the drops and the break.
    vilelabox::MixTrack mix;
    mix.phrases = &analysis;
    mix.beats = beats;
    mix.duration = analysis.duration > 0 ? analysis.duration : track->getDuration();
    const auto vocals = VocalAnalysisStore::instance().get(track->getLocation());
    if (vocals && vocals->status == VocalAnalysis::Status::Ready) mix.vocals = vocals->segments;
    const auto seconds = [rate](const CuePointer& cue) {
        return cue && cue->getPosition().isValid() ? std::optional<double>(cue->getPosition().value() / rate) : std::nullopt;
    };
    mix.introCue = seconds(track->findCueByType(mixxx::CueType::Intro));
    mix.outroCue = seconds(track->findCueByType(mixxx::CueType::Outro));
    if (track->getMainCuePosition().isValid()) mix.mainCue = track->getMainCuePosition().value() / rate;

    const auto confident = [](const vilelabox::Phrase& p) { return p.confidence >= 0.5; };
    std::optional<double> drop, breakdown, secondDrop;
    for (const auto& phrase : analysis.phrases) {
        if (!confident(phrase)) continue;
        if (phrase.type == vilelabox::PhraseType::Chorus) {
            if (!drop) drop = phrase.start;
            else if (breakdown && !secondDrop && phrase.start > *breakdown) secondDrop = phrase.start;
        } else if (phrase.type == vilelabox::PhraseType::Down && drop && !breakdown) {
            breakdown = phrase.start;
        }
    }
    const auto mixIn = vilelabox::mixInPoint(mix);
    const auto mixOut = vilelabox::mixOutRange(mix);

    struct Point { std::optional<double> at; const char* label; quint32 color; };
    const Point points[]{
            {mixIn, "MIX IN", 0x2dd728},
            {drop, "DROP", 0xff4545},
            {breakdown, "BREAK", 0x498dff},
            {secondDrop, "DROP 2", 0xff4545},
            {mixOut ? std::optional<double>(mixOut->start) : std::nullopt, "MIX OUT", 0xf8a030},
    };
    std::array<bool, 8> occupied{};
    for (const auto& cue : track->getCuePoints()) {
        if (cue->getHotCue() >= 0 && cue->getHotCue() < 8) occupied[cue->getHotCue()] = true;
    }
    // One beat of tolerance: grid edits move points slightly, and two points
    // on the same beat would be the same hot cue.
    const auto isNear = [&beats](mixxx::audio::FramePos a, mixxx::audio::FramePos b) {
        return std::abs(a - b) <= beats->iteratorFrom(a).beatLengthFrames();
    };
    QList<mixxx::audio::FramePos> hotCuePositions;
    for (const auto& cue : track->getCuePoints()) {
        if (cue->getType() == mixxx::CueType::HotCue) hotCuePositions.push_back(cue->getPosition());
    }
    for (const auto& point : points) {
        if (!point.at || *point.at < 0) continue;
        const auto position = mixxx::audio::FramePos(*point.at * rate);
        if (std::any_of(hotCuePositions.cbegin(), hotCuePositions.cend(),
                    [&](auto other) { return isNear(position, other); })) continue;
        int slot = -1;
        for (int i = 0; i < 8; ++i) if (!occupied[i]) { slot = i; occupied[i] = true; break; }
        if (slot < 0) break;
        const mixxx::RgbColor color(point.color);
        auto cue = track->createAndAddCue(mixxx::CueType::HotCue, slot, position, mixxx::audio::kInvalidFramePos, color);
        cue->setLabel(QString::fromLatin1(point.label));
        m_created.push_back({cue, cue->getStartAndEndPosition(), cue->getLabel(), color});
        hotCuePositions.push_back(position);
    }
    // Range cues for the intro (mix in -> drop) and outro (mix out -> end).
    const auto addRange = [&](mixxx::CueType type, double from, double to, const char* label) {
        if (track->findCueByType(type) || to <= from) return;
        const mixxx::RgbColor color(0xffd447);
        auto cue = track->createAndAddCue(type, Cue::kNoHotCue,
                mixxx::audio::FramePos(from * rate), mixxx::audio::FramePos(to * rate), color);
        cue->setLabel(QString::fromLatin1(label));
        m_created.push_back({cue, cue->getStartAndEndPosition(), cue->getLabel(), color});
    };
    if (mixIn && drop) addRange(mixxx::CueType::Intro, *mixIn, *drop, "Intro");
    if (mixOut) addRange(mixxx::CueType::Outro, mixOut->start, mixOut->end, "Outro");
    return true;
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
