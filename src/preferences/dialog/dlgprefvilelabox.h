#pragma once

#include <QVector>

#include "preferences/dialog/dlgpreferencepage.h"
#include "preferences/usersettings.h"

class QCheckBox;
class QSpinBox;

class DlgPrefVilelabox : public DlgPreferencePage {
  public:
    DlgPrefVilelabox(QWidget* parent, UserSettingsPointer config);
    void slotUpdate() override;
    void slotApply() override;
    void slotResetToDefaults() override;

  private:
    struct Toggle {
        ConfigKey key;
        QCheckBox* widget;
        bool defaultValue;
    };
    UserSettingsPointer m_config;
    QVector<Toggle> m_toggles;
    QSpinBox* m_relatedLimit;
    QSpinBox* m_vocalTimeout;
};
