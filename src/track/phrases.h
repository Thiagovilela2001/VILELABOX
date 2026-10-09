#pragma once

#include <array>
#include <QVector>
#include "track/beats.h"

namespace vilelabox {
// Time-domain features survive changes to BPM and the downbeat reference.
struct PhraseFeature {
    double start = 0;
    double end = 0;
    double rms = 0;
    double bass = 0;
    double flux = 0;
    std::array<double, 12> chroma{};
    std::array<double, 6> timbre{};
};
enum class PhraseType { Unknown, Intro, Up, Chorus, Down, Verse, Outro };
struct Phrase {
    double start = 0;
    double end = 0;
    int bars = 0;
    PhraseType type = PhraseType::Unknown;
    double confidence = 0;
};
struct PhraseAnalysis {
    QVector<PhraseFeature> features;
    QVector<Phrase> phrases;
    bool lowConfidence = true;
    double duration = 0;
    int indexAt(double seconds) const;
};
// Pure segmentation. Grid positions use seconds; audio is never decoded here.
PhraseAnalysis segmentPhrases(const QVector<PhraseFeature>& features,
        double duration, const mixxx::BeatsPointer& beats,
        const QVector<QPair<double, double>>& vocals = {});
} // namespace vilelabox
