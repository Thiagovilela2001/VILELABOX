#pragma once
#include <QString>
#include <QHash>
#include "track/keyutils.h"
namespace vilelabox {
struct RelatedScore { double score = 0; double adjustedBpm = 0; QString reason; };
RelatedScore relatedScore(double sourceBpm, double candidateBpm,
        mixxx::track::io::key::ChromaticKey sourceKey, mixxx::track::io::key::ChromaticKey candidateKey,
        double sourceEnergy = 0, double candidateEnergy = 0);
QString energyIdentity(const QString& location);
void saveEnergy(const QString& settingsPath, const QString& location, double energy);
QHash<QString, double> loadEnergies(const QString& settingsPath);
}
