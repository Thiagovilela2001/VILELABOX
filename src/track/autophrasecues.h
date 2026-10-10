#pragma once
#include <QMutex>
#include "track/track_decl.h"
#include "track/cue.h"
#include "track/phrases.h"

// One history per deck; undo removes only unchanged cues created by this instance.
class AutoPhraseCues {
  public:
    void setTrack(TrackPointer track);
    /// Returns true if the analysis was usable for this track (whether or not
    /// any cue had to be created), false if it was rejected.
    bool apply(const vilelabox::PhraseAnalysis& analysis, TrackPointer expectedTrack = {});
    void undo();
  private:
    struct Created {
        CuePointer cue;
        Cue::StartAndEndPositions positions;
        QString label;
        mixxx::RgbColor color;
    };
    QMutex m_mutex;
    TrackWeakPointer m_track;
    QList<Created> m_created;
};
