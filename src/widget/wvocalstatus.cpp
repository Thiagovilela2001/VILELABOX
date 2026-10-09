#include "widget/wvocalstatus.h"
#include <QStyle>
#include "analyzer/vocalanalysisstore.h"
#include "mixer/playermanager.h"
#include "mixer/basetrackplayer.h"
#include "track/track.h"
#include "moc_wvocalstatus.cpp"
WVocalStatus::WVocalStatus(const QString& group, PlayerManager* manager, QWidget* parent)
        : WLabel(parent), m_position(group, "playposition", this) {
    if (auto* player = manager ? manager->getPlayer(group) : nullptr) {
        m_track = player->getLoadedTrack();
        connect(player, &BaseTrackPlayer::loadingTrack, this,
                [this](TrackPointer track, TrackPointer) { setTrack(track); });
    }
    m_position.connectValueChanged(this, &WVocalStatus::refresh);
    connect(&VocalAnalysisStore::instance(), &VocalAnalysisStore::updated, this,
            [this](const QString& path) {
                if (m_track && m_track->getLocation() == path) refresh();
            }, Qt::QueuedConnection);
    setToolTip(tr("Detecção experimental de vocal. Trechos prováveis aparecem em rosa."));
    refresh();
}
void WVocalStatus::setTrack(TrackPointer track) {
    m_track = std::move(track);
    refresh();
}
void WVocalStatus::setup(const QDomNode& node, const SkinContext& context) {
    WLabel::setup(node, context);
    refresh();
}
void WVocalStatus::refresh() {
    const auto analysis = m_track ? VocalAnalysisStore::instance().get(m_track->getLocation()) : nullptr;
    bool vocal = false;
    QString text;
    if (!m_track) text = QStringLiteral("—");
    else if (!analysis || analysis->status == VocalAnalysis::Status::Pending) text = tr("VOZ…");
    else if (analysis->status == VocalAnalysis::Status::Failed) text = tr("VOZ ?");
    else {
        vocal = analysis->hasVocal(m_position.get() * analysis->duration);
        text = vocal ? tr("VOCAL") : tr("SEM VOZ");
    }
    if (getHighlight() != (vocal ? 1 : 0)) {
        setHighlight(vocal ? 1 : 0);
        style()->unpolish(this);
        style()->polish(this);
    }
    setText(text);
}
