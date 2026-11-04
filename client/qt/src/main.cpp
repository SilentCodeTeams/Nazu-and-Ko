#include "MainWindow.h"

#include <QApplication>
#include <QFont>
#include <QTimer>

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);
    app.setApplicationName(QStringLiteral("NetTool ζ"));
    app.setOrganizationName(QStringLiteral("SilentStudio"));
    app.setStyle(QStringLiteral("Fusion"));

    QFont font;
#ifdef Q_OS_DARWIN
    font.setFamily(QStringLiteral("Helvetica"));
#elif defined(Q_OS_WIN)
    font.setFamily(QStringLiteral("Segoe UI"));
#else
    font.setFamily(QStringLiteral("Helvetica"));
#endif
    font.setPointSize(9);
    app.setFont(font);

    MainWindow window;
    window.show();

    const QStringList arguments = app.arguments();
    const int screenshotIndex = arguments.indexOf(QStringLiteral("--screenshot"));
    if (screenshotIndex >= 0 && screenshotIndex + 1 < arguments.size()) {
        const QString path = arguments.at(screenshotIndex + 1);
        QTimer::singleShot(250, &app, [&window, &app, path] {
            window.grab().save(path);
            app.quit();
        });
    } else if (arguments.contains(QStringLiteral("--smoke-test"))) {
        QTimer::singleShot(200, &app, &QCoreApplication::quit);
    }

    return app.exec();
}
