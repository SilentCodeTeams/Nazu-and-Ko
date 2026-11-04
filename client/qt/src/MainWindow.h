#pragma once

#include <QMainWindow>
#include <QString>

class QAction;
class QComboBox;
class QDockWidget;
class QLabel;
class QLineEdit;
class QPlainTextEdit;
class QPoint;
class QTableWidget;
class QWheelEvent;

class MainWindow : public QMainWindow {
    Q_OBJECT

public:
    explicit MainWindow(QWidget *parent = nullptr);
    ~MainWindow() override = default;

protected:
    void wheelEvent(QWheelEvent *event) override;

private slots:
    void startProxy();
    void stopProxy();
    void clearPackets();
    void exportPackets();
    void showProxyConfig();
    void showRules();
    void showRepeater();
    void showIntruder();
    void showScanner();
    void showSettings();
    void showTutorial();
    void showAbout();
    void applyFilter(const QString &text);
    void onThemeComboChanged(int index);
    void setTheme(const QString &theme);
    void zoomIn();
    void zoomOut();
    void resetZoom();
    void showPacketContextMenu(const QPoint &position);
    void onPacketDoubleClicked(int row, int column);

private:
    void createMenus();
    void createToolBar();
    void createDocks();
    void createStatusBar();
    void applyTheme(const QString &theme);
    void applyZoom();

    QTableWidget *packetTable_ = nullptr;
    QDockWidget *packetDock_ = nullptr;
    QDockWidget *requestDock_ = nullptr;
    QDockWidget *responseDock_ = nullptr;
    QPlainTextEdit *requestEditor_ = nullptr;
    QPlainTextEdit *responseEditor_ = nullptr;
    QLineEdit *filterInput_ = nullptr;
    QComboBox *themeCombo_ = nullptr;
    QAction *startAction_ = nullptr;
    QAction *stopAction_ = nullptr;
    QLabel *statusLabel_ = nullptr;
    QLabel *proxyStatusLabel_ = nullptr;
    QLabel *packetCountLabel_ = nullptr;
    QLabel *themeLabel_ = nullptr;

    QString theme_ = QStringLiteral("system");
    bool dark_ = false;
    double zoom_ = 1.0;
    int packetCount_ = 0;
};
