#pragma once

#include <QColor>

#include "skin/legacy/skincontext.h"
#include "util/class.h"
#include "waveform/renderers/waveformrendererabstract.h"

class WaveformRenderBeat : public WaveformRendererAbstract {
  public:
    explicit WaveformRenderBeat(WaveformWidgetRenderer* waveformWidgetRenderer);
    virtual ~WaveformRenderBeat();

    virtual void setup(const QDomNode& node, const SkinContext& context);
    virtual void draw(QPainter* painter, QPaintEvent* event);

  private:
    QColor m_beatColor;
    QColor m_barColor;
    bool m_showPhraseRegions = false;
    QString m_phraseColors;
    bool m_showVocalMarkers = false;
    bool m_showBarMarkers = false;
    QVector<QLineF> m_barBeats;
    QColor m_phraseColor;
    bool m_showPhraseMarkers = false;
    int m_phraseBars = 8;
    QVector<QLineF> m_phraseBeats;
    QVector<QLineF> m_beats;

    DISALLOW_COPY_AND_ASSIGN(WaveformRenderBeat);
};
