#include "track/mixpoints.h"
#include <algorithm>
#include <cmath>

namespace vilelabox {
namespace {
using mixxx::audio::FramePos;

double toSeconds(const mixxx::BeatsPointer& beats, FramePos position) {
    return position.value() / beats->getSampleRate();
}
FramePos toFrame(const mixxx::BeatsPointer& beats, double seconds) {
    return FramePos(seconds * beats->getSampleRate());
}
// Beat iterator at the start of the bar containing `seconds` (the beat itself
// when there is no bar reference).
mixxx::Beats::ConstIterator barStart(const mixxx::BeatsPointer& beats, double seconds) {
    auto it = beats->iteratorFrom(toFrame(beats, seconds));
    if (*it > toFrame(beats, seconds)) --it;
    if (const auto bar = beats->barPosition(*it)) it = it - (bar->beat - 1);
    return it;
}
double nearestBar(const mixxx::BeatsPointer& beats, double seconds) {
    auto before = barStart(beats, seconds);
    auto after = before + 4;
    const double a = toSeconds(beats, *before), b = toSeconds(beats, *after);
    return seconds - a <= b - seconds ? a : b;
}
// First 8-bar phrase boundary (counted from bar 1) strictly after `seconds`.
double nextPhraseBoundary(const mixxx::BeatsPointer& beats, double seconds) {
    auto it = barStart(beats, seconds);
    for (int i = 0; i < 64; ++i) {
        it += 4;
        const auto bar = beats->barPosition(*it);
        if (!bar || ((bar->bar - 1) % 8 + 8) % 8 == 0) break;
    }
    return toSeconds(beats, *it);
}
double moveBars(const mixxx::BeatsPointer& beats, double seconds, int bars) {
    return toSeconds(beats, *(barStart(beats, seconds) + bars * 4));
}
int barsBetween(const mixxx::BeatsPointer& beats, double from, double to) {
    return static_cast<int>((beats->iteratorFrom(toFrame(beats, to)) -
                                    beats->iteratorFrom(toFrame(beats, from))) / 4);
}
} // namespace

std::optional<double> mixInPoint(const MixTrack& track) {
    if (!track.beats) return std::nullopt;
    if (track.introCue) return nearestBar(track.beats, *track.introCue);
    if (track.beats->hasDownbeat()) {
        // Bar 1 is where the music starts (SET 1, set automatically from the
        // main cue or the downbeat detector).
        auto it = track.beats->iteratorFrom(mixxx::audio::kStartFramePos);
        for (int i = 0; i < 4096; ++i, ++it) {
            const auto bar = track.beats->barPosition(*it);
            if (bar && bar->bar == 1 && bar->beat == 1) {
                const double seconds = toSeconds(track.beats, *it);
                if (seconds >= 0 && (track.duration <= 0 || seconds < track.duration * 0.6)) return seconds;
                break;
            }
            if (bar && bar->bar > 1) break;
        }
    }
    if (track.mainCue) return nearestBar(track.beats, *track.mainCue);
    return std::nullopt;
}

int introBars(const MixTrack& track, double entry) {
    if (!track.beats) return 0;
    std::optional<double> drop;
    if (track.phrases) {
        for (const auto& phrase : track.phrases->phrases) {
            if (phrase.start > entry + 0.5 &&
                    (phrase.type == PhraseType::Chorus || phrase.type == PhraseType::Up)) {
                drop = phrase.start;
                break;
            }
        }
    }
    for (const auto& vocal : track.vocals) {
        if (vocal.first > entry + 0.5) {
            if (!drop || vocal.first < *drop) drop = vocal.first;
            break;
        }
    }
    if (!drop) return 0;
    const int bars = barsBetween(track.beats, entry, *drop) / 4 * 4;
    return bars < 4 ? 0 : std::min(bars, 64);
}

std::optional<MixOutRange> mixOutRange(const MixTrack& track) {
    if (!track.beats || track.duration <= 0) return std::nullopt;
    const double musicEnd = toSeconds(track.beats, *barStart(track.beats, track.duration));
    MixOutRange range{0, musicEnd};
    if (track.outroCue) {
        range.start = nearestBar(track.beats, *track.outroCue);
        return range.start < range.end ? std::optional(range) : std::nullopt;
    }
    if (track.phrases) {
        for (const auto& phrase : track.phrases->phrases) {
            if (phrase.type == PhraseType::Outro) {
                range.start = nearestBar(track.beats, phrase.start);
                range.end = std::min(musicEnd, nearestBar(track.beats, phrase.end));
                if (range.start < range.end) return range;
            }
        }
    }
    // Vocals finished: hand over at the next phrase boundary, so the two
    // tracks' vocals do not overlap.
    if (!track.vocals.isEmpty()) {
        const double boundary = nextPhraseBoundary(track.beats, track.vocals.last().second);
        if (barsBetween(track.beats, boundary, musicEnd) >= 8) {
            range.start = boundary;
            return range;
        }
    }
    range.start = moveBars(track.beats, musicEnd, -32);
    if (range.start < 0) range.start = 0;
    return range.start < range.end ? std::optional(range) : std::nullopt;
}

double mixTrigger(const MixTrack& outgoing, const MixOutRange& range, int incomingIntroBars, double now) {
    double trigger = range.start;
    if (outgoing.beats && incomingIntroBars > 0) {
        // The incoming drop lands where the outgoing outro ends, but never more
        // than 32 bars before the hand-over point.
        trigger = std::max(moveBars(outgoing.beats, range.end, -incomingIntroBars),
                moveBars(outgoing.beats, range.start, -32));
    }
    if (trigger > now) return trigger;
    if (range.start > now) return range.start;
    return outgoing.beats ? nextPhraseBoundary(outgoing.beats, now) : range.start;
}
} // namespace vilelabox
