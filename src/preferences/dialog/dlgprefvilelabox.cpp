#include "preferences/dialog/dlgprefvilelabox.h"

#include <QCheckBox>
#include <QFormLayout>
#include <QGroupBox>
#include <QLabel>
#include <QSpinBox>
#include <QTabWidget>
#include <QVBoxLayout>

#include "control/controlobject.h"

DlgPrefVilelabox::DlgPrefVilelabox(QWidget* parent, UserSettingsPointer config)
        : DlgPreferencePage(parent), m_config(std::move(config)) {
    auto* layout = new QVBoxLayout(this);
    auto* description = new QLabel(tr("Personalize os painéis das skins e as opções próprias do VILELABOX. "
                                     "As alterações entram em vigor ao clicar em Aplicar."), this);
    description->setWordWrap(true);
    layout->addWidget(description);
    auto* tabs = new QTabWidget(this);
    layout->addWidget(tabs);

    struct Setting { const char* key; QString label; bool defaultValue; };
    const QList<Setting> settings{
            {"show_waveform_band", tr("Waveforms dos decks"), true},
            {"show_mixer", tr("Mixer"), true},
            {"show_library", tr("Biblioteca"), true},
            {"show_big_library", tr("Biblioteca expandida"), false},
            {"show_hotcues", tr("Pads e hot cues"), true},
            {"show_8_hotcues", tr("Mostrar os oito hot cues"), true},
            {"show_loop_controls", tr("Controles de loop"), true},
            {"show_beatjump_controls", tr("Saltos de batida"), true},
            {"show_beatgrid_controls", tr("Editor de grid"), true},
            {"show_intro_outro", tr("Cues de intro e outro"), false},
            {"show_pitch", tr("Fader de tempo"), true},
            {"show_eq", tr("Equalizador"), true},
            {"show_xfader", tr("Crossfader"), true}};
    for (const auto& skin : {QStringLiteral("RbxOriginal"), QStringLiteral("RbxFlow")}) {
        auto* panel = new QWidget(tabs);
        auto* panelLayout = new QVBoxLayout(panel);
        for (const auto& setting : settings) {
            auto* checkbox = new QCheckBox(setting.label, panel);
            const ConfigKey key("[" + skin + "]", QString::fromLatin1(setting.key));
            checkbox->setObjectName(skin + "_" + QString::fromLatin1(setting.key));
            panelLayout->addWidget(checkbox);
            m_toggles.append({key, checkbox, setting.defaultValue});
        }
        panelLayout->addStretch();
        tabs->addTab(panel, skin == "RbxFlow" ? "RbxFlowBars" : skin);
    }
    auto* options = new QWidget(tabs);
    auto* optionsLayout = new QVBoxLayout(options);
    auto* display = new QGroupBox(tr("Waveform"), options);
    auto* displayLayout = new QVBoxLayout(display);
    auto* rgb = new QCheckBox(tr("Modo RGB: graves vermelhos, médios verdes e agudos azuis"), display);
    rgb->setObjectName("waveformRgb");
    displayLayout->addWidget(rgb);
    m_toggles.append({ConfigKey("[Vilelabox]", "waveform_rgb"), rgb, false});
    optionsLayout->addWidget(display);
    auto* analysis = new QGroupBox(tr("Biblioteca e análise"), options);
    auto* form = new QFormLayout(analysis);
    m_relatedLimit = new QSpinBox(analysis);
    m_relatedLimit->setObjectName("relatedLimit");
    m_relatedLimit->setRange(1, 1000);
    form->addRow(tr("Máximo de faixas relacionadas:"), m_relatedLimit);
    m_vocalTimeout = new QSpinBox(analysis);
    m_vocalTimeout->setObjectName("vocalTimeout");
    m_vocalTimeout->setRange(1, 3600);
    m_vocalTimeout->setSuffix(tr(" s"));
    form->addRow(tr("Tempo limite do detector de vocal:"), m_vocalTimeout);
    optionsLayout->addWidget(analysis);
    optionsLayout->addStretch();
    tabs->addTab(options, tr("Opções do mod"));
    layout->addStretch();
    setScrollSafeGuardForAllInputWidgets(this);
    slotUpdate();
}

void DlgPrefVilelabox::slotUpdate() {
    for (const auto& toggle : std::as_const(m_toggles)) {
        const bool value = ControlObject::exists(toggle.key)
                ? ControlObject::get(toggle.key) != 0
                : m_config->getValue(toggle.key, toggle.defaultValue);
        toggle.widget->setChecked(value);
    }
    m_relatedLimit->setValue(m_config->getValue(ConfigKey("[Vilelabox]", "RelatedLimit"), 100));
    m_vocalTimeout->setValue(m_config->getValue(ConfigKey("[Vilelabox]", "VocalTimeoutSeconds"), 120));
}

void DlgPrefVilelabox::slotApply() {
    for (const auto& toggle : std::as_const(m_toggles)) {
        const bool value = toggle.widget->isChecked();
        m_config->setValue(toggle.key, value);
        if (ControlObject::exists(toggle.key)) ControlObject::set(toggle.key, value ? 1.0 : 0.0);
    }
    m_config->setValue(ConfigKey("[Vilelabox]", "RelatedLimit"), m_relatedLimit->value());
    m_config->setValue(ConfigKey("[Vilelabox]", "VocalTimeoutSeconds"), m_vocalTimeout->value());
}

void DlgPrefVilelabox::slotResetToDefaults() {
    for (const auto& toggle : std::as_const(m_toggles)) toggle.widget->setChecked(toggle.defaultValue);
    m_relatedLimit->setValue(100);
    m_vocalTimeout->setValue(120);
}
