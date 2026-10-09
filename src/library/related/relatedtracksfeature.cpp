#include "library/related/relatedtracksfeature.h"
#include <QHeaderView>
#include <QLabel>
#include <QSqlQuery>
#include <QTableWidget>
#include <QVBoxLayout>
#include "library/library.h"
#include "library/libraryview.h"
#include <algorithm>
#include "library/related/relatedtracks.h"
#include "library/trackcollection.h"
#include "library/trackcollectionmanager.h"
#include "mixer/playermanager.h"
#include "mixer/basetrackplayer.h"
#include "track/track.h"
#include "widget/wlibrary.h"
#include "moc_relatedtracksfeature.cpp"
namespace {
class RelatedView : public QWidget, public LibraryView {
  public:
    explicit RelatedView(QWidget* parent) : QWidget(parent) {}
    void onShow() override {}
    bool hasFocus() const override { return QWidget::hasFocus(); }
    void setFocus() override { QWidget::setFocus(); }
};
}
RelatedTracksFeature::RelatedTracksFeature(Library* library, UserSettingsPointer config, PlayerManager* players)
        : LibraryFeature(library, config, QStringLiteral("prepare")), m_sidebar(std::make_unique<TreeItemModel>()) {
    for (int i = 0; players && i < players->numberOfDecks(); ++i) {
        if (auto* player = players->getDeckBase(i)) connect(player, &BaseTrackPlayer::loadingTrack, this,
                [this](TrackPointer track, TrackPointer) { m_reference = track; refresh(); });
    }
}
void RelatedTracksFeature::bindLibraryWidget(WLibrary* widget, KeyboardEventFilter* keyboard) {
    Q_UNUSED(keyboard);
    auto* view = new RelatedView(widget);
    auto* layout = new QVBoxLayout(view);
    m_label = new QLabel(tr("Carregue uma faixa em um deck para encontrar relacionadas."), view);
    m_table = new QTableWidget(view);
    m_table->setColumnCount(7);
    m_table->setHorizontalHeaderLabels({tr("Pontuação"), tr("Motivo"), tr("Artista"), tr("Título"), tr("BPM"), tr("Tom"), tr("Energia")});
    m_table->setEditTriggers(QAbstractItemView::NoEditTriggers);
    m_table->setSelectionBehavior(QAbstractItemView::SelectRows);
    m_table->horizontalHeader()->setSectionResizeMode(QHeaderView::ResizeToContents);
    m_table->horizontalHeader()->setStretchLastSection(true);
    layout->addWidget(m_label); layout->addWidget(m_table);
    connect(m_table, &QTableWidget::cellDoubleClicked, this, [this](int row, int) {
        const auto id = TrackId(m_table->item(row, 0)->data(Qt::UserRole));
        auto track = m_pLibrary->trackCollectionManager()->getTrackById(id);
        if (track) emit loadTrack(track);
    });
    widget->registerView("VilelaboxRelated", view);
    refresh();
}
void RelatedTracksFeature::activate() {
    refresh(); emit switchToView("VilelaboxRelated"); emit disableSearch();
}
void RelatedTracksFeature::refresh() {
    if (!m_table) return;
    m_table->setRowCount(0);
    if (!m_reference) return;
    m_label->setText(tr("Relacionadas a %1 — duplo clique para carregar").arg(m_reference->getTitle()));
    const auto energies = vilelabox::loadEnergies(m_pConfig->getSettingsPath());
    const double referenceEnergy = energies.value(vilelabox::energyIdentity(m_reference->getLocation()));
    struct Row { int id; QString artist, title, key; double bpm, energy; vilelabox::RelatedScore score; };
    QVector<Row> rows;
    QSqlQuery query(m_pLibrary->trackCollectionManager()->internalCollection()->database());
    if (!query.exec("SELECT l.id,l.artist,l.title,l.bpm,l.key_id,t.location FROM library l JOIN track_locations t ON l.location=t.id WHERE t.fs_deleted=0 AND l.mixxx_deleted=0")) return;
    while (query.next()) {
        const auto location = query.value(5).toString();
        if (location == m_reference->getLocation()) continue;
        const double bpm = query.value(3).toDouble();
        const auto key = KeyUtils::keyFromNumericValue(query.value(4).toInt());
        const double energy = energies.value(vilelabox::energyIdentity(location));
        const auto score = vilelabox::relatedScore(m_reference->getBpm(), bpm, m_reference->getKey(), key, referenceEnergy, energy);
        if (score.score > 0) rows.push_back({query.value(0).toInt(), query.value(1).toString(), query.value(2).toString(),
                KeyUtils::keyToString(key, KeyUtils::KeyNotation::Lancelot), bpm, energy, score});
    }
    std::sort(rows.begin(), rows.end(), [](const Row& a, const Row& b) { return a.score.score > b.score.score; });
    m_table->setRowCount(rows.size());
    for (int i = 0; i < rows.size(); ++i) {
        const auto& row = rows[i];
        const QStringList values{QString::number(row.score.score, 'f', 0), row.score.reason, row.artist, row.title,
                QString::number(row.bpm, 'f', 1), row.key, row.energy > 0 ? QString::number(row.energy, 'f', 1) : QStringLiteral("—")};
        for (int j = 0; j < values.size(); ++j) m_table->setItem(i, j, new QTableWidgetItem(values[j]));
        m_table->item(i, 0)->setData(Qt::UserRole, row.id);
    }
}
