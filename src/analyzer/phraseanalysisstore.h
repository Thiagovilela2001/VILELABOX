#pragma once
#include <QObject>
#include <QHash>
#include <QReadWriteLock>
#include <QMutex>
#include <QStringList>
#include <QThreadPool>
#include <QTimer>
#include <atomic>
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
    /// Writes both cache files (features and derived results).
    static bool save(const QString& path, const vilelabox::PhraseAnalysis& analysis,
            const mixxx::BeatsPointer& beats);
    /// Immutable per-track features; written once per analysis.
    static bool saveFeatures(const QString& path, const vilelabox::PhraseAnalysis& analysis);
    /// Grid-dependent results (phrases, grid); rewritten after grid edits.
    static bool saveDerived(const QString& path, const vilelabox::PhraseAnalysis& analysis,
            const mixxx::BeatsPointer& beats, qint64 sequence = 0);
    static QString derivedPath(const QString& featuresPath);
    /// Writes the derived file unless a write with a higher sequence for the
    /// same track already reached the disk. Used by the writer thread.
    void writeDerived(const QString& location, const QString& cache,
            const std::shared_ptr<const vilelabox::PhraseAnalysis>& analysis,
            const mixxx::BeatsPointer& beats, qint64 sequence);
    /// Waits until queued background writes are on disk.
    void waitForWrites();
    /// Tracks kept in memory beyond those in use (loaded in a deck).
    static constexpr int kMaxCachedTracks = 32;
    int cachedTrackCount() const;
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
    // Drops the oldest results nobody else holds. Requires m_lock (write).
    void evictLocked();
    struct PendingSave {
        QString cache;
        std::shared_ptr<const vilelabox::PhraseAnalysis> analysis;
        mixxx::BeatsPointer beats;
        qint64 sequence = 0;
    };
    mutable QReadWriteLock m_lock;
    QHash<QString, std::shared_ptr<const vilelabox::PhraseAnalysis>> m_results;
    QHash<QString, QString> m_caches;
    // Grid edits recalculate often; the cache is written once they settle.
    QHash<QString, PendingSave> m_pendingSaves;
    QTimer* m_saveTimer = nullptr;
    // Least recently set first.
    QStringList m_recent;
    // One writer thread: queued writes reach the disk in order. The sequence
    // numbers also stop an older write from replacing a newer one.
    QThreadPool m_writer;
    std::atomic<qint64> m_sequence{0};
    QMutex m_writtenLock;
    QHash<QString, qint64> m_written;
};
