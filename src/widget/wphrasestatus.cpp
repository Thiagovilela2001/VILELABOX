#include "widget/wphrasestatus.h"
#include "moc_wphrasestatus.cpp"
WPhraseStatus::WPhraseStatus(const QString& group, QWidget* parent)
        : WLabel(parent), m_type(group, "phrase_type", this),
          m_bars(group, "phrase_bars_left", this), m_valid(group, "phrase_valid", this) {
    m_type.connectValueChanged(this, &WPhraseStatus::refresh);
    m_bars.connectValueChanged(this, &WPhraseStatus::refresh);
    m_valid.connectValueChanged(this, &WPhraseStatus::refresh);
    refresh();
}
void WPhraseStatus::setup(const QDomNode& node, const SkinContext& context) {
    WLabel::setup(node, context);
    refresh();
}
void WPhraseStatus::refresh() {
    static const char* labels[]{"FRASE", "INTRO", "UP", "CHORUS", "DOWN", "VERSE", "OUTRO"};
    const int type = static_cast<int>(m_type.get());
    setText(m_valid.toBool() ? QStringLiteral("%1 · %2").arg(QString::fromLatin1(labels[qBound(0, type, 6)])).arg(static_cast<int>(m_bars.get())) : QStringLiteral("—"));
}
