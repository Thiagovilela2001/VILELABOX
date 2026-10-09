#include "analyzer/vocalanalysisstore.h"

#include <QJsonArray>
#include <algorithm>
#include <cmath>

#include "moc_vocalanalysisstore.cpp"

bool VocalAnalysis::hasVocal(double seconds) const {
    const auto it = std::upper_bound(segments.cbegin(), segments.cend(), seconds,
            [](double time, const auto& segment) { return time < segment.first; });
    return it != segments.cbegin() && seconds >= (it - 1)->first && seconds < (it - 1)->second;
}

std::shared_ptr<const VocalAnalysis> VocalAnalysis::fromJson(const QJsonObject& object) {
    if (!object.value("duration").isDouble() || !object.value("segments").isArray() ||
            object.value("version").toString().isEmpty()) {
        return {};
    }
    auto result = std::make_shared<VocalAnalysis>();
    result->duration = object.value("duration").toDouble();
    if (!std::isfinite(result->duration) || result->duration < 0) {
        return {};
    }
    double previousEnd = 0;
    for (const auto& value : object.value("segments").toArray()) {
        const auto segment = value.toArray();
        if (segment.size() != 2 || !segment[0].isDouble() || !segment[1].isDouble()) {
            return {};
        }
        const double start = segment[0].toDouble();
        const double end = segment[1].toDouble();
        if (!std::isfinite(start) || !std::isfinite(end) || start < previousEnd ||
                start < 0 || start >= result->duration || end <= start || end > result->duration + 0.001) {
            return {};
        }
        result->segments.append(qMakePair(start, std::min(end, result->duration)));
        previousEnd = end;
    }
    result->status = Status::Ready;
    return result;
}

VocalAnalysisStore& VocalAnalysisStore::instance() {
    static VocalAnalysisStore store;
    return store;
}

std::shared_ptr<const VocalAnalysis> VocalAnalysisStore::get(const QString& location) const {
    QReadLocker lock(&m_lock);
    return m_results.value(location);
}

void VocalAnalysisStore::set(const QString& location, std::shared_ptr<const VocalAnalysis> analysis) {
    {
        QWriteLocker lock(&m_lock);
        m_results.insert(location, std::move(analysis));
    }
    emit updated(location);
}
