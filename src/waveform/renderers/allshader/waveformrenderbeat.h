#pragma once

#include <QColor>

#include "shaders/unicolorshader.h"
#include "waveform/renderers/allshader/digitsrenderer.h"
#include "util/class.h"
#include "waveform/renderers/allshader/vertexdata.h"
#include "waveform/renderers/allshader/waveformrenderer.h"

class QDomNode;
class SkinContext;

namespace allshader {
class WaveformRenderBeat;
}

class allshader::WaveformRenderBeat final : public allshader::WaveformRenderer {
  public:
    explicit WaveformRenderBeat(WaveformWidgetRenderer* waveformWidget,
            ::WaveformRendererAbstract::PositionSource type =
                    ::WaveformRendererAbstract::Play);

    void setup(const QDomNode& node, const SkinContext& context) override;
    void paintGL() override;
    void initializeGL() override;

  private:
    void drawBeatNumbers(const QMatrix4x4& matrix, float devicePixelRatio);

  private:
    mixxx::UnicolorShader m_shader;
    QColor m_color;
    QColor m_barColor;
    bool m_showPhraseRegions = false;
    QString m_phraseColors;
    bool m_showVocalMarkers = false;
    bool m_showBarMarkers = false;
    VertexData m_barVertices;
    QColor m_phraseColor;
    bool m_showPhraseMarkers = false;
    int m_phraseBars = 8;
    VertexData m_phraseVertices;
    VertexData m_vertices;
    // VILELABOX: <ShowBeatNumbers>true</ShowBeatNumbers> writes the beat in
    // the bar (1-4) at each beat line; beat 1 uses the larger digits.
    bool m_showBeatNumbers = false;
    DigitsRenderer m_beatDigits;
    DigitsRenderer m_downbeatDigits;
    struct BeatNumber {
        float x;
        int beat;
    };
    QVector<BeatNumber> m_beatNumbers;

    bool m_isSlipRenderer;

    DISALLOW_COPY_AND_ASSIGN(WaveformRenderBeat);
};
