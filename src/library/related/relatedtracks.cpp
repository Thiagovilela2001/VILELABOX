#include "library/related/relatedtracks.h"
#include <QCryptographicHash>
#include <QDateTime>
#include <QDir>
#include <QFileInfo>
#include <QSqlDatabase>
#include <QSqlQuery>
#include <QUuid>
#include <cmath>
namespace vilelabox {
RelatedScore relatedScore(double sourceBpm, double candidateBpm,
        mixxx::track::io::key::ChromaticKey sourceKey, mixxx::track::io::key::ChromaticKey candidateKey,
        double sourceEnergy, double candidateEnergy) {
    RelatedScore result;
    if (!std::isfinite(sourceBpm) || !std::isfinite(candidateBpm) || sourceBpm <= 0 || candidateBpm <= 0) return result;
    double difference = 1;
    for (double factor : {0.5, 1.0, 2.0}) {
        double delta = std::abs(candidateBpm * factor / sourceBpm - 1);
        if (delta < difference) { difference = delta; result.adjustedBpm = candidateBpm * factor; }
    }
    if (difference > 0.06 + 1e-10 || !KeyUtils::getCompatibleKeys(sourceKey).contains(candidateKey)) return {};
    const int sourceWheel = KeyUtils::keyToOpenKeyNumber(sourceKey);
    const int candidateWheel = KeyUtils::keyToOpenKeyNumber(candidateKey);
    const int wheelDistance = std::abs(sourceWheel - candidateWheel);
    if (KeyUtils::keyIsMajor(sourceKey) != KeyUtils::keyIsMajor(candidateKey) && wheelDistance != 0) return {};
    result.score = 60 + 25 * (1 - difference / 0.06);
    if (sourceEnergy > 0 && candidateEnergy > 0) result.score += 15 * std::max(0.0, 1 - std::abs(sourceEnergy - candidateEnergy) / 9);
    result.reason = QStringLiteral("%1→%2 · %3 BPM%4").arg(KeyUtils::keyToString(sourceKey, KeyUtils::KeyNotation::Lancelot),
            KeyUtils::keyToString(candidateKey, KeyUtils::KeyNotation::Lancelot),
            QString::number(result.adjustedBpm - sourceBpm, 'f', 1),
            sourceEnergy > 0 && candidateEnergy > 0 ? QStringLiteral(" · ΔE %1").arg(std::abs(sourceEnergy - candidateEnergy), 0, 'f', 1) : QStringLiteral(" · energia pendente"));
    return result;
}
QString energyIdentity(const QString& location) {
    const QFileInfo f(location);
    const QByteArray identity = location.toUtf8() + '\0' + QByteArray::number(f.size()) + '\0' +
            QByteArray::number(f.lastModified().toMSecsSinceEpoch()) + "energy-v1";
    return QString::fromLatin1(QCryptographicHash::hash(identity, QCryptographicHash::Sha256).toHex());
}
namespace {
template<class Function> void withEnergyDb(const QString& settingsPath, Function function) {
    QDir directory(settingsPath + "/analysis"); if (!directory.mkpath(".")) return;
    const QString name = "vilelabox-energy-" + QUuid::createUuid().toString();
    {
        auto db = QSqlDatabase::addDatabase("QSQLITE", name);
        db.setDatabaseName(directory.filePath("blackmesa.sqlite"));
        db.setConnectOptions("QSQLITE_BUSY_TIMEOUT=5000");
        if (db.open()) {
            QSqlQuery schema(db);
            if (schema.exec("CREATE TABLE IF NOT EXISTS energy (identity TEXT PRIMARY KEY, value REAL NOT NULL)")) function(db);
        }
        db.close();
    }
    QSqlDatabase::removeDatabase(name);
}
}
void saveEnergy(const QString& settingsPath, const QString& location, double energy) {
    withEnergyDb(settingsPath, [&](const QSqlDatabase& db) {
        QSqlQuery query(db); query.prepare("INSERT OR REPLACE INTO energy VALUES (?, ?)");
        query.addBindValue(energyIdentity(location)); query.addBindValue(energy); query.exec();
    });
}
QHash<QString, double> loadEnergies(const QString& settingsPath) {
    QHash<QString, double> result;
    withEnergyDb(settingsPath, [&](const QSqlDatabase& db) {
        QSqlQuery query("SELECT identity, value FROM energy", db);
        while (query.next()) result.insert(query.value(0).toString(), query.value(1).toDouble());
    });
    return result;
}
}
