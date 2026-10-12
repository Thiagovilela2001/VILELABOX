#include "analyzer/analyzervocals.h"

#include <QCryptographicHash>
#include <QDir>
#include <QDateTime>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QProcess>
#include <QProcessEnvironment>
#include <QRegularExpression>
#include <QElapsedTimer>
#include <QHash>
#include <QMutex>
#include <QSaveFile>
#include <QStandardPaths>
#ifdef Q_OS_LINUX
#include <csignal>
#include <sys/prctl.h>
#endif

#include "analyzer/vocalanalysisstore.h"
#include "track/track.h"

namespace {
const ConfigKey kPythonKey("[Vilelabox]", "VocalPython");
const ConfigKey kTimeoutKey("[Vilelabox]", "VocalTimeoutSeconds");
}

AnalyzerVocals::AnalyzerVocals(UserSettingsPointer config, std::function<bool()> isStopping)
        : m_config(std::move(config)), m_isStopping(std::move(isStopping)) {
}

QString AnalyzerVocals::probeEngine(const QString& python, const QString& script) {
    // One short process per interpreter and session; it only checks whether
    // the demucs package can be imported, without loading it.
    static QMutex mutex;
    static QHash<QString, QString> engines;
    const QString key = python + QChar('\n') + script;
    QMutexLocker lock(&mutex);
    const auto it = engines.constFind(key);
    if (it != engines.cend()) return it.value();
    QProcess process;
    process.start(python, {script, "--probe"});
    QString engine = QStringLiteral("classifier");
    if (process.waitForStarted(5000) && process.waitForFinished(5000) && process.exitCode() == 0) {
        const QString output = QString::fromUtf8(process.readAllStandardOutput()).trimmed();
        if (output == QStringLiteral("demucs")) engine = output;
    } else {
        process.kill();
        process.waitForFinished(1000);
    }
    engines.insert(key, engine);
    return engine;
}

QString AnalyzerVocals::resolvePython(const UserSettingsPointer& config) {
    const QString configured = config->getValueString(kPythonKey).trimmed();
    if (!configured.isEmpty()) return configured;
    const QString environment = qEnvironmentVariable("VILELABOX_VOCAL_PYTHON");
    if (!environment.isEmpty()) return environment;
    const QString venv = QDir::homePath() + "/.venv/bin/python";
    if (QFileInfo::exists(venv)) return venv;
    const QString system = QStandardPaths::findExecutable("python3");
    return system.isEmpty() ? venv : system;
}

bool AnalyzerVocals::initialize(const AnalyzerTrack& track, mixxx::audio::SampleRate rate, SINT length) {
    cleanup();
    if (length <= 0 || !rate.isValid()) {
        return false;
    }
    m_location = track.getTrack()->getLocation();
    m_rate = static_cast<int>(rate);
    m_script = QDir(m_config->getResourcePath()).filePath("vocals/vocal_detector.py");
    m_python = resolvePython(m_config);
    QFile model(QFileInfo(m_script).dir().filePath("model.json"));
    QFile script(m_script);
    if (!QFileInfo(m_python).isExecutable() && QStandardPaths::findExecutable(m_python).isEmpty()) {
        fail(VocalAnalysis::Error::PythonMissing, m_python);
        return false;
    }
    if (!model.open(QIODevice::ReadOnly) || !script.open(QIODevice::ReadOnly)) {
        fail(VocalAnalysis::Error::ResourcesMissing, m_script);
        return false;
    }
    const QFileInfo source(m_location);
    // The engine (demucs or classifier) is part of the identity: installing
    // Demucs later makes tracks analysed by the classifier run again.
    QByteArray identity = m_location.toUtf8() + QByteArray::number(source.size()) +
            QByteArray::number(source.lastModified().toMSecsSinceEpoch()) + model.readAll() + script.readAll() +
            probeEngine(m_python, m_script).toUtf8();
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
        fail(VocalAnalysis::Error::DetectorFailed, QObject::tr("não foi possível criar o arquivo temporário em %1").arg(QDir::tempPath()));
        return false;
    }
    return true;
}

bool AnalyzerVocals::processSamples(const CSAMPLE* samples, SINT count) {
    if (!m_audio || count % 2 != 0) {
        fail(VocalAnalysis::Error::DetectorFailed, QObject::tr("áudio em formato inesperado"));
        return false;
    }
    m_mono.resize(count / 2);
    for (SINT frame = 0; frame < count / 2; ++frame) {
        m_mono[frame] = (samples[frame * 2] + samples[frame * 2 + 1]) * 0.5f;
    }
    const qint64 size = static_cast<qint64>(m_mono.size() * sizeof(float));
    if (m_audio->write(reinterpret_cast<const char*>(m_mono.data()), size) != size) {
        fail(VocalAnalysis::Error::DetectorFailed, QObject::tr("falha ao gravar o arquivo temporário (disco cheio?)"));
        return false;
    }
    return true;
}

void AnalyzerVocals::storeResults(TrackPointer track) {
    Q_UNUSED(track);
    if (!m_audio || !m_audio->flush()) {
        fail(VocalAnalysis::Error::DetectorFailed, QObject::tr("falha ao gravar o arquivo temporário (disco cheio?)"));
        return;
    }
    QProcess process;
    auto environment = QProcessEnvironment::systemEnvironment();
    environment.insert("OPENBLAS_NUM_THREADS", "1");
    environment.insert("OMP_NUM_THREADS", "1");
    process.setProcessEnvironment(environment);
#ifdef Q_OS_LINUX
    // The detector dies with Mixxx, even if Mixxx crashes.
    process.setChildProcessModifier([] { prctl(PR_SET_PDEATHSIG, SIGKILL); });
#endif
    process.start(m_python, {m_script, "--raw", m_audio->fileName(), "--sample-rate", QString::number(m_rate)});
    if (!process.waitForStarted(5000)) {
        fail(VocalAnalysis::Error::PythonMissing, m_python);
        return;
    }
    // Demucs separates the whole track on the CPU (about a third of real time
    // on a desktop), hence the generous default.
    const int timeoutSeconds = std::max(1, m_config->getValue(kTimeoutKey, 900));
    QElapsedTimer elapsed;
    elapsed.start();
    // Short slices: a stop request is honoured within ~100 ms.
    while (!process.waitForFinished(100)) {
        const bool stopping = m_isStopping && m_isStopping();
        if (stopping || elapsed.elapsed() > timeoutSeconds * 1000LL) {
            process.kill();
            process.waitForFinished(2000);
            if (stopping) fail(VocalAnalysis::Error::Cancelled);
            else fail(VocalAnalysis::Error::Timeout, QString::number(timeoutSeconds));
            return;
        }
    }
    if (process.exitStatus() != QProcess::NormalExit || process.exitCode() != 0) {
        const QString error = QString::fromUtf8(process.readAllStandardError()).trimmed();
        qWarning() << "Vocal analysis failed:" << error.left(1000);
        static const QRegularExpression missingModule(QStringLiteral("No module named '([^']+)'"));
        const auto match = missingModule.match(error);
        if (match.hasMatch()) {
            fail(VocalAnalysis::Error::DependenciesMissing, match.captured(1));
        } else {
            const QString last = error.section('\n', -1).trimmed();
            fail(VocalAnalysis::Error::DetectorFailed,
                    last.isEmpty() ? QObject::tr("código de saída %1").arg(process.exitCode()) : last.left(200));
        }
        return;
    }
    const QByteArray json = process.readAllStandardOutput();
    const auto result = VocalAnalysis::fromJson(QJsonDocument::fromJson(json).object());
    if (!result) {
        fail(VocalAnalysis::Error::InvalidResult);
        return;
    }
    QSaveFile cache(m_cache);
    if (cache.open(QIODevice::WriteOnly) && cache.write(json) == json.size()) {
        cache.commit();
    }
    VocalAnalysisStore::instance().set(m_location, result);
}

void AnalyzerVocals::fail(VocalAnalysis::Error error, const QString& detail) {
    if (!m_location.isEmpty()) {
        auto result = std::make_shared<VocalAnalysis>();
        result->status = VocalAnalysis::Status::Failed;
        result->error = error;
        result->detail = detail;
        VocalAnalysisStore::instance().set(m_location, result);
    }
}

void AnalyzerVocals::cleanup() {
    if (m_audio && !m_location.isEmpty()) {
        const auto result = VocalAnalysisStore::instance().get(m_location);
        // Cleaned up before a result: the analysis was cancelled.
        if (result && result->status == VocalAnalysis::Status::Pending) fail(VocalAnalysis::Error::Cancelled);
    }
    m_audio.reset();
    m_mono.clear();
    m_location.clear();
}
