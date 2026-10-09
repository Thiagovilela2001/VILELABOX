#pragma once

#include "widget/wlabel.h"
#include "control/controlproxy.h"

class WKey : public WLabel  {
    Q_OBJECT
  public:
    explicit WKey(const QString& group, QWidget* pParent = nullptr);

    void onConnectedControlChanged(double dParameter, double dValue) override;
    void setup(const QDomNode& node, const SkinContext& context) override;

  protected:
    void mousePressEvent(QMouseEvent* event) override;

  private slots:
    void setValue(double dValue);
    void keyNotationChanged(double dValue);
    void setCents();

  private:
    const QString m_group;
    bool m_editableKey = false;
    double m_dOldValue;
    bool m_displayCents;
    bool m_displayKey;
    ControlProxy m_keyNotation;
    ControlProxy m_engineKeyDistance;
};
