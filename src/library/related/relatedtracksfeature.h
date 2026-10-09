#pragma once
#include <QPointer>
#include "library/libraryfeature.h"
class PlayerManager;
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
    void refresh();
    std::unique_ptr<TreeItemModel> m_sidebar;
    TrackPointer m_reference;
    QPointer<QTableWidget> m_table;
    QPointer<QLabel> m_label;
};
