#include "Translator.h"

#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QLocale>
#include <QRegularExpression>
#include <QStringConverter>
#include <QTextStream>

Translator &Translator::instance() {
    static Translator translator;
    return translator;
}

QString Translator::languageDir() const {
    return QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral("lang"));
}

bool Translator::loadFile(const QString &path, QHash<QString, QString> &target) const {
    QFile file(path);
    if (!file.open(QIODevice::ReadOnly | QIODevice::Text)) {
        return false;
    }

    QTextStream stream(&file);
    stream.setEncoding(QStringConverter::Utf8);
    bool loaded = false;
    while (!stream.atEnd()) {
        QString line = stream.readLine();
        if (line.startsWith(QChar(0xFEFF))) {
            line.remove(0, 1);
        }
        line = line.trimmed();
        if (line.isEmpty() || line.startsWith(QLatin1Char('#'))) {
            continue;
        }
        const int separator = line.indexOf(QLatin1Char('='));
        if (separator <= 0) {
            continue;
        }
        const QString key = line.left(separator).trimmed();
        const QString value = line.mid(separator + 1).trimmed();
        if (!key.isEmpty()) {
            target.insert(key, value);
            loaded = true;
        }
    }
    return loaded;
}

bool Translator::load(const QString &requested) {
    primary_.clear();
    fallback_.clear();

    QString code = requested;
    static const QRegularExpression validCode(QStringLiteral("^[A-Za-z0-9-]+$"));
    if (code.isEmpty() || !validCode.match(code).hasMatch()) {
        code = systemDefault();
    }

    const QString dir = languageDir();
    loadFile(QDir(dir).filePath(QStringLiteral("en.lang")), fallback_);

    if (code == QStringLiteral("en")) {
        primary_ = fallback_;
    } else {
        loadFile(QDir(dir).filePath(code + QStringLiteral(".lang")), primary_);
        if (primary_.isEmpty()) {
            code = QStringLiteral("en");
            primary_ = fallback_;
        }
    }

    code_ = code;
    return !primary_.isEmpty();
}

QString Translator::text(const QString &key) const {
    QString value = primary_.value(key);
    if (value.isEmpty()) {
        value = fallback_.value(key, key);
    }
    value.replace(QStringLiteral("\\n"), QStringLiteral("\n"));
    return value;
}

QString Translator::code() const {
    return code_;
}

QList<QPair<QString, QString>> Translator::languages() const {
    QList<QPair<QString, QString>> result;
    const QString dir = languageDir();
    QFile index(QDir(dir).filePath(QStringLiteral("languages.conf")));
    if (index.open(QIODevice::ReadOnly | QIODevice::Text)) {
        QTextStream stream(&index);
        stream.setEncoding(QStringConverter::Utf8);
        while (!stream.atEnd()) {
            QString line = stream.readLine().trimmed();
            if (line.isEmpty() || line.startsWith(QLatin1Char('#'))) {
                continue;
            }
            const int separator = line.indexOf(QLatin1Char('='));
            if (separator <= 0) {
                continue;
            }
            result.append({line.left(separator).trimmed(), line.mid(separator + 1).trimmed()});
        }
    }

    if (result.isEmpty()) {
        const QStringList files = QDir(dir).entryList({QStringLiteral("*.lang")}, QDir::Files, QDir::Name);
        for (const QString &file : files) {
            const QString code = file.left(file.size() - 5);
            result.append({code, code});
        }
    }
    return result;
}

QString Translator::systemDefault() {
    QString name = QLocale::system().name();
    name.replace(QLatin1Char('_'), QLatin1Char('-'));
    if (name.startsWith(QStringLiteral("zh"))) {
        return QStringLiteral("zh-CN");
    }
    return QStringLiteral("en");
}
