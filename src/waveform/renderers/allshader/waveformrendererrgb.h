#pragma once

#include <memory>

#include "shaders/rgbshader.h"
#include "util/class.h"
#include "waveform/renderers/allshader/rgbdata.h"
#include "waveform/renderers/allshader/vertexdata.h"
#include "waveform/renderers/allshader/waveformrenderersignalbase.h"

class ControlProxy;

namespace allshader {
class WaveformRendererRGB;
}

class allshader::WaveformRendererRGB final : public allshader::WaveformRendererSignalBase {
  public:
    explicit WaveformRendererRGB(WaveformWidgetRenderer* waveformWidget,
            ::WaveformRendererAbstract::PositionSource type =
                    ::WaveformRendererAbstract::Play);

    // override ::WaveformRendererSignalBase
    void onSetup(const QDomNode& node) override;

    void initializeGL() override;
    void paintGL() override;

  private:
    mixxx::RGBShader m_shader;
    VertexData m_vertices;
    RGBData m_colors;

    bool m_isSlipRenderer;
    // VILELABOX: <RgbGradient>true</RgbGradient> in the skin draws each column
    // as a low (centre) -> mid -> high (tip) colour gradient.
    bool m_gradient = false;
    // VILELABOX: [Vilelabox],waveform_rgb = 1 switches to the rekordbox RGB
    // look: low red, mid green, high blue, blended per column, no gradient.
    std::unique_ptr<ControlProxy> m_rgbMode;
    int m_rgbModeRetry = 0;

    DISALLOW_COPY_AND_ASSIGN(WaveformRendererRGB);
};
