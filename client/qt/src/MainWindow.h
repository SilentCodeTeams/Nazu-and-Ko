#pragma once

#include <QMainWindow>
#include <QPoint>
#include <QString>

class QAction;
class QComboBox;
class QDockWidget;
class QFrame;
class QHBoxLayout;
class QLabel;
class QLineEdit;
class QTabWidget;
class QToolBar;
class QToolButton;
class NotificationCenter;
class PacketEditor;
class WebDashboardServer;
class QPoint;
class QProcess;
class QPushButton;
class QTableWidget;
class QWheelEvent;

class MainWindow : public QMainWindow {
    Q_OBJECT

public:
    explicit MainWindow(QWidget *parent = nullptr);
    ~MainWindow() override = default;

protected:
    void wheelEvent(QWheelEvent *event) override;
    bool eventFilter(QObject *watched, QEvent *event) override;

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
    QWidget *createTitleBar();
    QWidget *createWorkspace();
    void applyControlsSide(const QString &side);
    QString t(const QString &key) const;
    QString findProxyBinary() const;
    void setProxyRunning(bool running);
    void addPacketLine(const QString &line);
    void configureWebDashboard();
    void updateWebDashboard();
    QWidget *createDashboard();
    QFrame *createMetricCard(const QString &title, const QString &value,
                             const QString &detail, QLabel **valueLabel,
                             QLabel **detailLabel = nullptr);
    void applyTheme(const QString &theme);
    void applyZoom();

    QTableWidget *packetTable_ = nullptr;
    QToolBar *toolbar_ = nullptr;
    QFrame *titleBar_ = nullptr;
    QWidget *windowControls_ = nullptr;
    QHBoxLayout *titleLayout_ = nullptr;
    QTabWidget *inspectorTabs_ = nullptr;
    QPoint dragOffset_;
    QDockWidget *packetDock_ = nullptr;
    QDockWidget *requestDock_ = nullptr;
    QDockWidget *responseDock_ = nullptr;
    PacketEditor *requestEditor_ = nullptr;
    PacketEditor *responseEditor_ = nullptr;
    QLineEdit *filterInput_ = nullptr;
    QComboBox *themeCombo_ = nullptr;
    QAction *startAction_ = nullptr;
    QAction *stopAction_ = nullptr;
    QLabel *statusLabel_ = nullptr;
    QLabel *proxyStatusLabel_ = nullptr;
    QLabel *packetCountLabel_ = nullptr;
    QLabel *themeLabel_ = nullptr;
    QLabel *proxyPill_ = nullptr;
    QLabel *proxyCardValue_ = nullptr;
    QLabel *proxyCardDetail_ = nullptr;
    QLabel *emptyHint_ = nullptr;
    QLabel *sessionCardValue_ = nullptr;
    QLabel *packetCardValue_ = nullptr;
    QPushButton *primaryStartButton_ = nullptr;
    QProcess *proxyProcess_ = nullptr;
    NotificationCenter *notifications_ = nullptr;
    WebDashboardServer *webDashboard_ = nullptr;
    bool proxyRunning_ = false;

    QString theme_ = QStringLiteral("system");
    bool dark_ = false;
    double zoom_ = 1.0;
    int packetCount_ = 0;
};
