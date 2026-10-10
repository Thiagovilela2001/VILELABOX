#pragma once
#include <QObject>
#include <QHash>
#include <QReadWriteLock>
#include <QTimer>
#include <memory>
#include "track/phrases.h"
#include "track/track_decl.h"

class PhraseAnalysisStore : public QObject {
    Q_OBJECT
  public:
    static PhraseAnalysisStore& instance();
    static QString trackKey(const TrackPointer& track);
    std::shared_ptr<const vilelabox::PhraseAnalysis> get(const QString& location) const;
    /// `persist` writes the cache immediately on the calling thread (the analyzer
    /// thread); false for results that were just read from the cache.
    void set(TrackPointer track, vilelabox::PhraseAnalysis analysis, const QString& cache,
            bool persist = false);
    static QString cachePath(const QString& directory, const QString& location);
    static bool load(const QString& path, vilelabox::PhraseAnalysis* analysis);
    static bool save(const QString& path, const vilelabox::PhraseAnalysis& analysis,
            const mixxx::BeatsPointer& beats);
    /// Number of tracks whose cache is waiting for the debounced write.
    int pendingSaveCount() const;
    /// Writes every pending cache now, on the calling thread.
    void flushPendingSaves();
  signals:
    void updated(const QString& location);
  private:
    PhraseAnalysisStore();
    ~PhraseAnalysisStore() override;
    enum class Save { None, Now, Debounced };
    void recalculate(TrackPointer track, Save save);
    void scheduleSave(const QString& location, const mixxx::BeatsPointer& beats);
    void writePendingSaves(bool async);
    struct PendingSave {
        QString cache;
        std::shared_ptr<const vilelabox::PhraseAnalysis> analysis;
        mixxx::BeatsPointer beats;
    };
    mutable QReadWriteLock m_lock;
    QHash<QString, std::shared_ptr<const vilelabox::PhraseAnalysis>> m_results;
    QHash<QString, QString> m_caches;
    // Grid edits recalculate often; the cache is written once they settle.
    QHash<QString, PendingSave> m_pendingSaves;
    QTimer* m_saveTimer = nullptr;
};
