#include "library/related/relatedtracksfeature.h"
#include <QCheckBox>
#include <QComboBox>
#include <QHBoxLayout>
#include <QHeaderView>
#include <QLabel>
#include <QTableWidget>
#include <QVBoxLayout>
#include "library/library.h"
#include "library/libraryview.h"
#include <algorithm>
#include <functional>
#include "library/related/relatedtracks.h"
#include "library/trackcollection.h"
#include "library/trackcollectionmanager.h"
#include "mixer/playermanager.h"
#include "mixer/basetrackplayer.h"
#include "track/track.h"
#include "widget/wlibrary.h"
#include "moc_relatedtracksfeature.cpp"
namespace {
const ConfigKey kReferenceDeckKey("[Vilelabox]", "RelatedReferenceDeck");
const ConfigKey kPinnedKey("[Vilelabox]", "RelatedPinned");
const ConfigKey kLimitKey("[Vilelabox]", "RelatedLimit");
class RelatedView : public QWidget, public LibraryView {
  public:
    RelatedView(QWidget* parent, std::function<void()> shown) : QWidget(parent), m_shown(std::move(shown)) {}
    void onShow() override { m_shown(); }
    bool hasFocus() const override { return QWidget::hasFocus(); }
    void setFocus() override { QWidget::setFocus(); }
  private:
    std::function<void()> m_shown;
};
}
RelatedTracksFeature::RelatedTracksFeature(Library* library, UserSettingsPointer config, PlayerManager* players)
        : LibraryFeature(library, config, QStringLiteral("prepare")), m_sidebar(std::make_unique<TreeItemModel>()) {
    m_search = new vilelabox::RelatedSearch(
            library->trackCollectionManager()->internalCollection()->database().databaseName(),
            config->getSettingsPath(), this);
    connect(m_search, &vilelabox::RelatedSearch::finished, this, &RelatedTracksFeature::showRows);
    const int decks = players ? players->numberOfDecks() : 0;
    m_deckTracks.resize(decks);
    for (int i = 0; i < decks; ++i) {
        if (auto* player = players->getDeckBase(i)) connect(player, &BaseTrackPlayer::loadingTrack, this,
                [this, i](TrackPointer track, TrackPointer) { deckLoading(i, track); });
    }
}
int RelatedTracksFeature::limit() const {
    return std::clamp(m_pConfig->getValue(kLimitKey, 100), 1, 1000);
}
void RelatedTracksFeature::deckLoading(int deck, const TrackPointer& track) {
    if (deck >= 0 && deck < m_deckTracks.size()) m_deckTracks[deck] = track;
    const bool pinned = m_pinned ? m_pinned->isChecked() : m_pConfig->getValue(kPinnedKey, false);
    if (pinned) return;
    // 0 = the last deck that loaded a track; n = deck n only.
    const int reference = m_referenceDeck ? m_referenceDeck->currentIndex() : m_pConfig->getValue(kReferenceDeckKey, 0);
    if (reference != 0 && reference != deck + 1) return;
    m_reference = track;
    requestRefresh();
}
void RelatedTracksFeature::bindLibraryWidget(WLibrary* widget, KeyboardEventFilter* keyboard) {
    Q_UNUSED(keyboard);
    auto* view = new RelatedView(widget, [this] { if (m_pending) requestRefresh(); });
    m_view = view;
    auto* layout = new QVBoxLayout(view);
    auto* bar = new QHBoxLayout();
    m_label = new QLabel(tr("Carregue uma faixa em um deck para encontrar relacionadas."), view);
    m_referenceDeck = new QComboBox(view);
    m_referenceDeck->addItem(tr("Último deck carregado"));
    for (int i = 0; i < m_deckTracks.size(); ++i) m_referenceDeck->addItem(tr("Deck %1").arg(i + 1));
    m_referenceDeck->setCurrentIndex(std::clamp(m_pConfig->getValue(kReferenceDeckKey, 0), 0, m_referenceDeck->count() - 1));
    m_referenceDeck->setToolTip(tr("De qual deck vem a faixa de referência."));
    m_pinned = new QCheckBox(tr("Fixar"), view);
    m_pinned->setChecked(m_pConfig->getValue(kPinnedKey, false));
    m_pinned->setToolTip(tr("Mantém a referência atual mesmo ao carregar outras faixas."));
    bar->addWidget(m_label, 1);
    bar->addWidget(new QLabel(tr("Referência:"), view));
    bar->addWidget(m_referenceDeck);
    bar->addWidget(m_pinned);
    connect(m_referenceDeck, QOverload<int>::of(&QComboBox::currentIndexChanged), this, [this](int index) {
        m_pConfig->setValue(kReferenceDeckKey, index);
        if (index > 0 && index <= m_deckTracks.size() && m_deckTracks[index - 1] && !m_pinned->isChecked()) {
            m_reference = m_deckTracks[index - 1];
            requestRefresh();
        }
    });
    connect(m_pinned, &QCheckBox::toggled, this, [this](bool on) { m_pConfig->setValue(kPinnedKey, on); });
    m_table = new QTableWidget(view);
    m_table->setColumnCount(7);
    m_table->setHorizontalHeaderLabels({tr("Pontuação"), tr("Motivo"), tr("Artista"), tr("Título"), tr("BPM"), tr("Tom"), tr("Energia")});
    m_table->setEditTriggers(QAbstractItemView::NoEditTriggers);
    m_table->setSelectionBehavior(QAbstractItemView::SelectRows);
    m_table->horizontalHeader()->setSectionResizeMode(QHeaderView::ResizeToContents);
    m_table->horizontalHeader()->setStretchLastSection(true);
    layout->addLayout(bar); layout->addWidget(m_table);
    connect(m_table, &QTableWidget::cellDoubleClicked, this, [this](int row, int) {
        const auto id = TrackId(m_table->item(row, 0)->data(Qt::UserRole));
        auto track = m_pLibrary->trackCollectionManager()->getTrackById(id);
        if (track) emit loadTrack(track);
    });
    widget->registerView("VilelaboxRelated", view);
}
void RelatedTracksFeature::activate() {
    emit switchToView("VilelaboxRelated"); emit disableSearch();
    requestRefresh();
}
void RelatedTracksFeature::requestRefresh() {
    if (!m_table || !m_view || !m_view->isVisible()) {
        m_pending = true; // computed when the section is shown
        return;
    }
    m_pending = false;
    if (!m_reference) {
        m_table->setRowCount(0);
        return;
    }
    m_label->setText(tr("Buscando relacionadas a %1…").arg(m_reference->getTitle()));
    vilelabox::RelatedReference reference;
    reference.location = m_reference->getLocation();
    reference.bpm = m_reference->getBpm();
    reference.key = m_reference->getKey();
    m_search->start(reference, limit());
}
void RelatedTracksFeature::showRows(const QVector<vilelabox::RelatedRow>& rows, qint64 elapsedMs) {
    if (!m_table) return;
    m_label->setText(tr("Relacionadas a %1 — %2 sugestões em %3 ms — duplo clique para carregar")
                    .arg(m_reference ? m_reference->getTitle() : QString())
                    .arg(rows.size()).arg(elapsedMs));
    m_table->setUpdatesEnabled(false);
    m_table->setRowCount(rows.size());
    for (int i = 0; i < rows.size(); ++i) {
        const auto& row = rows[i];
        const QStringList values{QString::number(row.score.score, 'f', 0), row.score.reason, row.artist, row.title,
                QString::number(row.bpm, 'f', 1), row.key, row.energy > 0 ? QString::number(row.energy, 'f', 1) : QStringLiteral("—")};
        for (int j = 0; j < values.size(); ++j) m_table->setItem(i, j, new QTableWidgetItem(values[j]));
        m_table->item(i, 0)->setData(Qt::UserRole, row.id);
    }
    m_table->setUpdatesEnabled(true);
}
