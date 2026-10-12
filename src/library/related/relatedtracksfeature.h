#pragma once
#include <QPointer>
#include "library/libraryfeature.h"
#include "library/related/relatedtracks.h"
class PlayerManager;
class QCheckBox;
class QComboBox;
class QTableWidget;
class QLabel;
class RelatedTracksFeature : public LibraryFeature {
    Q_OBJECT
  public:
    RelatedTracksFeature(Library* library, UserSettingsPointer config, PlayerManager* players);
    QVariant title() override { return tr("Relacionadas"); }
    TreeItemModel* sidebarModel() const override { return m_sidebar.get(); }
    void bindLibraryWidget(WLibrary* widget, KeyboardEventFilter* keyboard) override;
    void activate() override;
  private:
    void deckLoading(int deck, const TrackPointer& track);
    // Starts a search if the view is visible; otherwise marks it pending.
    void requestRefresh();
    void showRows(const QVector<vilelabox::RelatedRow>& rows, qint64 elapsedMs);
    int limit() const;
    std::unique_ptr<TreeItemModel> m_sidebar;
    vilelabox::RelatedSearch* m_search = nullptr;
    TrackPointer m_reference;
    QVector<TrackPointer> m_deckTracks;
    bool m_pending = false;
    QPointer<QWidget> m_view;
    QPointer<QTableWidget> m_table;
    QPointer<QLabel> m_label;
    QPointer<QComboBox> m_referenceDeck;
    QPointer<QCheckBox> m_pinned;
};
