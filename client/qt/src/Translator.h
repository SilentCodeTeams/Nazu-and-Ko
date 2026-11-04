#pragma once

#include <QHash>
#include <QList>
#include <QPair>
#include <QString>

class Translator {
public:
    static Translator &instance();

    bool load(const QString &code);
    QString text(const QString &key) const;
    QString code() const;
    QList<QPair<QString, QString>> languages() const;
    static QString systemDefault();

private:
    Translator() = default;

    QString languageDir() const;
    bool loadFile(const QString &path, QHash<QString, QString> &target) const;

    QHash<QString, QString> primary_;
    QHash<QString, QString> fallback_;
    QString code_ = QStringLiteral("en");
};
