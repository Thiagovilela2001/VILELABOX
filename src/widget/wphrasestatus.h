#pragma once
#include "widget/wlabel.h"
#include "control/controlproxy.h"
class WPhraseStatus : public WLabel {
    Q_OBJECT
  public:
    WPhraseStatus(const QString& group, QWidget* parent);
    void setup(const QDomNode& node, const SkinContext& context) override;
  private:
    void refresh();
    ControlProxy m_type, m_bars, m_valid;
};
