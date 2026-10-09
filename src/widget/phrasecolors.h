#pragma once
#include <QColor>
#include <QStringList>
#include "track/phrases.h"
inline QColor phraseColor(vilelabox::PhraseType type, const QString& custom = {}) {
    static const char* defaults[]{"#808080", "#36bfa3", "#ffd447", "#ff4545", "#498dff", "#a778dd", "#39a6b8"};
    const int index = static_cast<int>(type);
    const auto colors = custom.split(',');
    const QColor color = index < colors.size() ? QColor(colors[index].trimmed()) : QColor();
    return color.isValid() ? color : QColor(defaults[index]);
}
