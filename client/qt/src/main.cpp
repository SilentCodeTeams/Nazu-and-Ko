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

#include "Dialogs.h"
#include "MainWindow.h"
#include "Tools.h"
#include "Translator.h"

#include <QApplication>
#include <QFont>
#include <QSettings>
#include <QTimer>

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);
    QCoreApplication::setOrganizationName(QStringLiteral("SilentStudio"));
    QCoreApplication::setApplicationName(QStringLiteral("Nazu&Ko"));
    app.setStyle(QStringLiteral("Fusion"));

    const QStringList arguments = app.arguments();
    QSettings settings;
    QString language = settings.value(QStringLiteral("ui/language")).toString();
    const int languageIndex = arguments.indexOf(QStringLiteral("--lang"));
    if (languageIndex >= 0 && languageIndex + 1 < arguments.size()) {
        language = arguments.at(languageIndex + 1);
    }
    if (language.isEmpty()) {
        language = Translator::systemDefault();
    }
    Translator::instance().load(language);
    app.setApplicationDisplayName(Translator::instance().text(QStringLiteral("app.name")));

    QFont font;
#ifdef Q_OS_DARWIN
    font.setFamily(QStringLiteral("Helvetica"));
#elif defined(Q_OS_WIN)
    font.setFamily(QStringLiteral("Segoe UI"));
#else
    font.setFamily(QStringLiteral("Helvetica"));
#endif
    font.setPointSize(10);
    app.setFont(font);

    MainWindow window;
    window.show();

    const int screenshotIndex = arguments.indexOf(QStringLiteral("--screenshot"));
    if (screenshotIndex >= 0 && screenshotIndex + 1 < arguments.size()) {
        const QString path = arguments.at(screenshotIndex + 1);
        QTimer::singleShot(250, &app, [&window, &app, path] {
            window.grab().save(path);
            app.quit();
        });
    } else if (arguments.contains(QStringLiteral("--dialog-smoke"))) {
        ProxyConfigDialog proxy(ProxySettings(), &window);
        SettingsDialog settings(&window);
        RepeaterDialog repeater(&window);
        RuleManagerDialog rules(&window);
        IntruderDialog intruder(&window);
        ScannerDialog scanner(&window);
        SmartExportDialog exportDialog(&window);
        QTimer::singleShot(200, &app, &QCoreApplication::quit);
    } else if (arguments.contains(QStringLiteral("--smoke-test"))) {
        QTimer::singleShot(200, &app, &QCoreApplication::quit);
    }

    return app.exec();
}
