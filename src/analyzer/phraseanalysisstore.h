#pragma once
#include <QObject>
#include <QHash>
#include <QReadWriteLock>
#include <memory>
#include "track/phrases.h"
#include "track/track_decl.h"

class PhraseAnalysisStore : public QObject {
    Q_OBJECT
  public:
    static PhraseAnalysisStore& instance();
    static QString trackKey(const TrackPointer& track);
    std::shared_ptr<const vilelabox::PhraseAnalysis> get(const QString& location) const;
    void set(TrackPointer track, vilelabox::PhraseAnalysis analysis, const QString& cache);
    static QString cachePath(const QString& directory, const QString& location);
    static bool load(const QString& path, vilelabox::PhraseAnalysis* analysis);
    static bool save(const QString& path, const vilelabox::PhraseAnalysis& analysis,
            const mixxx::BeatsPointer& beats);
  signals:
    void updated(const QString& location);
  private:
    PhraseAnalysisStore();
    void recalculate(TrackPointer track);
    mutable QReadWriteLock m_lock;
    QHash<QString, std::shared_ptr<const vilelabox::PhraseAnalysis>> m_results;
    QHash<QString, QString> m_caches;
};
