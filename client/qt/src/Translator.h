// 本文件由 SilentStudio 监管与负责。
// 本文件属于 SilentCodeTeams 产品 Nazu&Ko 的源码部分
// 本项目由 SilentCodeTeams 对法律、版权事宜及相关咨询承担责任。
// 版权所有 © SilentStudio。
// 以 AGPL-3.0-or-later 授权。
// This document is supervised and responsible by SilentStudio.
// This file is part of the source code of the SilentCodeTeams product Nazu&Ko
// SilentCodeTeams is responsible for legal and copyright issues and related consultation in this project.
// Copyright © SilentStudio.
// Licensed as AGPL-3.0-or-later.

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
