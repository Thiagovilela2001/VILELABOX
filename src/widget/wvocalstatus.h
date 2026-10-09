#pragma once
#include "widget/wlabel.h"
#include "control/controlproxy.h"
#include "track/track_decl.h"
class PlayerManager;
class WVocalStatus : public WLabel {
    Q_OBJECT
  public:
    WVocalStatus(const QString& group, PlayerManager* manager, QWidget* parent);
    void setTrack(TrackPointer track);
    void setup(const QDomNode& node, const SkinContext& context) override;
  private:
    void refresh();
    TrackPointer m_track;
    ControlProxy m_position;
};
