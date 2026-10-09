#include "waveform/renderers/waveformrenderbeat.h"

#include <QPainter>

#include "track/track.h"
#include "analyzer/vocalanalysisstore.h"
#include "util/painterscope.h"
#include "waveform/renderers/waveformwidgetrenderer.h"
#include "widget/wskincolor.h"

class QPaintEvent;

WaveformRenderBeat::WaveformRenderBeat(WaveformWidgetRenderer* waveformWidgetRenderer)
        : WaveformRendererAbstract(waveformWidgetRenderer) {
    m_beats.resize(128);
}

WaveformRenderBeat::~WaveformRenderBeat() {
}

void WaveformRenderBeat::setup(const QDomNode& node, const SkinContext& context) {
    m_beatColor = QColor(context.selectString(node, "BeatColor"));
    m_beatColor = WSkinColor::getCorrectColor(m_beatColor).toRgb();
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

void WaveformRenderBeat::draw(QPainter* painter, QPaintEvent* /*event*/) {
    TrackPointer pTrackInfo = m_waveformRenderer->getTrackInfo();

    if (!pTrackInfo) {
        return;
    }

    const auto vocals = VocalAnalysisStore::instance().get(pTrackInfo->getLocation());
    if (m_showVocalMarkers && vocals && vocals->status == VocalAnalysis::Status::Ready &&
            vocals->duration > 0 && m_waveformRenderer->getTrackSamples() > 0) {
        for (const auto& segment : vocals->segments) {
            const double start = m_waveformRenderer->transformSamplePositionInRendererWorld(
                    segment.first / vocals->duration * m_waveformRenderer->getTrackSamples());
            const double end = m_waveformRenderer->transformSamplePositionInRendererWorld(
                    segment.second / vocals->duration * m_waveformRenderer->getTrackSamples());
            if (end < 0 || start > m_waveformRenderer->getLength()) continue;
            painter->fillRect(m_waveformRenderer->getOrientation() == Qt::Horizontal
                            ? QRectF(start, m_waveformRenderer->getHeight() - 5, end - start, 5)
                            : QRectF(m_waveformRenderer->getWidth() - 5, start, 5, end - start),
                    QColor("#ff63c6"));
        }
    }
    mixxx::BeatsPointer trackBeats = pTrackInfo->getBeats();
    if (!trackBeats) {
        return;
    }

    int alpha = m_waveformRenderer->getBeatGridAlpha();
    if (alpha == 0) {
        return;
    }
#ifdef MIXXX_USE_QOPENGL
    // Using alpha transparency with drawLines causes a graphical issue when
    // drawing with QPainter on the QOpenGLWindow: instead of individual lines
    // a large rectangle encompassing all beatlines is drawn.
    m_beatColor.setAlphaF(1.f);
    m_barColor.setAlphaF(1.f);
    m_phraseColor.setAlphaF(1.f);
#else
    m_beatColor.setAlphaF(alpha/100.0);
    m_barColor.setAlphaF(alpha/100.0);
    m_phraseColor.setAlphaF(alpha / 100.0);
#endif

    const double trackSamples = m_waveformRenderer->getTrackSamples();
    if (trackSamples <= 0) {
        return;
    }

    const float devicePixelRatio = m_waveformRenderer->getDevicePixelRatio();

    const double firstDisplayedPosition =
            m_waveformRenderer->getFirstDisplayedPosition();
    const double lastDisplayedPosition =
            m_waveformRenderer->getLastDisplayedPosition();

    // qDebug() << "trackSamples" << trackSamples
    //          << "firstDisplayedPosition" << firstDisplayedPosition
    //          << "lastDisplayedPosition" << lastDisplayedPosition;

    const auto startPosition = mixxx::audio::FramePos::fromEngineSamplePos(
            firstDisplayedPosition * trackSamples);
    const auto endPosition = mixxx::audio::FramePos::fromEngineSamplePos(
            lastDisplayedPosition * trackSamples);
    auto it = trackBeats->iteratorFrom(startPosition);

    // if no beat do not waste time saving/restoring painter
    if (it == trackBeats->cend() || *it > endPosition) {
        return;
    }

    PainterScope PainterScope(painter);

    painter->setRenderHint(QPainter::Antialiasing);

    QPen beatPen(m_beatColor);
    beatPen.setWidthF(std::max(1.0, scaleFactor()));
    painter->setPen(beatPen);

    const Qt::Orientation orientation = m_waveformRenderer->getOrientation();
    const float rendererWidth = m_waveformRenderer->getWidth();
    const float rendererHeight = m_waveformRenderer->getHeight();

    int beatCount = 0;
    m_barBeats.clear();
    m_phraseBeats.clear();

    for (; it != trackBeats->cend() && *it <= endPosition; ++it) {
        double beatPosition = it->toEngineSamplePos();
        double xBeatPoint =
                m_waveformRenderer->transformSamplePositionInRendererWorld(beatPosition);

        xBeatPoint = qRound(xBeatPoint * devicePixelRatio) / devicePixelRatio;

        // If we don't have enough space, double the size.
        if (beatCount >= m_beats.size()) {
            m_beats.resize(m_beats.size() * 2);
        }

        const QLineF line = orientation == Qt::Horizontal
                ? QLineF(xBeatPoint, 0.0f, xBeatPoint, rendererHeight)
                : QLineF(0.0f, xBeatPoint, rendererWidth, xBeatPoint);
        if (m_showPhraseMarkers && trackBeats->isDownbeat(it, m_phraseBars)) {
            m_phraseBeats.append(line);
        } else if (m_showBarMarkers && trackBeats->isDownbeat(it)) {
            m_barBeats.append(line);
        } else {
            m_beats[beatCount++] = line;
        }
    }

    // Make sure to use constData to prevent detaches!
    painter->drawLines(m_beats.constData(), beatCount);
    QPen barPen(m_barColor);
    barPen.setWidthF(std::max(2.0, 2.0 * scaleFactor()));
    painter->setPen(barPen);
    painter->drawLines(m_barBeats.constData(), m_barBeats.size());
    QPen phrasePen(m_phraseColor);
    phrasePen.setWidthF(std::max(3.0, 3.0 * scaleFactor()));
    painter->setPen(phrasePen);
    painter->drawLines(m_phraseBeats.constData(), m_phraseBeats.size());
}
