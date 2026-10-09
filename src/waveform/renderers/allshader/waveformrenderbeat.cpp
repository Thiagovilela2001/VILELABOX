#include "waveform/renderers/allshader/waveformrenderbeat.h"

#include <QDomNode>

#include "skin/legacy/skincontext.h"
#include "track/track.h"
#include "analyzer/vocalanalysisstore.h"
#include "waveform/renderers/allshader/matrixforwidgetgeometry.h"
#include "waveform/renderers/waveformwidgetrenderer.h"
#include "widget/wskincolor.h"

namespace allshader {

WaveformRenderBeat::WaveformRenderBeat(WaveformWidgetRenderer* waveformWidget,
        ::WaveformRendererAbstract::PositionSource type)
        : WaveformRenderer(waveformWidget),
          m_isSlipRenderer(type == ::WaveformRendererAbstract::Slip) {
}

void WaveformRenderBeat::initializeGL() {
    WaveformRenderer::initializeGL();
    m_shader.init();
}

void WaveformRenderBeat::setup(const QDomNode& node, const SkinContext& context) {
    m_color = QColor(context.selectString(node, "BeatColor"));
    m_color = WSkinColor::getCorrectColor(m_color).toRgb();
    m_showVocalMarkers = context.selectBool(node, "ShowVocalMarkers", false);
    m_showBarMarkers = context.selectBool(node, "ShowBarMarkers", false);
    m_barColor = context.selectColor(node, "BarMarkerColor");
    if (!m_barColor.isValid()) {
        m_barColor = QColor("#ff8c1a");
    }
    m_barColor = WSkinColor::getCorrectColor(m_barColor).toRgb();
    m_showPhraseMarkers = context.selectBool(node, "ShowPhraseMarkers", false);
    bool phraseBarsOk = false;
    const int phraseBars = context.selectInt(node, "PhraseBars", &phraseBarsOk);
    m_phraseBars = phraseBarsOk && phraseBars > 0 ? phraseBars : 8;
    m_phraseColor = context.selectColor(node, "PhraseMarkerColor");
    if (!m_phraseColor.isValid()) {
        m_phraseColor = QColor("#ff3b30");
    }
    m_phraseColor = WSkinColor::getCorrectColor(m_phraseColor).toRgb();
}

void WaveformRenderBeat::paintGL() {
    TrackPointer trackInfo = m_waveformRenderer->getTrackInfo();

    if (!trackInfo || (m_isSlipRenderer && !m_waveformRenderer->isSlipActive())) {
        return;
    }

    auto positionType = m_isSlipRenderer ? ::WaveformRendererAbstract::Slip
                                         : ::WaveformRendererAbstract::Play;

    const auto vocals = VocalAnalysisStore::instance().get(trackInfo->getLocation());
    if (m_showVocalMarkers && !m_isSlipRenderer && vocals &&
            vocals->status == VocalAnalysis::Status::Ready && vocals->duration > 0 &&
            m_waveformRenderer->getTrackSamples() > 0) {
        VertexData vertices;
        for (const auto& segment : vocals->segments) {
            const float start = m_waveformRenderer->transformSamplePositionInRendererWorld(
                    segment.first / vocals->duration * m_waveformRenderer->getTrackSamples());
            const float end = m_waveformRenderer->transformSamplePositionInRendererWorld(
                    segment.second / vocals->duration * m_waveformRenderer->getTrackSamples());
            if (end < 0 || start > m_waveformRenderer->getLength()) continue;
            vertices.addRectangle(std::max(0.f, start), m_waveformRenderer->getBreadth() - 5,
                    std::min(float(m_waveformRenderer->getLength()), end), m_waveformRenderer->getBreadth());
        }
        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
        m_shader.bind();
        const int position = m_shader.attributeLocation("position");
        m_shader.enableAttributeArray(position);
        m_shader.setAttributeArray(position, GL_FLOAT, vertices.constData(), 2);
        m_shader.setUniformValue(m_shader.uniformLocation("matrix"), matrixForWidgetGeometry(m_waveformRenderer, false));
        m_shader.setUniformValue(m_shader.uniformLocation("color"), QColor("#ff63c6"));
        glDrawArrays(GL_TRIANGLES, 0, vertices.size());
        m_shader.disableAttributeArray(position);
        m_shader.release();
    }
    mixxx::BeatsPointer trackBeats = trackInfo->getBeats();
    if (!trackBeats) {
        return;
    }

    int alpha = m_waveformRenderer->getBeatGridAlpha();
    if (alpha == 0) {
        return;
    }

    const float devicePixelRatio = m_waveformRenderer->getDevicePixelRatio();

    glEnable(GL_BLEND);
    glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

    m_color.setAlphaF(alpha / 100.0f);
    m_barColor.setAlphaF(alpha / 100.0f);
    m_phraseColor.setAlphaF(alpha / 100.0f);

    const double trackSamples = m_waveformRenderer->getTrackSamples();
    if (trackSamples <= 0) {
        return;
    }

    const double firstDisplayedPosition =
            m_waveformRenderer->getFirstDisplayedPosition(positionType);
    const double lastDisplayedPosition =
            m_waveformRenderer->getLastDisplayedPosition(positionType);

    const auto startPosition = mixxx::audio::FramePos::fromEngineSamplePos(
            firstDisplayedPosition * trackSamples);
    const auto endPosition = mixxx::audio::FramePos::fromEngineSamplePos(
            lastDisplayedPosition * trackSamples);

    if (!startPosition.isValid() || !endPosition.isValid()) {
        return;
    }

    const float rendererBreadth = m_waveformRenderer->getBreadth();

    const int numVerticesPerLine = 6; // 2 triangles

    // Count the number of beats in the range to reserve space in the m_vertices vector.
    // Note that we could also use
    //   int numBearsInRange = trackBeats->numBeatsInRange(startPosition, endPosition);
    // for this, but there have been reports of that method failing with a DEBUG_ASSERT.
    int numBeatsInRange = 0;
    for (auto it = trackBeats->iteratorFrom(startPosition);
            it != trackBeats->cend() && *it <= endPosition;
            ++it) {
        numBeatsInRange++;
    }

    const int reserved = numBeatsInRange * numVerticesPerLine;
    m_vertices.clear();
    m_vertices.reserve(reserved);
    m_barVertices.clear();
    m_barVertices.reserve(reserved);
    m_phraseVertices.clear();
    m_phraseVertices.reserve(reserved);

    for (auto it = trackBeats->iteratorFrom(startPosition);
            it != trackBeats->cend() && *it <= endPosition;
            ++it) {
        double beatPosition = it->toEngineSamplePos();
        double xBeatPoint =
                m_waveformRenderer->transformSamplePositionInRendererWorld(
                        beatPosition, positionType);

        xBeatPoint = qRound(xBeatPoint * devicePixelRatio) / devicePixelRatio;

        const float x1 = static_cast<float>(xBeatPoint);
        const bool phrase = m_showPhraseMarkers && trackBeats->isDownbeat(it, m_phraseBars);
        const bool downbeat = !phrase && m_showBarMarkers && trackBeats->isDownbeat(it);
        const float x2 = x1 + (phrase ? 3.f : (downbeat ? 2.f : 1.f));
        auto& vertices = phrase ? m_phraseVertices
                                : (downbeat ? m_barVertices : m_vertices);

        vertices.addRectangle(x1,
                0.f,
                x2,
                m_isSlipRenderer ? rendererBreadth / 2 : rendererBreadth);
    }

    DEBUG_ASSERT(reserved ==
            m_vertices.size() + m_barVertices.size() + m_phraseVertices.size());

    const int positionLocation = m_shader.positionLocation();
    const int matrixLocation = m_shader.matrixLocation();
    const int colorLocation = m_shader.colorLocation();

    m_shader.bind();
    m_shader.enableAttributeArray(positionLocation);

    const QMatrix4x4 matrix = matrixForWidgetGeometry(m_waveformRenderer, false);

    m_shader.setAttributeArray(
            positionLocation, GL_FLOAT, m_vertices.constData(), 2);

    m_shader.setUniformValue(matrixLocation, matrix);
    m_shader.setUniformValue(colorLocation, m_color);

    glDrawArrays(GL_TRIANGLES, 0, m_vertices.size());
    if (m_barVertices.size() > 0) {
        m_shader.setAttributeArray(
                positionLocation, GL_FLOAT, m_barVertices.constData(), 2);
        m_shader.setUniformValue(colorLocation, m_barColor);
        glDrawArrays(GL_TRIANGLES, 0, m_barVertices.size());
    }
    if (m_phraseVertices.size() > 0) {
        m_shader.setAttributeArray(
                positionLocation, GL_FLOAT, m_phraseVertices.constData(), 2);
        m_shader.setUniformValue(colorLocation, m_phraseColor);
        glDrawArrays(GL_TRIANGLES, 0, m_phraseVertices.size());
    }

    m_shader.disableAttributeArray(positionLocation);
    m_shader.release();
}

} // namespace allshader
