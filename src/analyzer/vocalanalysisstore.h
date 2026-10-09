#pragma once

#include <QObject>
#include <QReadWriteLock>
#include <QHash>
#include <QJsonObject>
#include <QVector>
#include <memory>

struct VocalAnalysis {
    enum class Status { Pending, Ready, Failed };
    Status status = Status::Pending;
    double duration = 0;
    QVector<QPair<double, double>> segments;
    bool hasVocal(double seconds) const;
    static std::shared_ptr<const VocalAnalysis> fromJson(const QJsonObject& object);
};

class VocalAnalysisStore : public QObject {
    Q_OBJECT
  public:
    static VocalAnalysisStore& instance();
    std::shared_ptr<const VocalAnalysis> get(const QString& location) const;
    void set(const QString& location, std::shared_ptr<const VocalAnalysis> analysis);
  signals:
    void updated(const QString& location);
  private:
    mutable QReadWriteLock m_lock;
    QHash<QString, std::shared_ptr<const VocalAnalysis>> m_results;
};
