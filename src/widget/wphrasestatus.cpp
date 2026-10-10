#include "widget/wphrasestatus.h"
#include "moc_wphrasestatus.cpp"
#include "widget/phrasecolors.h"
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
    const int type = qBound(0, static_cast<int>(m_type.get()), 6);
    if (!m_valid.toBool()) {
        // Without an analysis the skin hides the label; never show a placeholder.
        setText(QString());
        return;
    }
    setText(QStringLiteral("%1 · %2").arg(QString::fromLatin1(labels[type])).arg(static_cast<int>(m_bars.get())));
    // Same colour as the phrase band in the waveform and the overview.
    setStyleSheet(QStringLiteral("color: %1;").arg(
            phraseColor(static_cast<vilelabox::PhraseType>(type)).name()));
}
