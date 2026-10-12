#pragma once
#include <QPair>
#include <QVector>
#include <optional>
#include "track/beats.h"
#include "track/phrases.h"

// Automatic mix points for Mix Point Link (MIXPOINT.md). Pure functions over a
// track's grid, phrases, vocals and cues; all positions are in seconds.
namespace vilelabox {
struct MixTrack {
    const PhraseAnalysis* phrases = nullptr;
    QVector<QPair<double, double>> vocals;
    mixxx::BeatsPointer beats;
    double duration = 0;
    std::optional<double> introCue, outroCue, mainCue;
};
struct MixOutRange {
    double start = 0; // the track starts to wind down
    double end = 0;   // the music is over (end of the outro)
};
/// Where the incoming track starts: Intro cue, else bar 1, else main cue.
std::optional<double> mixInPoint(const MixTrack& track);
/// Bars from `entry` to the first drop (CHORUS or UP phrase) or first vocal,
/// in multiples of 4 (4..64); 0 if unknown.
int introBars(const MixTrack& track, double entry);
/// Where the outgoing track hands over: Outro cue, else OUTRO phrase, else the
/// first phrase boundary after the last vocal, else 32 bars before the end.
std::optional<MixOutRange> mixOutRange(const MixTrack& track);
/// When the incoming deck should start (position in the outgoing track) so
/// that its drop lands where the outgoing outro ends. Falls back to the start
/// of the range, or the next phrase boundary after `now`.
double mixTrigger(const MixTrack& outgoing, const MixOutRange& range, int incomingIntroBars, double now);
} // namespace vilelabox
