#pragma once
#include <QHash>
#include <QObject>
#include <QString>
#include <QVector>
#include <optional>
#include "track/keyutils.h"
namespace vilelabox {
struct RelatedScore { double score = 0; double adjustedBpm = 0; QString reason; };
RelatedScore relatedScore(double sourceBpm, double candidateBpm,
        mixxx::track::io::key::ChromaticKey sourceKey, mixxx::track::io::key::ChromaticKey candidateKey,
        double sourceEnergy = 0, double candidateEnergy = 0);
QString energyIdentity(const QString& location);
void saveEnergy(const QString& settingsPath, const QString& location, double energy);
QHash<QString, double> loadEnergies(const QString& settingsPath);
/// Energy of one track, looked up by identity (no full table load).
std::optional<double> loadEnergy(const QString& settingsPath, const QString& location);

/// What the search compares against; plain values, safe to copy to a worker.
struct RelatedReference {
    QString location;
    double bpm = 0;
    mixxx::track::io::key::ChromaticKey key = mixxx::track::io::key::INVALID;
};
struct RelatedRow {
    int id = 0;
    QString artist, title, key;
    double bpm = 0, energy = 0;
    RelatedScore score;
};
/// Scores the library against `reference` and returns at most `limit` rows,
/// best first. Opens its own read-only connection, so it can run on any thread.
QVector<RelatedRow> findRelated(const QString& databasePath, const QString& settingsPath,
        const RelatedReference& reference, int limit);

/// Runs findRelated() on the thread pool. Only the most recent request is
/// reported; results of older requests are dropped.
class RelatedSearch : public QObject {
    Q_OBJECT
  public:
    RelatedSearch(QString databasePath, QString settingsPath, QObject* parent = nullptr);
    void start(const RelatedReference& reference, int limit);
    int generation() const { return m_generation; }
  signals:
    void finished(const QVector<vilelabox::RelatedRow>& rows, qint64 elapsedMs);
  private:
    QString m_databasePath, m_settingsPath;
    int m_generation = 0;
};
}
