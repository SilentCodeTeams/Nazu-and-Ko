#pragma once

#include <QMainWindow>
#include <QString>

class QButtonGroup;
class QLabel;
class QStackedWidget;
class QToolButton;

class V2Window : public QMainWindow {
    Q_OBJECT

public:
    explicit V2Window(QWidget *parent = nullptr);
    void setPage(int page);

private:
    QWidget *createTopBar();
    QWidget *createRail();
    QWidget *createHomePage();
    QWidget *createTrafficPage();
    QWidget *createToolsPage();
    QWidget *createSettingsPage();
    QWidget *createPlaceholderPage(const QString &titleKey, const QString &subtitleKey);
    QWidget *createActionCard(const QString &index, const QString &titleKey,
                              const QString &descriptionKey, int page);
    void navigate(int page);
    void applyTheme(bool dark);

    QStackedWidget *stack_ = nullptr;
    QButtonGroup *railGroup_ = nullptr;
    QLabel *statusPill_ = nullptr;
    QLabel *themeButton_ = nullptr;
    bool dark_ = false;
};
