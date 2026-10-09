#include "analyzer/phraseanalysisstore.h"
#include <QCryptographicHash>
#include <QCoreApplication>
#include <QDateTime>
#include <QDir>
#include <QFileInfo>
#include <QJsonArray>
#include <QJsonDocument>
#include <QSaveFile>
#include <cmath>
#include "analyzer/vocalanalysisstore.h"
#include "track/track.h"
#include "moc_phraseanalysisstore.cpp"

namespace { constexpr int kVersion = 1; }
PhraseAnalysisStore::PhraseAnalysisStore() {
    if (QCoreApplication::instance()) moveToThread(QCoreApplication::instance()->thread());
}
PhraseAnalysisStore& PhraseAnalysisStore::instance() {
    static PhraseAnalysisStore store;
    return store;
}
QString PhraseAnalysisStore::trackKey(const TrackPointer& track) {
    return track->getLocation().isEmpty() ? QStringLiteral("temporary:%1").arg(reinterpret_cast<quintptr>(track.get()), 0, 16) : track->getLocation();
}
std::shared_ptr<const vilelabox::PhraseAnalysis> PhraseAnalysisStore::get(const QString& location) const {
    QReadLocker lock(&m_lock);
    return m_results.value(location);
}
QString PhraseAnalysisStore::cachePath(const QString& directory, const QString& location) {
    const QFileInfo file(location);
    const auto identity = QJsonDocument(QJsonArray{location, QString::number(file.size()),
            QString::number(file.lastModified().toMSecsSinceEpoch()), kVersion}).toJson(QJsonDocument::Compact);
    return QDir(directory).filePath(QString::fromLatin1(
            QCryptographicHash::hash(identity, QCryptographicHash::Sha256).toHex()) + ".json");
}
bool PhraseAnalysisStore::load(const QString& path, vilelabox::PhraseAnalysis* analysis) {
    QFile file(path);
    if (!file.open(QIODevice::ReadOnly) || file.size() > 64 * 1024 * 1024) return false;
    const auto object = QJsonDocument::fromJson(file.readAll()).object();
    if (object.value("version").toInt(-1) != kVersion || !object.value("features").isArray()) return false;
    vilelabox::PhraseAnalysis result;
    result.duration = object.value("duration").toDouble(-1);
    if (!std::isfinite(result.duration) || result.duration <= 0) return false;
    double previousEnd = 0;
    for (const auto value : object.value("features").toArray()) {
        const auto row = value.toArray();
        if (row.size() != 23) return false;
        for (const auto v : row) if (!v.isDouble() || !std::isfinite(v.toDouble()) || v.toDouble() < 0) return false;
        vilelabox::PhraseFeature f;
        f.start = row[0].toDouble(); f.end = row[1].toDouble();
        if (f.start < previousEnd - 1e-8 || f.end <= f.start || f.end > result.duration + 1e-8) return false;
        f.rms = row[2].toDouble(); f.bass = row[3].toDouble(); f.flux = row[4].toDouble();
        for (int k = 0; k < 12; ++k) f.chroma[k] = row[5 + k].toDouble();
        for (int k = 0; k < 6; ++k) f.timbre[k] = row[17 + k].toDouble();
        previousEnd = f.end;
        result.features.push_back(f);
    }
    if (result.features.isEmpty()) return false;
    *analysis = std::move(result);
    return true;
}
bool PhraseAnalysisStore::save(const QString& path, const vilelabox::PhraseAnalysis& analysis,
        const mixxx::BeatsPointer& beats) {
    QJsonArray features, phrases;
    for (const auto& f : analysis.features) {
        QJsonArray row{f.start, f.end, f.rms, f.bass, f.flux};
        for (auto v : f.chroma) row.append(v);
        for (auto v : f.timbre) row.append(v);
        features.append(row);
    }
    for (const auto& p : analysis.phrases) phrases.append(QJsonArray{p.start, p.end, p.bars, static_cast<int>(p.type), p.confidence});
    QJsonObject grid;
    if (beats) {
        grid["serialized"] = QString::fromLatin1(beats->toByteArray().toBase64());
    }
    const auto bytes = QJsonDocument(QJsonObject{{"version", kVersion}, {"duration", analysis.duration},
            {"features", features}, {"phrases", phrases}, {"grid", grid}}).toJson(QJsonDocument::Compact);
    QDir().mkpath(QFileInfo(path).absolutePath());
    QSaveFile file(path);
    return file.open(QIODevice::WriteOnly) && file.write(bytes) == bytes.size() && file.commit();
}
void PhraseAnalysisStore::set(TrackPointer track, vilelabox::PhraseAnalysis analysis, const QString& cache) {
    const auto location = trackKey(track);
    {
        QWriteLocker lock(&m_lock);
        m_results.insert(location, std::make_shared<const vilelabox::PhraseAnalysis>(std::move(analysis)));
        m_caches.insert(location, cache);
    }
    if (track->getLocation().isEmpty()) {
        connect(track.get(), &QObject::destroyed, this, [this, location] {
            QWriteLocker lock(&m_lock); m_results.remove(location); m_caches.remove(location);
        }, Qt::DirectConnection);
    }
    // Context is the track: automatic disconnect on destruction. A weak pointer
    // prevents retaining every analyzed track for the lifetime of the process.
    const std::weak_ptr<Track> weak(track);
    QObject::disconnect(track.get(), &Track::beatsUpdated, this, nullptr);
    QObject::connect(track.get(), &Track::beatsUpdated, this, [this, weak] {
        if (auto current = weak.lock()) recalculate(current);
    }, Qt::QueuedConnection);
    recalculate(track);
}
void PhraseAnalysisStore::recalculate(TrackPointer track) {
    const auto location = trackKey(track);
    const auto old = get(location);
    if (!old) return;
    QVector<QPair<double, double>> vocals;
    const auto voice = VocalAnalysisStore::instance().get(location);
    if (voice && voice->status == VocalAnalysis::Status::Ready) vocals = voice->segments;
    auto result = std::make_shared<const vilelabox::PhraseAnalysis>(vilelabox::segmentPhrases(
            old->features, old->duration, track->getBeats(), vocals));
    QString cache;
    {
        QWriteLocker lock(&m_lock);
        m_results.insert(location, result);
        cache = m_caches.value(location);
    }
    if (!cache.isEmpty()) save(cache, *result, track->getBeats());
    emit updated(location);
}
