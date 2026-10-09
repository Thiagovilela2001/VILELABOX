#include "analyzer/analyzervocals.h"

#include <QCryptographicHash>
#include <QDir>
#include <QDateTime>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QProcess>
#include <QProcessEnvironment>
#include <QSaveFile>

#include "analyzer/vocalanalysisstore.h"
#include "track/track.h"

AnalyzerVocals::AnalyzerVocals(UserSettingsPointer config) : m_config(std::move(config)) {
}

bool AnalyzerVocals::initialize(const AnalyzerTrack& track, mixxx::audio::SampleRate rate, SINT length) {
    cleanup();
    if (length <= 0 || !rate.isValid()) {
        return false;
    }
    m_location = track.getTrack()->getLocation();
    m_rate = static_cast<int>(rate);
    m_script = QDir(m_config->getResourcePath()).filePath("vocals/vocal_detector.py");
    m_python = qEnvironmentVariable("VILELABOX_VOCAL_PYTHON", QDir::homePath() + "/.venv/bin/python");
    QFile model(QFileInfo(m_script).dir().filePath("model.json"));
    QFile script(m_script);
    if (!QFileInfo::exists(m_python) || !model.open(QIODevice::ReadOnly) || !script.open(QIODevice::ReadOnly)) {
        fail();
        return false;
    }
    const QFileInfo source(m_location);
    QByteArray identity = m_location.toUtf8() + QByteArray::number(source.size()) +
            QByteArray::number(source.lastModified().toMSecsSinceEpoch()) + model.readAll() + script.readAll();
    const auto hash = QCryptographicHash::hash(identity, QCryptographicHash::Sha256).toHex();
    QDir cache(m_config->getSettingsPath() + "/analysis/vocals");
    cache.mkpath(".");
    m_cache = cache.filePath(QString::fromLatin1(hash) + ".json");
    QFile previous(m_cache);
    if (previous.open(QIODevice::ReadOnly)) {
        const auto result = VocalAnalysis::fromJson(QJsonDocument::fromJson(previous.readAll()).object());
        if (result) {
            VocalAnalysisStore::instance().set(m_location, result);
            return false;
        }
    }
    VocalAnalysisStore::instance().set(m_location, std::make_shared<VocalAnalysis>());
    m_audio = std::make_unique<QTemporaryFile>(QDir::tempPath() + "/vilelabox-vocal-XXXXXX.f32");
    if (!m_audio->open()) {
        fail();
        return false;
    }
    return true;
}

bool AnalyzerVocals::processSamples(const CSAMPLE* samples, SINT count) {
    if (!m_audio || count % 2 != 0) {
        fail();
        return false;
    }
    m_mono.resize(count / 2);
    for (SINT frame = 0; frame < count / 2; ++frame) {
        m_mono[frame] = (samples[frame * 2] + samples[frame * 2 + 1]) * 0.5f;
    }
    const qint64 size = static_cast<qint64>(m_mono.size() * sizeof(float));
    if (m_audio->write(reinterpret_cast<const char*>(m_mono.data()), size) != size) {
        fail();
        return false;
    }
    return true;
}

void AnalyzerVocals::storeResults(TrackPointer track) {
    Q_UNUSED(track);
    if (!m_audio || !m_audio->flush()) {
        fail();
        return;
    }
    QProcess process;
    auto environment = QProcessEnvironment::systemEnvironment();
    environment.insert("OPENBLAS_NUM_THREADS", "1");
    environment.insert("OMP_NUM_THREADS", "1");
    process.setProcessEnvironment(environment);
    process.start(m_python, {m_script, "--raw", m_audio->fileName(), "--sample-rate", QString::number(m_rate)});
    if (!process.waitForStarted(5000) || !process.waitForFinished(120000) ||
            process.exitStatus() != QProcess::NormalExit || process.exitCode() != 0) {
        process.kill();
        process.waitForFinished(1000);
        qWarning() << "Vocal analysis failed:" << process.readAllStandardError().left(1000);
        fail();
        return;
    }
    const QByteArray json = process.readAllStandardOutput();
    const auto result = VocalAnalysis::fromJson(QJsonDocument::fromJson(json).object());
    if (!result) {
        fail();
        return;
    }
    QSaveFile cache(m_cache);
    if (cache.open(QIODevice::WriteOnly) && cache.write(json) == json.size()) {
        cache.commit();
    }
    VocalAnalysisStore::instance().set(m_location, result);
}

void AnalyzerVocals::fail() {
    if (!m_location.isEmpty()) {
        auto result = std::make_shared<VocalAnalysis>();
        result->status = VocalAnalysis::Status::Failed;
        VocalAnalysisStore::instance().set(m_location, result);
    }
}

void AnalyzerVocals::cleanup() {
    if (m_audio && !m_location.isEmpty()) {
        const auto result = VocalAnalysisStore::instance().get(m_location);
        if (result && result->status == VocalAnalysis::Status::Pending) fail();
    }
    m_audio.reset();
    m_mono.clear();
    m_location.clear();
}
