#include "library/related/relatedtracks.h"
#include "moc_relatedtracks.cpp"
#include <QCryptographicHash>
#include <QDateTime>
#include <QDir>
#include <QElapsedTimer>
#include <QFutureWatcher>
#include <QtConcurrent>
#include <QFileInfo>
#include <QSqlDatabase>
#include <QSqlQuery>
#include <QUuid>
#include <algorithm>
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
std::optional<double> loadEnergy(const QString& settingsPath, const QString& location) {
    std::optional<double> result;
    withEnergyDb(settingsPath, [&](const QSqlDatabase& db) {
        QSqlQuery query(db);
        query.prepare("SELECT value FROM energy WHERE identity = ?");
        query.addBindValue(energyIdentity(location));
        if (query.exec() && query.next()) result = query.value(0).toDouble();
    });
    return result;
}
QHash<QString, double> loadEnergies(const QString& settingsPath) {
    QHash<QString, double> result;
    withEnergyDb(settingsPath, [&](const QSqlDatabase& db) {
        QSqlQuery query("SELECT identity, value FROM energy", db);
        while (query.next()) result.insert(query.value(0).toString(), query.value(1).toDouble());
    });
    return result;
}
QVector<RelatedRow> findRelated(const QString& databasePath, const QString& settingsPath,
        const RelatedReference& reference, int limit) {
    QVector<RelatedRow> rows;
    if (reference.bpm <= 0 || limit <= 0) return rows;
    const auto energies = loadEnergies(settingsPath);
    const double referenceEnergy = energies.value(energyIdentity(reference.location));
    const QString name = "vilelabox-related-" + QUuid::createUuid().toString();
    {
        auto db = QSqlDatabase::addDatabase("QSQLITE", name);
        db.setDatabaseName(databasePath);
        db.setConnectOptions("QSQLITE_OPEN_READONLY;QSQLITE_BUSY_TIMEOUT=5000");
        if (db.open()) {
            QSqlQuery query(db);
            query.setForwardOnly(true);
            if (query.exec("SELECT l.id,l.artist,l.title,l.bpm,l.key_id,t.location FROM library l "
                           "JOIN track_locations t ON l.location=t.id "
                           "WHERE t.fs_deleted=0 AND l.mixxx_deleted=0")) {
                while (query.next()) {
                    const auto location = query.value(5).toString();
                    if (location == reference.location) continue;
                    const double bpm = query.value(3).toDouble();
                    const auto key = KeyUtils::keyFromNumericValue(query.value(4).toInt());
                    // Key and tempo filter first: the energy identity needs a file
                    // stat and a hash, so it is only computed for candidates.
                    if (relatedScore(reference.bpm, bpm, reference.key, key).score <= 0) continue;
                    const double energy = energies.value(energyIdentity(location));
                    const auto score = relatedScore(reference.bpm, bpm, reference.key, key, referenceEnergy, energy);
                    if (score.score <= 0) continue;
                    rows.push_back({query.value(0).toInt(), query.value(1).toString(), query.value(2).toString(),
                            KeyUtils::keyToString(key, KeyUtils::KeyNotation::Lancelot), bpm, energy, score});
                }
            }
        }
        db.close();
    }
    QSqlDatabase::removeDatabase(name);
    const auto better = [](const RelatedRow& a, const RelatedRow& b) { return a.score.score > b.score.score; };
    if (rows.size() > limit) {
        std::partial_sort(rows.begin(), rows.begin() + limit, rows.end(), better);
        rows.resize(limit);
    } else {
        std::sort(rows.begin(), rows.end(), better);
    }
    return rows;
}
RelatedSearch::RelatedSearch(QString databasePath, QString settingsPath, QObject* parent)
        : QObject(parent), m_databasePath(std::move(databasePath)), m_settingsPath(std::move(settingsPath)) {}
void RelatedSearch::start(const RelatedReference& reference, int limit) {
    const int generation = ++m_generation;
    struct Result { QVector<RelatedRow> rows; qint64 elapsed = 0; };
    // The watcher is a child of this object: if the search object goes away,
    // the result is simply never delivered.
    auto* watcher = new QFutureWatcher<Result>(this);
    connect(watcher, &QFutureWatcher<Result>::finished, this, [this, watcher, generation] {
        const auto result = watcher->result();
        watcher->deleteLater();
        if (generation == m_generation) emit finished(result.rows, result.elapsed);
    });
    watcher->setFuture(QtConcurrent::run([databasePath = m_databasePath, settingsPath = m_settingsPath,
                                                 reference, limit] {
        QElapsedTimer timer;
        timer.start();
        Result result;
        result.rows = findRelated(databasePath, settingsPath, reference, limit);
        result.elapsed = timer.elapsed();
        return result;
    }));
}
}
