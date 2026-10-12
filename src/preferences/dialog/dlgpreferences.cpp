#include "preferences/dialog/dlgpreferences.h"

#include <QDialog>
#include <QAbstractButton>
#include <QGroupBox>
#include <QLabel>
#include <QLineEdit>
#include <QPalette>
#include <QSignalBlocker>
#include <QShortcut>
#include <QTabBar>
#include <QTabWidget>
#include <QVBoxLayout>
#include <algorithm>
#include <functional>
#include <QEvent>
#include <QMoveEvent>
#include <QResizeEvent>
#include <QScreen>
#include <QScrollArea>
#include <QtGlobal>

#include "controllers/dlgprefcontrollers.h"
#include "library/library.h"
#include "library/trackcollectionmanager.h"
#include "moc_dlgpreferences.cpp"
#include "preferences/dialog/dlgpreflibrary.h"
#include "preferences/dialog/dlgprefsound.h"
#include "util/color/color.h"
#include "util/desktophelper.h"
#include "util/widgethelper.h"

#ifdef __VINYLCONTROL__
#include "preferences/dialog/dlgprefvinyl.h"
#endif // __VINYLCONTROL__

#include "preferences/dialog/dlgprefautodj.h"
#include "preferences/dialog/dlgprefcolors.h"
#include "preferences/dialog/dlgprefdeck.h"
#include "preferences/dialog/dlgprefeffects.h"
#include "preferences/dialog/dlgprefinterface.h"
#include "preferences/dialog/dlgprefmixer.h"
#include "preferences/dialog/dlgprefwaveform.h"
#include "preferences/dialog/dlgprefvilelabox.h"

#ifdef __BROADCAST__
#include "preferences/dialog/dlgprefbroadcast.h"
#endif // __BROADCAST__

#include "preferences/dialog/dlgprefbeats.h"
#include "preferences/dialog/dlgprefkey.h"
#include "preferences/dialog/dlgprefrecord.h"
#include "preferences/dialog/dlgprefreplaygain.h"

#ifdef __MODPLUG__
#include "preferences/dialog/dlgprefmodplug.h"
#endif // __MODPLUG__

#ifdef Q_OS_MACOS
#include "util/darkappearance.h"
#endif

DlgPreferences::DlgPreferences(
        std::shared_ptr<mixxx::ScreensaverManager> pScreensaverManager,
        std::shared_ptr<mixxx::skin::SkinLoader> pSkinLoader,
        std::shared_ptr<SoundManager> pSoundManager,
        std::shared_ptr<ControllerManager> pControllerManager,
        std::shared_ptr<VinylControlManager> pVCManager,
        std::shared_ptr<EffectsManager> pEffectsManager,
        std::shared_ptr<SettingsManager> pSettingsManager,
        std::shared_ptr<Library> pLibrary)
        : m_allPages(),
          m_pConfig(pSettingsManager->settings()),
          m_pageSizeHint(QSize(0, 0)) {
    setupUi(this);
    fixSliderStyle();
    setupSettingsInterface();
    contentsTreeWidget->setHeaderHidden(true);

    // Add '&' to default button labels to always have Alt shortcuts, indpependent
    // of operating system.
    //: Preferences standard buttons: consider the other buttons to choose a unique Alt hotkey (&)
    buttonBox->button(QDialogButtonBox::Help)->setText(tr("&Help"));
    //: Preferences standard buttons: consider the other buttons to choose a unique Alt hotkey (&)
    buttonBox->button(QDialogButtonBox::RestoreDefaults)->setText(tr("&Restore Defaults"));
    //: Preferences standard buttons: consider the other buttons to choose a unique Alt hotkey (&)
    buttonBox->button(QDialogButtonBox::Apply)->setText(tr("&Apply"));
    //: Preferences standard buttons: consider the other buttons to choose a unique Alt hotkey (&)
    buttonBox->button(QDialogButtonBox::Cancel)->setText(tr("&Cancel"));
    //: Preferences standard buttons: consider the other buttons to choose a unique Alt hotkey (&)
    buttonBox->button(QDialogButtonBox::Ok)->setText(tr("&Ok"));

    connect(buttonBox,
            QOverload<QAbstractButton*>::of(&QDialogButtonBox::clicked),
            this,
            &DlgPreferences::slotButtonPressed);

    connect(contentsTreeWidget,
            &QTreeWidget::currentItemChanged,
            this,
            &DlgPreferences::changePage);

    while (pagesWidget->count() > 0) {
        pagesWidget->removeWidget(pagesWidget->currentWidget());
    }

    // Check the text color of the palette for whether to use dark or light icons
    if (!Color::isDimColor(palette().text().color())) {
        m_iconsPath.setPath(":/images/preferences/light/");
    } else {
        m_iconsPath.setPath(":/images/preferences/dark/");
    }

    // Construct widgets for use in tabs.
    m_soundPage = PreferencesPage(
            new DlgPrefSound(this, pSoundManager, m_pConfig),
            new QTreeWidgetItem(contentsTreeWidget, QTreeWidgetItem::Type));
    addPageWidget(m_soundPage,
            tr("Sound Hardware"),
            "ic_preferences_soundhardware.svg",
            tr("Áudio"));

    DlgPrefLibrary* plibraryPage = new DlgPrefLibrary(this, m_pConfig, pLibrary);
    connect(plibraryPage,
            &DlgPrefLibrary::scanLibrary,
            pLibrary->trackCollectionManager(),
            &TrackCollectionManager::startLibraryScan);
    addPageWidget(PreferencesPage(plibraryPage,
                          new QTreeWidgetItem(contentsTreeWidget, QTreeWidgetItem::Type)),
            tr("Library"),
            "ic_preferences_library.svg",
            tr("Biblioteca"));

    QTreeWidgetItem* pControllerRootItem =
            new QTreeWidgetItem(contentsTreeWidget, QTreeWidgetItem::Type);
    m_pControllersDlg = new DlgPrefControllers(
            this, m_pConfig, pControllerManager, pControllerRootItem);
    addPageWidget(PreferencesPage(m_pControllersDlg,
                          pControllerRootItem),
            tr("Controllers"),
            "ic_preferences_controllers.svg",
            tr("Controladoras"));

#ifdef __VINYLCONTROL__
    // It's important for this to be before the connect for wsound.
    // TODO(rryan) determine why/if this is still true
    addPageWidget(PreferencesPage(
                          new DlgPrefVinyl(this, pVCManager, m_pConfig),
                          new QTreeWidgetItem(contentsTreeWidget, QTreeWidgetItem::Type)),
            tr("Vinyl Control"),
            "ic_preferences_vinyl.svg",
            tr("Controladoras"));
#endif // __VINYLCONTROL__

    DlgPrefInterface* pInterfacePage = new DlgPrefInterface(this,
            pScreensaverManager,
            pSkinLoader,
            m_pConfig);
    connect(pInterfacePage,
            &DlgPrefInterface::tooltipModeChanged,
            this,
            &DlgPreferences::tooltipModeChanged);
    connect(pInterfacePage,
            &DlgPrefInterface::reloadUserInterface,
            this,
            &DlgPreferences::reloadUserInterface,
            Qt::DirectConnection);
    connect(pInterfacePage,
            &DlgPrefInterface::menuBarAutoHideChanged,
            this,
            &DlgPreferences::menuBarAutoHideChanged,
            Qt::DirectConnection);
    addPageWidget(PreferencesPage(pInterfacePage,
                          new QTreeWidgetItem(
                                  contentsTreeWidget, QTreeWidgetItem::Type)),
            tr("Interface"),
            "ic_preferences_interface.svg",
            tr("Visualização"));

    // ugly proxy for determining whether this is being instantiated for QML or legacy QWidgets GUI
    if (pSkinLoader) {
        addPageWidget(PreferencesPage(
                              new DlgPrefWaveform(this, m_pConfig, pLibrary),
                              new QTreeWidgetItem(contentsTreeWidget, QTreeWidgetItem::Type)),
                tr("Waveforms"),
                "ic_preferences_waveforms.svg",
            tr("Visualização"));
    }

    addPageWidget(PreferencesPage(
                          new DlgPrefColors(this, m_pConfig, pLibrary),
                          new QTreeWidgetItem(contentsTreeWidget, QTreeWidgetItem::Type)),
            tr("Colors"),
            "ic_preferences_colors.svg",
            tr("Visualização"));

    addPageWidget(PreferencesPage(
                          new DlgPrefDeck(this, m_pConfig),
                          new QTreeWidgetItem(contentsTreeWidget, QTreeWidgetItem::Type)),
            tr("Decks"),
            "ic_preferences_decks.svg",
            tr("Performance"));

    addPageWidget(PreferencesPage(
                          new DlgPrefMixer(this, pEffectsManager, m_pConfig),
                          new QTreeWidgetItem(contentsTreeWidget, QTreeWidgetItem::Type)),
            tr("Mixer"),
            "ic_preferences_crossfader.svg",
            tr("Performance"));

    addPageWidget(PreferencesPage(
                          new DlgPrefEffects(this, m_pConfig, pEffectsManager),
                          new QTreeWidgetItem(contentsTreeWidget, QTreeWidgetItem::Type)),
            tr("Effects"),
            "ic_preferences_effects.svg",
            tr("Performance"));

    addPageWidget(PreferencesPage(
                          new DlgPrefAutoDJ(this, m_pConfig),
                          new QTreeWidgetItem(contentsTreeWidget, QTreeWidgetItem::Type)),
            tr("Auto DJ"),
            "ic_preferences_autodj.svg",
            tr("Performance"));

#ifdef __BROADCAST__
    addPageWidget(PreferencesPage(
                          new DlgPrefBroadcast(this, pSettingsManager->broadcastSettings()),
                          new QTreeWidgetItem(contentsTreeWidget, QTreeWidgetItem::Type)),
            tr("Live Broadcasting"),
            "ic_preferences_broadcast.svg",
            tr("Gravação e transmissão"));
#endif // __BROADCAST__

    addPageWidget(PreferencesPage(
                          new DlgPrefRecord(this, m_pConfig),
                          new QTreeWidgetItem(contentsTreeWidget, QTreeWidgetItem::Type)),
            tr("Recording"),
            "ic_preferences_recording.svg",
            tr("Gravação e transmissão"));

    addPageWidget(PreferencesPage(
                          new DlgPrefBeats(this, m_pConfig),
                          new QTreeWidgetItem(contentsTreeWidget, QTreeWidgetItem::Type)),
            tr("Beat Detection"),
            "ic_preferences_bpmdetect.svg",
            tr("Análise"));

    addPageWidget(PreferencesPage(
                          new DlgPrefKey(this, m_pConfig),
                          new QTreeWidgetItem(contentsTreeWidget, QTreeWidgetItem::Type)),
            tr("Key Detection"),
            "ic_preferences_keydetect.svg",
            tr("Análise"));
    addPageWidget(PreferencesPage(
                          new DlgPrefReplayGain(this, m_pConfig),
                          new QTreeWidgetItem(contentsTreeWidget, QTreeWidgetItem::Type)),
            tr("Normalization"),
            "ic_preferences_replaygain.svg",
            tr("Análise"));

#ifdef __MODPLUG__
    addPageWidget(PreferencesPage(
                          new DlgPrefModplug(this, m_pConfig),
                          new QTreeWidgetItem(contentsTreeWidget, QTreeWidgetItem::Type)),
            tr("Modplug Decoder"),
            "ic_preferences_modplug.svg",
            tr("Áudio"));
#endif // __MODPLUG__

    addPageWidget(PreferencesPage(new DlgPrefVilelabox(this, m_pConfig),
                          new QTreeWidgetItem(contentsTreeWidget)),
            tr("Painéis e opções do mod"), "ic_preferences_interface.svg", tr("Visualização"));

    // Find accept and apply buttons
    const auto buttons = buttonBox->buttons();
    for (QAbstractButton* button : buttons) {
        QDialogButtonBox::ButtonRole role = buttonBox->buttonRole(button);
        if (role == QDialogButtonBox::ButtonRole::ApplyRole) {
            m_pApplyButton = button;
        } else if (role == QDialogButtonBox::ButtonRole::AcceptRole) {
            m_pAcceptButton = button;
        }
    }

    labelWarning->hide();
    labelWarningIcon->hide();
    labelWarning->setText(tr(
            "<font color='#BB0000'><b>Some preferences pages have errors. "
            "To apply the changes please first fix the issues.</b></font>"));
    QIcon icon = style()->standardIcon(QStyle::SP_MessageBoxWarning);
    labelWarningIcon->setPixmap(icon.pixmap(16));

    // Install event handler to generate closeDlg signal
    installEventFilter(this);

    // If we don't call this explicitly, then we default to showing the sound
    // hardware page but the tree item is not selected.
    showSoundHardwarePage();
}

void DlgPreferences::setupSettingsInterface() {
    setWindowTitle(tr("VILELABOX — Configurações"));
    // Keep the existing functional pages and their Apply/Cancel lifecycle.
    // This shell supplies category navigation, section tabs and discovery.
    QPalette colors = palette();
    colors.setColor(QPalette::Window, QColor("#202226"));
    colors.setColor(QPalette::WindowText, QColor("#e5e7eb"));
    colors.setColor(QPalette::Base, QColor("#17191c"));
    colors.setColor(QPalette::AlternateBase, QColor("#282b30"));
    colors.setColor(QPalette::Text, QColor("#e5e7eb"));
    colors.setColor(QPalette::Button, QColor("#30343a"));
    colors.setColor(QPalette::ButtonText, QColor("#e5e7eb"));
    colors.setColor(QPalette::Highlight, QColor("#236da2"));
    colors.setColor(QPalette::HighlightedText, Qt::white);
    colors.setColor(QPalette::Link, QColor("#65baff"));
    colors.setColor(QPalette::Disabled, QPalette::Text, QColor("#818791"));
    colors.setColor(QPalette::Disabled, QPalette::ButtonText, QColor("#818791"));
    colors.setColor(QPalette::Disabled, QPalette::WindowText, QColor("#818791"));
    setPalette(colors);
    setStyleSheet(styleSheet() + QStringLiteral(R"(
        QDialog#DlgPreferencesDlg { background: #202226; }
        QDialog#DlgPreferencesDlg QWidget { background-color: #202226; color: #e5e7eb; }
        QDialog#DlgPreferencesDlg QWidget:disabled { color: #818791; }
        QDialog#DlgPreferencesDlg QLabel { background: transparent; }
        QDialog#DlgPreferencesDlg QLineEdit,
        QDialog#DlgPreferencesDlg QComboBox,
        QDialog#DlgPreferencesDlg QSpinBox,
        QDialog#DlgPreferencesDlg QDoubleSpinBox,
        QDialog#DlgPreferencesDlg QTextEdit,
        QDialog#DlgPreferencesDlg QPlainTextEdit {
            background: #17191c; color: #e5e7eb; border: 1px solid #454b55;
            border-radius: 3px; padding: 4px;
        }
        QDialog#DlgPreferencesDlg QLineEdit:disabled,
        QDialog#DlgPreferencesDlg QComboBox:disabled,
        QDialog#DlgPreferencesDlg QSpinBox:disabled,
        QDialog#DlgPreferencesDlg QDoubleSpinBox:disabled { color: #818791; background: #282b30; }
        QDialog#DlgPreferencesDlg QAbstractItemView {
            background: #17191c; color: #e5e7eb; selection-background-color: #245c83;
        }
        QDialog#DlgPreferencesDlg QHeaderView::section {
            background: #30343a; color: #e5e7eb; padding: 5px; border: 1px solid #454b55;
        }
        QDialog#DlgPreferencesDlg QPushButton {
            background: #30343a; color: #e5e7eb; border: 1px solid #555d68;
            border-radius: 4px; padding: 6px 12px; min-height: 20px;
        }
        QDialog#DlgPreferencesDlg QPushButton:hover { background: #3c444f; }
        QDialog#DlgPreferencesDlg QPushButton:pressed { background: #245c83; }
        QDialog#DlgPreferencesDlg QPushButton:default { border-color: #65baff; }
        QDialog#DlgPreferencesDlg QPushButton:disabled { color: #818791; border-color: #363a41; }
        QDialog#DlgPreferencesDlg QTabWidget::pane { border: 1px solid #454b55; }
        QDialog#DlgPreferencesDlg QTabBar::tab {
            background: #282b30; color: #b8bec8; padding: 8px 12px;
            border-bottom: 2px solid #454b55;
        }
        QDialog#DlgPreferencesDlg QTabBar::tab:selected { color: white; border-bottom-color: #65baff; }
        QDialog#DlgPreferencesDlg QCheckBox::indicator {
            width: 15px; height: 15px; border: 1px solid #738392;
            border-radius: 3px; background: #17191c;
        }
        QDialog#DlgPreferencesDlg QCheckBox::indicator:checked {
            background: #236da2; border-color: #65baff; image: url(:/images/ic_checkmark.svg);
        }
        QDialog#DlgPreferencesDlg QCheckBox::indicator:indeterminate {
            background: #56606b; image: url(:/images/ic_checkmark.svg);
        }
        QDialog#DlgPreferencesDlg QCheckBox::indicator:disabled { border-color: #454b55; background: #30343a; }
        QDialog#DlgPreferencesDlg QScrollBar:vertical { background: #202226; width: 12px; }
        QDialog#DlgPreferencesDlg QScrollBar:horizontal { background: #202226; height: 12px; }
        QDialog#DlgPreferencesDlg QScrollBar::handle { background: #56606b; border-radius: 4px; min-height: 24px; min-width: 24px; }
        QDialog#DlgPreferencesDlg QScrollBar::add-line,
        QDialog#DlgPreferencesDlg QScrollBar::sub-line { width: 0; height: 0; }
        QTreeWidget#contentsTreeWidget {
            background: #17191c; border: 1px solid #363a41;
            border-radius: 6px; padding: 8px 2px;
        }
        QTreeWidget#contentsTreeWidget::item { min-height: 30px; padding: 2px 6px; }
        QTreeWidget#contentsTreeWidget::item:selected { background: #245c83; color: white; }
        QTreeWidget#contentsTreeWidget::item:hover:!selected { background: #2c3036; }
        QLabel#settingsBrand { font-size: 20px; font-weight: bold; color: #f5f6f8; }
        QLabel#settingsSubtitle { color: #a7aeb8; }
        QLabel#settingsPageHeading { font-size: 18px; font-weight: bold; padding: 6px 0; }
        QLineEdit#settingsSearch {
            background: #17191c; color: #e5e7eb; border: 1px solid #454b55;
            border-radius: 5px; padding: 8px 12px; min-width: 230px;
        }
        QLineEdit#settingsSearch:focus { border-color: #65baff; }
        QTabBar#settingsSections::tab {
            background: transparent; color: #b8bec8; padding: 10px 14px;
            border-bottom: 2px solid #3a3e45;
        }
        QTabBar#settingsSections::tab:selected { color: white; border-bottom: 2px solid #65baff; }
        QGroupBox { border: 1px solid #454b55; border-radius: 5px; margin-top: 14px; padding-top: 14px; }
        QGroupBox::title { subcontrol-origin: margin; left: 10px; padding: 0 5px; color: #b8d8f0; }
        QPushButton { padding: 6px 12px; min-height: 20px; }
        QDialogButtonBox QPushButton { min-width: 84px; }
        QSplitter::handle { background: #202226; }
    )"));

    horizontalLayout->setDirection(QBoxLayout::TopToBottom);
    horizontalLayout->setContentsMargins(16, 14, 16, 14);
    horizontalLayout->setSpacing(14);
    auto* header = new QHBoxLayout();
    auto* identity = new QVBoxLayout();
    auto* brand = new QLabel(tr("VILELABOX  /  Configurações"), this);
    brand->setObjectName("settingsBrand");
    auto* subtitle = new QLabel(tr("Áudio, visualização, análise e performance em um só lugar."), this);
    subtitle->setObjectName("settingsSubtitle");
    identity->addWidget(brand);
    identity->addWidget(subtitle);
    header->addLayout(identity, 1);
    m_search = new QLineEdit(this);
    m_search->setObjectName("settingsSearch");
    m_search->setPlaceholderText(tr("Buscar configurações…"));
    m_search->setAccessibleName(tr("Buscar configurações"));
    m_search->setClearButtonEnabled(true);
    header->addWidget(m_search);
    horizontalLayout->insertLayout(0, header);
    contentsTreeWidget->setMinimumWidth(210);
    contentsTreeWidget->setMaximumWidth(300);
    contentsTreeWidget->setIndentation(12);
    contentsTreeWidget->setIconSize(QSize(20, 20));
    contentsTreeWidget->setAccessibleName(tr("Categorias de configurações"));
    splitter->setStretchFactor(0, 0);
    splitter->setStretchFactor(1, 1);
    splitter->setSizes({230, 800});

    auto* pageLayout = qobject_cast<QVBoxLayout*>(layoutWidget->layout());
    m_pageHeading = new QLabel(this);
    m_pageHeading->setObjectName("settingsPageHeading");
    m_sectionTabs = new QTabBar(this);
    m_sectionTabs->setObjectName("settingsSections");
    m_sectionTabs->setExpanding(false);
    m_sectionTabs->setUsesScrollButtons(true);
    m_sectionTabs->setAccessibleName(tr("Seções da categoria"));
    pageLayout->insertWidget(0, m_sectionTabs);
    pageLayout->insertWidget(0, m_pageHeading);
    connect(m_search, &QLineEdit::textChanged, this, &DlgPreferences::filterPages);
    auto* find = new QShortcut(QKeySequence::Find, this);
    connect(find, &QShortcut::activated, this, [this] {
        m_search->setFocus();
        m_search->selectAll();
    });
    connect(m_sectionTabs, &QTabBar::currentChanged, this, [this](int index) {
        if (index >= 0 && index < m_tabItems.size()) {
            contentsTreeWidget->setCurrentItem(m_tabItems[index]);
        }
    });
}

void DlgPreferences::updateSectionTabs(QTreeWidgetItem* item) {
    const QSignalBlocker blocker(m_sectionTabs);
    while (m_sectionTabs->count()) m_sectionTabs->removeTab(0);
    m_tabItems.clear();
    auto* parent = item ? item->parent() : nullptr;
    if (!parent) return;
    for (int i = 0; i < parent->childCount(); ++i) {
        auto* section = parent->child(i);
        if (section->isHidden()) continue;
        m_tabItems.append(section);
        const int index = m_sectionTabs->addTab(section->text(0));
        if (section == item) m_sectionTabs->setCurrentIndex(index);
    }
}

void DlgPreferences::filterPages(const QString& query) {
    const QString term = query.trimmed();
    QHash<QTreeWidgetItem*, QString> searchable;
    for (const auto& page : std::as_const(m_allPages)) {
        QString text = page.pTreeItem->text(0);
        for (auto* label : page.pDlg->findChildren<QLabel*>()) text += " " + label->text();
        for (auto* button : page.pDlg->findChildren<QAbstractButton*>()) text += " " + button->text();
        for (auto* group : page.pDlg->findChildren<QGroupBox*>()) text += " " + group->title();
        searchable.insert(page.pTreeItem, text);
    }
    std::function<bool(QTreeWidgetItem*, bool)> filter = [&](QTreeWidgetItem* item, bool parentMatches) {
        const bool matches = term.isEmpty() || parentMatches ||
                searchable.value(item, item->text(0)).contains(term, Qt::CaseInsensitive);
        bool visible = matches;
        for (int i = 0; i < item->childCount(); ++i) visible = filter(item->child(i), matches) || visible;
        item->setHidden(!visible);
        if (visible && item->childCount()) item->setExpanded(true);
        return visible;
    };
    for (int i = 0; i < contentsTreeWidget->topLevelItemCount(); ++i) {
        filter(contentsTreeWidget->topLevelItem(i), false);
    }
    auto* selected = contentsTreeWidget->currentItem();
    if (selected && !selected->isHidden()) {
        changePage(selected, nullptr);
        return;
    }
    for (const auto& page : std::as_const(m_allPages)) {
        if (!page.pTreeItem->isHidden()) {
            contentsTreeWidget->setCurrentItem(page.pTreeItem);
            return;
        }
    }
    // Keep unsaved inputs intact when a search has no matching section.
    m_pageHeading->setText(tr("Nenhuma configuração encontrada"));
    m_sectionTabs->hide();
    pagesWidget->hide();
}

DlgPreferences::~DlgPreferences() {
    // store last geometry in mixxx.cfg
    if (m_geometry.size() == 4) {
        m_pConfig->set(ConfigKey("[Preferences]", "geometry"),
                m_geometry.join(","));
    }

    // When DlgPrefControllers is deleted it manually deletes the controller tree items,
    // which makes QTreeWidgetItem trigger this signal. If we don't disconnect,
    // &DlgPreferences::changePage iterates on the PreferencesPage instances in m_allPages,
    // but the pDlg objects of the controller items are already destroyed by DlgPrefControllers,
    // which causes a crash when accessed.
    disconnect(contentsTreeWidget, &QTreeWidget::currentItemChanged, this, &DlgPreferences::changePage);
    // Need to explicitly delete rather than relying on child auto-deletion
    // because otherwise the QStackedWidget will delete the controller
    // preference pages (and DlgPrefControllers dynamically generates and
    // deletes them).
    delete m_pControllersDlg;
}

void DlgPreferences::changePage(QTreeWidgetItem* pCurrent, QTreeWidgetItem* pPrevious) {
    if (!pCurrent) {
        pCurrent = pPrevious;
    }

    if (!pCurrent) return;
    // Category entries select their first visible settings section.
    if (pCurrent->data(0, Qt::UserRole).toBool()) {
        for (int i = 0; i < pCurrent->childCount(); ++i) {
            if (!pCurrent->child(i)->isHidden()) {
                contentsTreeWidget->setCurrentItem(pCurrent->child(i));
                return;
            }
        }
        return;
    }
    updateSectionTabs(pCurrent);
    if (m_pControllersDlg->handleTreeItemClick(pCurrent)) {
        // Do nothing. m_controllersPage handled this click.
        return;
    }

    for (PreferencesPage page : std::as_const(m_allPages)) {
        if (pCurrent == page.pTreeItem) {
            switchToPage(pCurrent->text(0), page.pDlg);
            break;
        }
    }
}

void DlgPreferences::showSoundHardwarePage() {
    m_search->clear();
    switchToPage(m_soundPage.pTreeItem->text(0), m_soundPage.pDlg);
    contentsTreeWidget->setCurrentItem(m_soundPage.pTreeItem);
}

bool DlgPreferences::eventFilter(QObject* o, QEvent* e) {
    // Send a close signal if dialog is closing
    if (e->type() == QEvent::Hide) {
        onHide();
    }

    if (e->type() == QEvent::Show) {
        onShow();
    }

    // Standard event processing
    return QWidget::eventFilter(o, e);
}

void DlgPreferences::onHide() {
    // Notify children that we are about to hide.
    emit closeDlg();
}

void DlgPreferences::onShow() {
    // init m_geometry
    if (m_geometry.length() < 4) {
        // load default values (optimum size)
        m_geometry = m_pConfig->getValue(
                                      ConfigKey("[Preferences]", "geometry"))
                             .split(",");
        if (m_geometry.length() < 4) {
            // Warning! geometry does NOT include the frame/title.
            QRect defaultGeometry = getDefaultGeometry();
            m_geometry.clear();
            m_geometry.append(QString::number(defaultGeometry.left()));
            m_geometry.append(QString::number(defaultGeometry.top()));
            m_geometry.append(QString::number(defaultGeometry.width()));
            m_geometry.append(QString::number(defaultGeometry.height()));
        }
    }
    int newX = m_geometry[0].toInt();
    int newY = m_geometry[1].toInt();
    int newWidth = m_geometry[2].toInt();
    int newHeight = m_geometry[3].toInt();

    const QScreen* const pScreen = mixxx::widgethelper::getScreen(*this);
    QRect screenAvailableGeometry;
    VERIFY_OR_DEBUG_ASSERT(pScreen) {
        qWarning() << "Assuming screen size of 800x600px.";
        screenAvailableGeometry = QRect(0, 0, 800, 600);
    }
    else {
        screenAvailableGeometry = pScreen->availableGeometry();
    }

    // Make sure the entire window is visible on screen and is not occluded by taskbar
    // Note: Window geometry excludes window decoration
    int windowDecorationWidth = frameGeometry().width() - geometry().width();
    int windowDecorationHeight = frameGeometry().height() - geometry().height();
    if (windowDecorationWidth <= 0) {
        windowDecorationWidth = 2;
    }
    if (windowDecorationHeight <= 0) {
        windowDecorationHeight = 30;
    }
    int availableWidth = screenAvailableGeometry.width() - windowDecorationWidth;
    int availableHeight = screenAvailableGeometry.height() - windowDecorationHeight;
    setMinimumSize(std::min(900, availableWidth), std::min(600, availableHeight));
    newWidth = std::min(std::max(newWidth, 1060), availableWidth);
    newHeight = std::min(std::max(newHeight, 740), availableHeight);
    int minX = screenAvailableGeometry.x();
    int minY = screenAvailableGeometry.y();
    int maxX = screenAvailableGeometry.x() + availableWidth - newWidth;
    int maxY = screenAvailableGeometry.y() + availableHeight - newHeight;
    newX = std::clamp(newX, minX, maxX);
    newY = std::clamp(newY, minY, maxY);
    m_geometry[0] = QString::number(newX);
    m_geometry[1] = QString::number(newY);
    m_geometry[2] = QString::number(newWidth);
    m_geometry[3] = QString::number(newHeight);

    // Update geometry with last values
#ifdef __WINDOWS__
    resize(m_geometry[2].toInt(), m_geometry[3].toInt());
#else  // __WINDOWS__
    // On linux, when the window is opened for the first time by the window manager,
    // QT does not have information about the frame size so the offset is zero.
    // As such, the first time it opens the window does not include the offset,
    // so it is moved from the last position it had.
    // Excluding the offset from the saved value tries to fix that.
    int offsetX = geometry().left() - frameGeometry().left();
    int offsetY = geometry().top() - frameGeometry().top();
    newX += offsetX;
    newY += offsetY;
    setGeometry(newX,   // x position
            newY,       // y position
            newWidth,   // width
            newHeight); // height
#endif // __LINUX__ / __MACOS__
    // Move is also needed on linux.
    move(newX, newY);

    // Notify children that we are about to show.
    emit showDlg();
}

void DlgPreferences::slotButtonPressed(QAbstractButton* pButton) {
    QDialogButtonBox::ButtonRole role = buttonBox->buttonRole(pButton);
    DlgPreferencePage* pCurrentPage = currentPage();
    switch (role) {
    case QDialogButtonBox::ResetRole:
        // Only reset to defaults on the current page.
        if (pCurrentPage) {
            pCurrentPage->slotResetToDefaults();
        }
        break;
    case QDialogButtonBox::ApplyRole:
        emit applyPreferences();
        if (!pendingConfigValidOnAllPages()) {
            return;
        }
        break;
    case QDialogButtonBox::AcceptRole:
        emit applyPreferences();
        if (!pendingConfigValidOnAllPages()) {
            return;
        }
        accept();
        // Same as Apply but close the dialog
        break;
    case QDialogButtonBox::RejectRole:
        emit cancelPreferences();
        reject();
        break;
    case QDialogButtonBox::HelpRole:
        if (pCurrentPage) {
            QUrl helpUrl = pCurrentPage->helpUrl();
            DEBUG_ASSERT(helpUrl.isValid());
            mixxx::DesktopHelper::openUrl(helpUrl);
        }
        break;
    default:
        break;
    }
}

bool DlgPreferences::pendingConfigValidOnAllPages() {
    for (const PreferencesPage& page : std::as_const(m_allPages)) {
        if (page.pDlg && !page.pDlg->okayToClose()) {
            // If any page is not okay to close, eg. with an invalid sound config,
            // switch to it and don't accept.
            // Fixes https://github.com/mixxxdj/mixxx/issues/6077
            // and may help with other pages in the future.
            m_search->clear();
            contentsTreeWidget->setCurrentItem(page.pTreeItem);
            return false;
        }
    }
    return true;
}

void DlgPreferences::addPageWidget(PreferencesPage page,
        const QString& pageTitle,
        const QString& iconFile,
        const QString& category) {
    if (!category.isEmpty()) {
        auto* group = m_categories.value(category);
        if (!group) {
            group = new QTreeWidgetItem(contentsTreeWidget);
            group->setText(0, category);
            group->setIcon(0, QIcon(m_iconsPath.filePath(iconFile)));
            group->setData(0, Qt::UserRole, true);
            group->setExpanded(true);
            m_categories.insert(category, group);
        }
        const int index = contentsTreeWidget->indexOfTopLevelItem(page.pTreeItem);
        if (index >= 0) contentsTreeWidget->takeTopLevelItem(index);
        group->addChild(page.pTreeItem);
    }
    // Configure the tree button linked to the page
    page.pTreeItem->setIcon(0, QIcon(m_iconsPath.filePath(iconFile)));
    page.pTreeItem->setText(0, pageTitle);
    page.pTreeItem->setTextAlignment(0, Qt::AlignLeft | Qt::AlignVCenter);
    page.pTreeItem->setFlags(Qt::ItemIsSelectable | Qt::ItemIsEnabled);

    connect(this, &DlgPreferences::showDlg, page.pDlg, &DlgPreferencePage::slotShow);
    connect(this, &DlgPreferences::closeDlg, page.pDlg, &DlgPreferencePage::slotHide);
    connect(this, &DlgPreferences::showDlg, page.pDlg, &DlgPreferencePage::slotUpdate);

    connect(this, &DlgPreferences::applyPreferences, page.pDlg, &DlgPreferencePage::slotApply);
    connect(this, &DlgPreferences::cancelPreferences, page.pDlg, &DlgPreferencePage::slotCancel);
    connect(this,
            &DlgPreferences::resetToDefaults,
            page.pDlg,
            &DlgPreferencePage::slotResetToDefaults);

    // Add a new scroll area to the stacked pages widget containing the page
    QScrollArea* sa = new QScrollArea(pagesWidget);
    sa->setWidgetResizable(true);
    sa->setFrameShape(QFrame::NoFrame);

    sa->setWidget(page.pDlg);
    pagesWidget->addWidget(sa);
    m_allPages.append(page);

    int iframe = 2 * sa->frameWidth();
    m_pageSizeHint = m_pageSizeHint.expandedTo(
            page.pDlg->sizeHint() + QSize(iframe, iframe));
}

DlgPreferencePage* DlgPreferences::currentPage() {
    QObject* pObject = pagesWidget->currentWidget();
    for (int i = 0; i < 2; ++i) {
        if (pObject == nullptr) {
            return nullptr;
        }
        QObjectList children = pObject->children();
        if (children.isEmpty()) {
            return nullptr;
        }
        pObject = children[0];
    }
    return qobject_cast<DlgPreferencePage*>(pObject);
}

void DlgPreferences::removePageWidget(DlgPreferencePage* pWidget) {
    m_allPages.erase(std::remove_if(m_allPages.begin(), m_allPages.end(),
                            [pWidget](const PreferencesPage& page) { return page.pDlg == pWidget; }),
            m_allPages.end());
    m_tabItems.clear();
    pagesWidget->removeWidget(pWidget->parentWidget()->parentWidget());
}

void DlgPreferences::expandTreeItem(QTreeWidgetItem* pItem) {
    contentsTreeWidget->expandItem(pItem);
}

void DlgPreferences::switchToPage(const QString& pageTitle, DlgPreferencePage* pWidget) {
    pagesWidget->show();
    m_sectionTabs->show();
#ifdef __APPLE__
    // According to Apple's Human Interface Guidelines, settings dialogs have to
    // "Update the window’s title to reflect the currently visible pane."
    // This also solves the problem of the changed in terminology, Settings instead
    // of Preferences, since macOS Ventura.
    setWindowTitle(pageTitle);
#else
    Q_UNUSED(pageTitle);
#endif
    m_pageHeading->setText(pageTitle);
    pagesWidget->setCurrentWidget(pWidget->parentWidget()->parentWidget());
    // Reveal matching settings inside pages that have their own tabs.
    const QString term = m_search->text().trimmed();
    if (!term.isEmpty()) {
        for (auto* tabs : pWidget->findChildren<QTabWidget*>()) {
            for (int i = 0; i < tabs->count(); ++i) {
                QString text = tabs->tabText(i);
                for (auto* label : tabs->widget(i)->findChildren<QLabel*>()) text += " " + label->text();
                for (auto* button : tabs->widget(i)->findChildren<QAbstractButton*>()) text += " " + button->text();
                if (text.contains(term, Qt::CaseInsensitive)) {
                    tabs->setCurrentIndex(i);
                    break;
                }
            }
        }
    }

    QPushButton* pButton = buttonBox->button(QDialogButtonBox::Help);
    VERIFY_OR_DEBUG_ASSERT(pButton) {
        return;
    }

    if (pWidget->helpUrl().isValid()) {
        pButton->show();
    } else {
        pButton->hide();
    }
}

void DlgPreferences::moveEvent(QMoveEvent* e) {
    if (m_geometry.length() == 4) {
#ifdef __WINDOWS__
        Q_UNUSED(e);
        m_geometry[0] = QString::number(frameGeometry().left());
        m_geometry[1] = QString::number(frameGeometry().top());
#else
        // Warning! geometry does NOT include the frame/title.
        int offsetX = geometry().left() - frameGeometry().left();
        int offsetY = geometry().top() - frameGeometry().top();
        m_geometry[0] = QString::number(e->pos().x() - offsetX);
        m_geometry[1] = QString::number(e->pos().y() - offsetY);
#endif
    }
}

void DlgPreferences::resizeEvent(QResizeEvent* e) {
    if (m_geometry.length() == 4) {
        m_geometry[2] = QString::number(e->size().width());
        m_geometry[3] = QString::number(e->size().height());
    }
}

QRect DlgPreferences::getDefaultGeometry() {
    adjustSize();
    const auto* const pScreen = mixxx::widgethelper::getScreen(*this);
    VERIFY_OR_DEBUG_ASSERT(pScreen) {
        return QRect();
    }
    QSize optimumSize = pScreen->availableGeometry().size();

    if (frameSize() == size()) {
        // This code is reached in Gnome 2.3
        qDebug() << "guess the size of the window decoration";
        optimumSize -= QSize(2, 30);
    } else {
        optimumSize -= (frameSize() - size());
    }

    QSize staticSize = size() - pagesWidget->size();
    optimumSize = optimumSize.boundedTo((staticSize + m_pageSizeHint).expandedTo(QSize(1060, 740)));

    QRect optimumRect = geometry();
    optimumRect.setSize(optimumSize);

    return optimumRect;
}

void DlgPreferences::fixSliderStyle() {
#ifdef Q_OS_MACOS
    // Only used on macOS where the default slider style has several issues:
    // - the handle is semi-transparent
    // - the slider is higher than the space we give it, which causes that:
    //   - the groove is not correctly centered vertically
    //   - the handle is cut off at the top
    // The style below is based on sliders in the macOS system settings dialogs.
    if (darkAppearance()) {
        setStyleSheet(R"--(
QSlider::handle:horizontal {
    background-color: #8f8c8b; 
    border-radius: 4px;
    width: 8px;
    margin: -8px;
} 
QSlider::handle:horizontal::pressed {
    background-color: #a9a7a7;
}
QSlider::groove:horizontal {
    background: #1e1e1e; 
    height: 4px;
    border-radius: 2px;
    margin-left: 8px; 
    margin-right: 8px;
}
)--");
    } else {
        setStyleSheet(R"--(
QSlider::handle:horizontal {
    background-color: #ffffff;
    border-radius: 4px;
    border: 1px solid #d4d3d3;
    width: 7px;
    margin: -8px;
}
QSlider::handle:horizontal::pressed {
    background-color: #ececec;
}
QSlider::groove:horizontal {
    background: #c6c5c5;
    height: 4px;
    border-radius: 2px;
    margin-left: 8px;
    margin-right: 8px;
}
)--");
    }
#endif // __APPLE__
}
