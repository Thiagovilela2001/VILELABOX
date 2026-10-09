#include "widget/wkey.h"

#include <QMenu>
#include <QMouseEvent>

#include "library/library_prefs.h"
#include "moc_wkey.cpp"
#include "skin/legacy/skincontext.h"
#include "track/keyutils.h"

WKey::WKey(const QString& group, QWidget* pParent)
        : WLabel(pParent),
          m_group(group),
          m_dOldValue(0),
          m_keyNotation(mixxx::library::prefs::kKeyNotationConfigKey, this),
          m_engineKeyDistance(group,
                  "visual_key_distance",
                  this,
                  ControlFlag::AllowMissingOrInvalid) {
    setValue(m_dOldValue);
    m_keyNotation.connectValueChanged(this, &WKey::keyNotationChanged);
    m_engineKeyDistance.connectValueChanged(this, &WKey::setCents);
}

void WKey::onConnectedControlChanged(double dParameter, double dValue) {
    Q_UNUSED(dParameter);
    // Enums are not currently represented using parameter space so it doesn't
    // make sense to use the parameter here yet.
    setValue(dValue);
}

void WKey::setup(const QDomNode& node, const SkinContext& context) {
    WLabel::setup(node, context);
    m_editableKey = context.selectBool(node, "EditableKey", false);
    if (m_editableKey) {
        setCursor(Qt::PointingHandCursor);
        setToolTip(tr("Clique para travar ou escolher a tonalidade"));
    }
    m_displayCents = context.selectBool(node, "DisplayCents", false);
    m_displayKey = context.selectBool(node, "DisplayKey", true);
    setValue(m_dOldValue);
}

void WKey::setValue(double dValue) {
    m_dOldValue = dValue;
    mixxx::track::io::key::ChromaticKey key =
            KeyUtils::keyFromNumericValue(dValue);
    if (key != mixxx::track::io::key::INVALID) {
        // Render this key with the user-provided notation.
        QString keyStr = "";
        if (m_displayKey) {
            keyStr = KeyUtils::keyToString(key);
        }
        if (m_displayCents) {
            double diff_cents = m_engineKeyDistance.get();
            int cents_to_display = static_cast<int>(diff_cents * 100);
            char sign = ' ';
            if (diff_cents < 0) {
                sign = '-';
            } else if (diff_cents > 0) {
                sign = '+';
            }
            keyStr.append(QString(" %1%2c").arg(sign).arg(qAbs(cents_to_display)));
        }
        setText(m_editableKey ? keyStr + QStringLiteral(" ▾") : keyStr);
    } else {
        setText(m_editableKey ? tr("TOM ▾") : QString());
    }
}

void WKey::setCents() {
    setValue(m_dOldValue);
}

void WKey::keyNotationChanged(double dKeyNotationValue) {
    Q_UNUSED(dKeyNotationValue);
    // NOTE: dKeyNotationValue is the index of the key notation type, NOT the
    // key itself, so we intentionally set the old value again to update the UI.
    setValue(m_dOldValue);
}

void WKey::mousePressEvent(QMouseEvent* event) {
    if (!m_editableKey ||
            (event->button() != Qt::LeftButton && event->button() != Qt::RightButton)) {
        WLabel::mousePressEvent(event);
        return;
    }
    ControlProxy keylock(m_group, "keylock");
    ControlProxy loaded(m_group, "track_loaded");
    ControlProxy fileKey(m_group, "file_key");
    ControlProxy engineKey(m_group, "key");
    ControlProxy pitch(m_group, "pitch");
    QMenu menu(this);
    auto* lock = menu.addAction(tr("Travar tonalidade (MT)"));
    lock->setCheckable(true);
    lock->setChecked(keylock.toBool());
    connect(lock, &QAction::triggered, &menu, [&keylock](bool checked) {
        keylock.set(checked ? 1.0 : 0.0);
    });
    menu.addSeparator();
    for (const int step : {-1, 1}) {
        auto* action = menu.addAction(step < 0 ? tr("Diminuir 1 semitom") : tr("Aumentar 1 semitom"));
        action->setEnabled(loaded.toBool());
        connect(action, &QAction::triggered, &menu, [&, step] {
            const double target = pitch.get() + step;
            keylock.set(1.0);
            pitch.set(target);
        });
    }
    auto* reset = menu.addAction(tr("Restaurar tom original"));
    reset->setEnabled(loaded.toBool());
    connect(reset, &QAction::triggered, &menu, [&] {
        keylock.set(1.0);
        pitch.set(0.0);
    });
    menu.addSeparator();
    const auto original = KeyUtils::keyFromNumericValue(fileKey.get());
    if (original != mixxx::track::io::key::INVALID && loaded.toBool()) {
        const bool major = KeyUtils::keyIsMajor(original);
        menu.addSection(major ? tr("Escolher tom (maior)") : tr("Escolher tom (menor)"));
        for (int tonic = 0; tonic < 12; ++tonic) {
            const auto target = KeyUtils::tonicToKey(tonic, major);
            const QString traditional = KeyUtils::keyToString(target, KeyUtils::KeyNotation::Traditional);
            const QString configured = KeyUtils::keyToString(target);
            auto* action = menu.addAction(traditional == configured ? traditional :
                    QString("%1 (%2)").arg(traditional, configured));
            action->setCheckable(true);
            action->setChecked(KeyUtils::keyFromNumericValue(engineKey.get()) == target);
            connect(action, &QAction::triggered, &menu, [&, original, target] {
                keylock.set(1.0);
                pitch.set(KeyUtils::shortestStepsToKey(original, target));
            });
        }
    } else {
        auto* hint = menu.addAction(tr("Analise o tom da faixa para escolher uma nota"));
        hint->setEnabled(false);
    }
    menu.exec(event->globalPosition().toPoint());
    event->accept();
}
