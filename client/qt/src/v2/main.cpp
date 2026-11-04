#include "V2Window.h"
#include "Translator.h"

#include <QApplication>
#include <QFont>
#include <QSettings>
#include <QTimer>

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);
    QCoreApplication::setOrganizationName(QStringLiteral("SilentStudio"));
    QCoreApplication::setApplicationName(QStringLiteral("NetTool ζ"));
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

    QFont font = QApplication::font();
    font.setPointSize(10);
    app.setFont(font);

    V2Window window;
    const int pageIndex = arguments.indexOf(QStringLiteral("--page"));
    if (pageIndex >= 0 && pageIndex + 1 < arguments.size()) {
        window.setPage(arguments.at(pageIndex + 1).toInt());
    }
    window.show();

    const int screenshotIndex = arguments.indexOf(QStringLiteral("--screenshot"));
    if (screenshotIndex >= 0 && screenshotIndex + 1 < arguments.size()) {
        const QString path = arguments.at(screenshotIndex + 1);
        QTimer::singleShot(300, &app, [&window, &app, path] {
            window.grab().save(path);
            app.quit();
        });
    } else if (arguments.contains(QStringLiteral("--smoke-test"))) {
        QTimer::singleShot(250, &app, &QCoreApplication::quit);
    }

    return app.exec();
}
