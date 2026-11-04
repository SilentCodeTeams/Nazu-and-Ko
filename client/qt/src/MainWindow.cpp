#include "MainWindow.h"

#include "Dialogs.h"
#include "NotificationCenter.h"
#include "PacketEditor.h"
#include "WebDashboard.h"
#include "Tools.h"
#include "Translator.h"

#include <QAbstractItemView>
#include <QAction>
#include <QActionGroup>
#include <QApplication>
#include <QClipboard>
#include <QComboBox>
#include <QCoreApplication>
#include <QDockWidget>
#include <QFile>
#include <QFileDialog>
#include <QFileInfo>
#include <QFont>
#include <QFontDatabase>
#include <QFrame>
#include <QGridLayout>
#include <QGuiApplication>
#include <QHBoxLayout>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QHeaderView>
#include <QKeySequence>
#include <QLabel>
#include <QLineEdit>
#include <QMenu>
#include <QMenuBar>
#include <QMessageBox>
#include <QMouseEvent>
#include <QPlainTextEdit>
#include <QPoint>
#include <QProcess>
#include <QPushButton>
#include <QRegularExpression>
#include <QSettings>
#include <QSignalBlocker>
#include <QSizeGrip>
#include <QSplitter>
#include <QSizePolicy>
#include <QStandardPaths>
#include <QStatusBar>
#include <QStringConverter>
#include <QStyle>
#include <QStyleHints>
#include <QTabWidget>
#include <QTableWidget>
#include <QTableWidgetItem>
#include <QTextStream>
#include <QTime>
#include <QTimer>
#include <QToolBar>
#include <QToolButton>
#include <QVBoxLayout>
#include <QWheelEvent>
#include <QWidget>

namespace {

QString lightStyle() {
    return QStringLiteral(R"STYLE(
QMainWindow { background: #F5F6F8; }
QFrame#titleBar { background: #FFFFFF; border-bottom: 1px solid #E4E7EC; }
QLabel#titleBrand { color: #111827; font-size: 12px; font-weight: 700; }
QLabel#titleMeta { color: #667085; font-size: 10px; }
QFrame#workspacePanel { background: #FFFFFF; border: 1px solid #E4E7EC; border-radius: 10px; }
QTabWidget#inspectorTabs::pane { border: none; background: #FFFFFF; }
QWidget#dashboard { background: #F5F6F8; }
QWidget { color: #111827; }
QLabel { background: transparent; }
QLabel#eyebrow { color: #111111; font-size: 10px; font-weight: 700; }
QLabel#pageTitle { color: #111827; font-size: 24px; font-weight: 700; }
QLabel#muted, QLabel#metricDetail, QLabel#statusText, QLabel#statusMeta { color: #667085; }
QLabel#metricLabel { color: #667085; font-size: 11px; font-weight: 600; }
QLabel#metricValue { color: #111827; font-size: 26px; font-weight: 700; }
QLabel#sectionTitle { color: #111827; font-size: 14px; font-weight: 700; }
QLabel#caption { color: #98A2B3; font-size: 10px; }
QLabel#toolbarLabel { color: #667085; font-size: 11px; }
QLabel#emptyHint { color: #667085; background: #F9FAFB; border-bottom: 1px solid #E4E7EC; padding: 10px 12px; }
QLabel#pill { color: #475467; background: #EEF2F6; border: 1px solid #E4E7EC; border-radius: 10px; padding: 3px 9px; font-size: 11px; font-weight: 600; }
QLabel#pill[state="offline"] { color: #525252; background: #F5F5F5; border-color: #E5E5E5; }
QLabel#pill[state="online"] { color: #111111; background: #FAFAFA; border-color: #D4D4D4; }
QFrame#metricCard, QFrame#panel { background: #FFFFFF; border: 1px solid #E4E7EC; border-radius: 14px; }
QPushButton#primaryButton { color: #FFFFFF; background: #111111; border: none; border-radius: 8px; padding: 8px 14px; font-weight: 600; }
QPushButton#primaryButton:hover { background: #000000; }
QPushButton#primaryButton:pressed { background: #000000; }
QPushButton#ghostButton { color: #111111; background: transparent; border: 1px solid #CBD5E1; border-radius: 8px; padding: 7px 12px; font-weight: 600; }
QPushButton#ghostButton:hover { background: #FAFAFA; border-color: #D4D4D4; }
QTableWidget { background: #FFFFFF; alternate-background-color: #F9FAFB; color: #111827; border: 1px solid #E4E7EC; border-radius: 8px; gridline-color: transparent; selection-background-color: #E5E5E5; selection-color: #111827; }
QHeaderView::section { background: #FFFFFF; color: #667085; border: none; border-bottom: 1px solid #E4E7EC; padding: 8px 6px; font-weight: 600; }
QTableWidget::item { padding: 5px; }
QPlainTextEdit { background: #FFFFFF; color: #111827; border: 1px solid #E4E7EC; border-radius: 8px; padding: 10px; }
QLineEdit { background: #FFFFFF; color: #111827; border: 1px solid #D0D5DD; border-radius: 7px; padding: 6px 8px; }
QComboBox { background: #FFFFFF; color: #111827; border: 1px solid #D0D5DD; border-radius: 7px; padding: 5px 8px; }
QComboBox QAbstractItemView { background: #FFFFFF; color: #111827; selection-background-color: #FAFAFA; }
QToolBar { background: #FFFFFF; border: none; border-bottom: 1px solid #E4E7EC; spacing: 6px; padding: 5px 8px; }
QToolButton { color: #344054; background: transparent; border: none; border-radius: 6px; padding: 5px 8px; }
QToolButton:hover { background: #F2F4F7; }
QToolButton:pressed { background: #EAECF0; }
QMenuBar { background: #FFFFFF; color: #344054; border-bottom: 1px solid #E4E7EC; }
QMenuBar::item { padding: 5px 9px; background: transparent; }
QMenuBar::item:selected { background: #F2F4F7; }
QMenu { background: #FFFFFF; color: #111827; border: 1px solid #E4E7EC; padding: 4px; }
QMenu::item { padding: 6px 22px 6px 10px; border-radius: 4px; }
QMenu::item:selected { background: #FAFAFA; }
QStatusBar { background: #FFFFFF; color: #667085; border-top: 1px solid #E4E7EC; }
QStatusBar::item { border: none; }
QDockWidget { color: #344054; }
QDockWidget::title { background: #FFFFFF; color: #344054; padding: 7px 10px; border-bottom: 1px solid #E4E7EC; font-weight: 600; }
QTabBar::tab { background: #F2F4F7; color: #667085; padding: 6px 12px; border-top-left-radius: 6px; border-top-right-radius: 6px; margin-right: 2px; }
QTabBar::tab:selected { background: #FFFFFF; color: #111827; border: 1px solid #E4E7EC; border-bottom-color: #FFFFFF; }
QScrollBar:vertical { background: #F5F6F8; width: 10px; }
QScrollBar::handle:vertical { background: #D0D5DD; border-radius: 5px; min-height: 24px; }
QDialog { background: #F5F6F8; }
QPushButton { background: #FFFFFF; color: #344054; border: 1px solid #D0D5DD; border-radius: 7px; padding: 6px 12px; }
QPushButton:hover { background: #F9FAFB; }
QSpinBox { background: #FFFFFF; color: #111827; border: 1px solid #D0D5DD; border-radius: 7px; padding: 4px 6px; }
QCheckBox { color: #344054; }
QTabWidget::pane { border: 1px solid #E4E7EC; border-radius: 8px; background: #FFFFFF; }
)STYLE");
}

QString darkStyle() {
    return QStringLiteral(R"STYLE(
QMainWindow { background: #0B0F14; }
QFrame#titleBar { background: #11161F; border-bottom: 1px solid #232B36; }
QLabel#titleBrand { color: #E6EAF0; font-size: 12px; font-weight: 700; }
QLabel#titleMeta { color: #8B96A7; font-size: 10px; }
QFrame#workspacePanel { background: #11161F; border: 1px solid #232B36; border-radius: 10px; }
QTabWidget#inspectorTabs::pane { border: none; background: #11161F; }
QWidget#dashboard { background: #0B0F14; }
QWidget { color: #E6EAF0; }
QLabel { background: transparent; }
QLabel#eyebrow { color: #FFFFFF; font-size: 10px; font-weight: 700; }
QLabel#pageTitle { color: #E6EAF0; font-size: 24px; font-weight: 700; }
QLabel#muted, QLabel#metricDetail, QLabel#statusText, QLabel#statusMeta { color: #8B96A7; }
QLabel#metricLabel { color: #8B96A7; font-size: 11px; font-weight: 600; }
QLabel#metricValue { color: #E6EAF0; font-size: 26px; font-weight: 700; }
QLabel#sectionTitle { color: #E6EAF0; font-size: 14px; font-weight: 700; }
QLabel#caption { color: #64748B; font-size: 10px; }
QLabel#toolbarLabel { color: #8B96A7; font-size: 11px; }
QLabel#emptyHint { color: #8B96A7; background: #0F141B; border-bottom: 1px solid #232B36; padding: 10px 12px; }
QLabel#pill { color: #A7B3C2; background: #1A2230; border: 1px solid #232B36; border-radius: 10px; padding: 3px 9px; font-size: 11px; font-weight: 600; }
QLabel#pill[state="offline"] { color: #A3A3A3; background: #1A1A1A; border-color: #2A2A2A; }
QLabel#pill[state="online"] { color: #FFFFFF; background: #1A1A1A; border-color: #2A2A2A; }
QFrame#metricCard, QFrame#panel { background: #11161F; border: 1px solid #232B36; border-radius: 14px; }
QPushButton#primaryButton { color: #0A0A0A; background: #FFFFFF; border: none; border-radius: 8px; padding: 8px 14px; font-weight: 700; }
QPushButton#primaryButton:hover { background: #E5E5E5; }
QPushButton#primaryButton:pressed { background: #F5F5F5; }
QPushButton#ghostButton { color: #E5E5E5; background: transparent; border: 1px solid #334155; border-radius: 8px; padding: 7px 12px; font-weight: 600; }
QPushButton#ghostButton:hover { background: #1A1A1A; border-color: #FFFFFF; }
QTableWidget { background: #11161F; alternate-background-color: #0F141B; color: #E6EAF0; border: 1px solid #232B36; border-radius: 8px; gridline-color: transparent; selection-background-color: #2A2A2A; selection-color: #E6EAF0; }
QHeaderView::section { background: #11161F; color: #8B96A7; border: none; border-bottom: 1px solid #232B36; padding: 8px 6px; font-weight: 600; }
QTableWidget::item { padding: 5px; }
QPlainTextEdit { background: #11161F; color: #E6EAF0; border: 1px solid #232B36; border-radius: 8px; padding: 10px; }
QLineEdit { background: #11161F; color: #E6EAF0; border: 1px solid #334155; border-radius: 7px; padding: 6px 8px; }
QComboBox { background: #11161F; color: #E6EAF0; border: 1px solid #334155; border-radius: 7px; padding: 5px 8px; }
QComboBox QAbstractItemView { background: #11161F; color: #E6EAF0; selection-background-color: #1A1A1A; }
QToolBar { background: #11161F; border: none; border-bottom: 1px solid #232B36; spacing: 6px; padding: 5px 8px; }
QToolButton { color: #A7B3C2; background: transparent; border: none; border-radius: 6px; padding: 5px 8px; }
QToolButton:hover { background: #1A2230; }
QToolButton:pressed { background: #232B36; }
QMenuBar { background: #11161F; color: #A7B3C2; border-bottom: 1px solid #232B36; }
QMenuBar::item { padding: 5px 9px; background: transparent; }
QMenuBar::item:selected { background: #1A2230; }
QMenu { background: #11161F; color: #E6EAF0; border: 1px solid #232B36; padding: 4px; }
QMenu::item { padding: 6px 22px 6px 10px; border-radius: 4px; }
QMenu::item:selected { background: #1A1A1A; }
QStatusBar { background: #11161F; color: #8B96A7; border-top: 1px solid #232B36; }
QStatusBar::item { border: none; }
QDockWidget { color: #A7B3C2; }
QDockWidget::title { background: #11161F; color: #A7B3C2; padding: 7px 10px; border-bottom: 1px solid #232B36; font-weight: 600; }
QTabBar::tab { background: #0F141B; color: #8B96A7; padding: 6px 12px; border-top-left-radius: 6px; border-top-right-radius: 6px; margin-right: 2px; }
QTabBar::tab:selected { background: #11161F; color: #E6EAF0; border: 1px solid #232B36; border-bottom-color: #11161F; }
QScrollBar:vertical { background: #0B0F14; width: 10px; }
QScrollBar::handle:vertical { background: #334155; border-radius: 5px; min-height: 24px; }
QDialog { background: #0B0F14; }
QPushButton { background: #11161F; color: #E6EAF0; border: 1px solid #334155; border-radius: 7px; padding: 6px 12px; }
QPushButton:hover { background: #1A2230; }
QSpinBox { background: #11161F; color: #E6EAF0; border: 1px solid #334155; border-radius: 7px; padding: 4px 6px; }
QCheckBox { color: #A7B3C2; }
QTabWidget::pane { border: 1px solid #232B36; border-radius: 8px; background: #11161F; }
)STYLE");
}

} // namespace

MainWindow::MainWindow(QWidget *parent) : QMainWindow(parent) {
    setWindowTitle(t("app.title"));
    setWindowFlags(Qt::Window | Qt::FramelessWindowHint);
    setMinimumSize(1100, 700);
    setGeometry(50, 50, 1400, 900);

    QSettings settings;
    theme_ = settings.value(QStringLiteral("ui/theme"), QStringLiteral("system")).toString();
    notifications_ = new NotificationCenter(this);
    webDashboard_ = new WebDashboardServer(this);

    createMenus();

    auto *titleToolbar = new QToolBar(this);
    titleToolbar->setObjectName(QStringLiteral("title_toolbar"));
    titleToolbar->setMovable(false);
    titleToolbar->setFloatable(false);
    titleToolbar->addWidget(createTitleBar());
    addToolBar(Qt::TopToolBarArea, titleToolbar);

    createToolBar();
    createDocks();
    createStatusBar();
    setCentralWidget(createDashboard());
    setTheme(theme_);
    configureWebDashboard();
    updateWebDashboard();

    if (QSettings().value(QStringLiteral("proxy/autoStart"), false).toBool()) {
        QTimer::singleShot(500, this, &MainWindow::startProxy);
    }
}

QString MainWindow::t(const QString &key) const {
    return Translator::instance().text(key);
}

void MainWindow::createMenus() {
    QMenu *fileMenu = menuBar()->addMenu(t("menu.file"));
    QAction *exportAction = fileMenu->addAction(t("menu.export"));
    exportAction->setShortcut(QKeySequence(QStringLiteral("Ctrl+E")));
    connect(exportAction, &QAction::triggered, this, &MainWindow::exportPackets);
    QAction *clearAction = fileMenu->addAction(t("menu.clear"));
    clearAction->setShortcut(QKeySequence(QStringLiteral("Ctrl+L")));
    connect(clearAction, &QAction::triggered, this, &MainWindow::clearPackets);
    fileMenu->addSeparator();
    QAction *exitAction = fileMenu->addAction(t("menu.exit"));
    exitAction->setShortcut(QKeySequence(QStringLiteral("Alt+F4")));
    connect(exitAction, &QAction::triggered, this, &QWidget::close);

    QMenu *proxyMenu = menuBar()->addMenu(t("menu.proxy"));
    startAction_ = proxyMenu->addAction(t("menu.startProxy"));
    connect(startAction_, &QAction::triggered, this, &MainWindow::startProxy);
    stopAction_ = proxyMenu->addAction(t("menu.stopProxy"));
    stopAction_->setEnabled(false);
    connect(stopAction_, &QAction::triggered, this, &MainWindow::stopProxy);
    proxyMenu->addSeparator();
    QAction *configAction = proxyMenu->addAction(t("menu.proxyConfig"));
    connect(configAction, &QAction::triggered, this, &MainWindow::showProxyConfig);
    QAction *rulesAction = proxyMenu->addAction(t("menu.rules"));
    connect(rulesAction, &QAction::triggered, this, &MainWindow::showRules);

    QMenu *toolsMenu = menuBar()->addMenu(t("menu.tools"));
    QAction *repeaterAction = toolsMenu->addAction(t("menu.repeater"));
    repeaterAction->setShortcut(QKeySequence(QStringLiteral("Ctrl+R")));
    connect(repeaterAction, &QAction::triggered, this, &MainWindow::showRepeater);
    QAction *intruderAction = toolsMenu->addAction(t("menu.intruder"));
    intruderAction->setShortcut(QKeySequence(QStringLiteral("Ctrl+T")));
    connect(intruderAction, &QAction::triggered, this, &MainWindow::showIntruder);
    QAction *scannerAction = toolsMenu->addAction(t("menu.scanner"));
    scannerAction->setShortcut(QKeySequence(QStringLiteral("Ctrl+S")));
    connect(scannerAction, &QAction::triggered, this, &MainWindow::showScanner);

    QMenu *viewMenu = menuBar()->addMenu(t("menu.view"));
    QMenu *themeMenu = viewMenu->addMenu(t("menu.theme"));
    QAction *lightAction = themeMenu->addAction(t("theme.light"));
    connect(lightAction, &QAction::triggered, this, [this] { setTheme(QStringLiteral("light")); });
    QAction *darkAction = themeMenu->addAction(t("theme.dark"));
    connect(darkAction, &QAction::triggered, this, [this] { setTheme(QStringLiteral("dark")); });
    QAction *systemAction = themeMenu->addAction(t("theme.system"));
    connect(systemAction, &QAction::triggered, this, [this] { setTheme(QStringLiteral("system")); });
    viewMenu->addSeparator();
    QAction *resetZoomAction = viewMenu->addAction(t("menu.zoomReset"));
    resetZoomAction->setShortcut(QKeySequence(QStringLiteral("Ctrl+0")));
    connect(resetZoomAction, &QAction::triggered, this, &MainWindow::resetZoom);
    QAction *zoomInAction = viewMenu->addAction(t("menu.zoomIn"));
    zoomInAction->setShortcut(QKeySequence(QStringLiteral("Ctrl++")));
    connect(zoomInAction, &QAction::triggered, this, &MainWindow::zoomIn);
    QAction *zoomOutAction = viewMenu->addAction(t("menu.zoomOut"));
    zoomOutAction->setShortcut(QKeySequence(QStringLiteral("Ctrl+-")));
    connect(zoomOutAction, &QAction::triggered, this, &MainWindow::zoomOut);

    QMenu *settingsMenu = menuBar()->addMenu(t("menu.settings"));
    QAction *preferencesAction = settingsMenu->addAction(t("menu.preferences"));
    preferencesAction->setShortcut(QKeySequence(QStringLiteral("Ctrl+,")));
    connect(preferencesAction, &QAction::triggered, this, &MainWindow::showSettings);

    QMenu *languageMenu = settingsMenu->addMenu(t("menu.language"));
    auto *languageGroup = new QActionGroup(this);
    languageGroup->setExclusive(true);
    const auto languages = Translator::instance().languages();
    for (const auto &language : languages) {
        QAction *action = languageMenu->addAction(language.second);
        action->setCheckable(true);
        action->setChecked(language.first == Translator::instance().code());
        action->setData(language.first);
        languageGroup->addAction(action);
        const QString code = language.first;
        connect(action, &QAction::triggered, this, [this, code] {
            if (Translator::instance().code() == code) {
                return;
            }
            QSettings().setValue(QStringLiteral("ui/language"), code);
            QMessageBox::information(this, t("language.title"), t("language.restart"));
        });
    }

    QMenu *helpMenu = menuBar()->addMenu(t("menu.help"));
    QAction *tutorialAction = helpMenu->addAction(t("menu.tutorial"));
    connect(tutorialAction, &QAction::triggered, this, &MainWindow::showTutorial);
    helpMenu->addSeparator();
    QAction *aboutAction = helpMenu->addAction(t("menu.about"));
    connect(aboutAction, &QAction::triggered, this, &MainWindow::showAbout);
}

void MainWindow::createToolBar() {
    toolbar_ = new QToolBar(this);
    QToolBar *toolbar = toolbar_;
    toolbar->setObjectName(QStringLiteral("main_toolbar"));
    toolbar->setMovable(false);
    toolbar->setIconSize(QSize(14, 14));
    toolbar->setToolButtonStyle(Qt::ToolButtonTextBesideIcon);

    startAction_->setIcon(style()->standardIcon(QStyle::SP_MediaPlay));
    stopAction_->setIcon(style()->standardIcon(QStyle::SP_MediaStop));
    toolbar->addAction(startAction_);
    toolbar->addAction(stopAction_);
    toolbar->addSeparator();

    QAction *repeaterAction = toolbar->addAction(t("menu.repeater"));
    connect(repeaterAction, &QAction::triggered, this, &MainWindow::showRepeater);
    QAction *intruderAction = toolbar->addAction(t("menu.intruder"));
    connect(intruderAction, &QAction::triggered, this, &MainWindow::showIntruder);
    QAction *scannerAction = toolbar->addAction(t("menu.scanner"));
    connect(scannerAction, &QAction::triggered, this, &MainWindow::showScanner);
    toolbar->addSeparator();

    QAction *clearAction = toolbar->addAction(t("toolbar.clear"));
    connect(clearAction, &QAction::triggered, this, &MainWindow::clearPackets);
    toolbar->addSeparator();

    auto *filterLabel = new QLabel(t("toolbar.filter") + QStringLiteral(":"), this);
    filterLabel->setObjectName(QStringLiteral("toolbarLabel"));
    toolbar->addWidget(filterLabel);
    filterInput_ = new QLineEdit(this);
    filterInput_->setPlaceholderText(t("toolbar.filterPlaceholder"));
    filterInput_->setMinimumWidth(190);
    connect(filterInput_, &QLineEdit::textChanged, this, &MainWindow::applyFilter);
    toolbar->addWidget(filterInput_);
    toolbar->addSeparator();
    addToolBarBreak(Qt::TopToolBarArea);
    addToolBar(Qt::TopToolBarArea, toolbar_);
}

QWidget *MainWindow::createTitleBar() {
    titleBar_ = new QFrame(this);
    titleBar_->setObjectName(QStringLiteral("titleBar"));
    titleBar_->setFixedHeight(42);
    titleBar_->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    titleLayout_ = new QHBoxLayout(titleBar_);
    titleLayout_->setContentsMargins(14, 0, 14, 0);
    titleLayout_->setSpacing(10);

    windowControls_ = new QWidget(titleBar_);
    auto *controlsLayout = new QHBoxLayout(windowControls_);
    controlsLayout->setContentsMargins(0, 0, 0, 0);
    controlsLayout->setSpacing(7);

    auto addLight = [&](const QString &color, const QString &tooltip, auto action) {
        auto *button = new QToolButton(windowControls_);
        button->setFixedSize(13, 13);
        button->setCursor(Qt::PointingHandCursor);
        button->setToolTip(tooltip);
        button->setStyleSheet(QStringLiteral(
            "QToolButton { background: %1; border: none; border-radius: 6px; }")
                                   .arg(color));
        connect(button, &QToolButton::clicked, this, action);
        controlsLayout->addWidget(button);
    };
    addLight(QStringLiteral("#FF5F57"), t("window.close"), [this] { close(); });
    addLight(QStringLiteral("#FEBC2E"), t("window.minimize"), [this] { showMinimized(); });
    addLight(QStringLiteral("#28C840"), t("window.zoom"), [this] {
        isMaximized() ? showNormal() : showMaximized();
    });

    titleLayout_->addWidget(windowControls_);
    auto *brand = new QLabel(QStringLiteral("Nazu&Ko"), titleBar_);
    brand->setObjectName(QStringLiteral("titleBrand"));
    brand->setAttribute(Qt::WA_TransparentForMouseEvents);
    auto *meta = new QLabel(t("window.subtitle"), titleBar_);
    meta->setObjectName(QStringLiteral("titleMeta"));
    meta->setAttribute(Qt::WA_TransparentForMouseEvents);
    titleLayout_->addWidget(brand);
    titleLayout_->addWidget(meta);
    titleLayout_->addStretch(1);

    titleBar_->installEventFilter(this);
    applyControlsSide(QSettings().value(QStringLiteral("window/controlsSide"),
                                        QStringLiteral("left")).toString());
    return titleBar_;
}

void MainWindow::applyControlsSide(const QString &side) {
    if (!titleLayout_ || !windowControls_) {
        return;
    }
    titleLayout_->removeWidget(windowControls_);
    if (side == QStringLiteral("right")) {
        titleLayout_->addWidget(windowControls_);
    } else {
        titleLayout_->insertWidget(0, windowControls_);
    }
}

bool MainWindow::eventFilter(QObject *watched, QEvent *event) {
    if (watched == titleBar_) {
        auto *mouse = static_cast<QMouseEvent *>(event);
        if (event->type() == QEvent::MouseButtonPress && mouse->button() == Qt::LeftButton) {
            dragOffset_ = mouse->globalPosition().toPoint() - frameGeometry().topLeft();
            return true;
        }
        if (event->type() == QEvent::MouseMove && (mouse->buttons() & Qt::LeftButton)) {
            move(mouse->globalPosition().toPoint() - dragOffset_);
            return true;
        }
        if (event->type() == QEvent::MouseButtonDblClick) {
            isMaximized() ? showNormal() : showMaximized();
            return true;
        }
    }
    return QMainWindow::eventFilter(watched, event);
}

QWidget *MainWindow::createWorkspace() {
    auto *root = new QWidget(this);
    auto *layout = new QVBoxLayout(root);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(0);
    layout->addWidget(createTitleBar());
    if (toolbar_) {
        layout->addWidget(toolbar_);
    }

    auto *splitter = new QSplitter(Qt::Horizontal, root);
    splitter->setChildrenCollapsible(false);

    auto *tablePanel = new QFrame(splitter);
    tablePanel->setObjectName(QStringLiteral("workspacePanel"));
    auto *tableLayout = new QVBoxLayout(tablePanel);
    tableLayout->setContentsMargins(12, 10, 12, 12);
    tableLayout->setSpacing(6);
    auto *tableTitle = new QLabel(t("table.title"), tablePanel);
    tableTitle->setObjectName(QStringLiteral("sectionTitle"));
    emptyHint_ = new QLabel(t("table.empty"), tablePanel);
    emptyHint_->setObjectName(QStringLiteral("emptyHint"));
    emptyHint_->setWordWrap(true);
    packetTable_ = new QTableWidget(tablePanel);
    packetTable_->setColumnCount(6);
    packetTable_->setHorizontalHeaderLabels({
        t("table.id"), t("table.time"), t("table.direction"),
        t("table.length"), t("table.status"), t("table.summary"),
    });
    packetTable_->setAlternatingRowColors(true);
    packetTable_->setSelectionBehavior(QAbstractItemView::SelectRows);
    packetTable_->setSelectionMode(QAbstractItemView::SingleSelection);
    packetTable_->setEditTriggers(QAbstractItemView::NoEditTriggers);
    packetTable_->setShowGrid(false);
    packetTable_->verticalHeader()->setVisible(false);
    packetTable_->verticalHeader()->setDefaultSectionSize(30);
    packetTable_->horizontalHeader()->setHighlightSections(false);
    packetTable_->horizontalHeader()->setDefaultAlignment(Qt::AlignLeft | Qt::AlignVCenter);
    packetTable_->horizontalHeader()->setSectionResizeMode(5, QHeaderView::Stretch);
    packetTable_->setColumnWidth(0, 48);
    packetTable_->setColumnWidth(1, 138);
    packetTable_->setColumnWidth(2, 66);
    packetTable_->setColumnWidth(3, 66);
    packetTable_->setColumnWidth(4, 78);
    packetTable_->setContextMenuPolicy(Qt::CustomContextMenu);
    connect(packetTable_, &QTableWidget::customContextMenuRequested,
            this, &MainWindow::showPacketContextMenu);
    connect(packetTable_, &QTableWidget::cellDoubleClicked,
            this, &MainWindow::onPacketDoubleClicked);
    tableLayout->addWidget(tableTitle);
    tableLayout->addWidget(emptyHint_);
    tableLayout->addWidget(packetTable_, 1);

    auto *inspectorPanel = new QFrame(splitter);
    inspectorPanel->setObjectName(QStringLiteral("workspacePanel"));
    auto *inspectorLayout = new QVBoxLayout(inspectorPanel);
    inspectorLayout->setContentsMargins(8, 8, 8, 8);
    inspectorTabs_ = new QTabWidget(inspectorPanel);
    inspectorTabs_->setObjectName(QStringLiteral("inspectorTabs"));
    requestEditor_ = new PacketEditor(inspectorTabs_);
    requestEditor_->setPlaceholderText(t("inspector.requestPlaceholder"));
    responseEditor_ = new PacketEditor(inspectorTabs_);
    responseEditor_->setPlaceholderText(t("inspector.responsePlaceholder"));
    inspectorTabs_->addTab(requestEditor_, t("inspector.requestTitle"));
    inspectorTabs_->addTab(responseEditor_, t("inspector.responseTitle"));
    inspectorLayout->addWidget(inspectorTabs_, 1);

    splitter->addWidget(tablePanel);
    splitter->addWidget(inspectorPanel);
    splitter->setStretchFactor(0, 3);
    splitter->setStretchFactor(1, 2);
    layout->addWidget(splitter, 1);
    return root;
}
void MainWindow::createDocks() {
    packetDock_ = new QDockWidget(t("table.title"), this);
    packetDock_->setObjectName(QStringLiteral("packet_dock"));
    packetTable_ = new QTableWidget(this);
    packetTable_->setColumnCount(6);
    packetTable_->setHorizontalHeaderLabels({
        t("table.id"),
        t("table.time"),
        t("table.direction"),
        t("table.length"),
        t("table.status"),
        t("table.summary"),
    });
    packetTable_->setAlternatingRowColors(true);
    packetTable_->setSelectionBehavior(QAbstractItemView::SelectRows);
    packetTable_->setSelectionMode(QAbstractItemView::SingleSelection);
    packetTable_->setEditTriggers(QAbstractItemView::NoEditTriggers);
    packetTable_->setShowGrid(false);
    packetTable_->verticalHeader()->setVisible(false);
    packetTable_->verticalHeader()->setDefaultSectionSize(30);
    packetTable_->horizontalHeader()->setHighlightSections(false);
    packetTable_->horizontalHeader()->setDefaultAlignment(Qt::AlignLeft | Qt::AlignVCenter);
    packetTable_->horizontalHeader()->setSectionResizeMode(5, QHeaderView::Stretch);
    packetTable_->setColumnWidth(0, 48);
    packetTable_->setColumnWidth(1, 138);
    packetTable_->setColumnWidth(2, 66);
    packetTable_->setColumnWidth(3, 66);
    packetTable_->setColumnWidth(4, 78);
    packetTable_->setContextMenuPolicy(Qt::CustomContextMenu);
    connect(packetTable_, &QTableWidget::customContextMenuRequested, this, &MainWindow::showPacketContextMenu);
    connect(packetTable_, &QTableWidget::cellDoubleClicked, this, &MainWindow::onPacketDoubleClicked);

    auto *packetContainer = new QWidget(this);
    auto *packetLayout = new QVBoxLayout(packetContainer);
    packetLayout->setContentsMargins(0, 0, 0, 0);
    packetLayout->setSpacing(0);
    emptyHint_ = new QLabel(t("table.empty"), packetContainer);
    emptyHint_->setObjectName(QStringLiteral("emptyHint"));
    emptyHint_->setWordWrap(true);
    emptyHint_->setContentsMargins(12, 10, 12, 10);
    packetLayout->addWidget(emptyHint_);
    packetLayout->addWidget(packetTable_, 1);
    packetDock_->setWidget(packetContainer);
    addDockWidget(Qt::LeftDockWidgetArea, packetDock_);

    requestDock_ = new QDockWidget(t("inspector.requestTitle"), this);
    requestDock_->setObjectName(QStringLiteral("request_dock"));
    requestEditor_ = new PacketEditor(this);
    requestEditor_->setPlaceholderText(t("inspector.requestPlaceholder"));
    requestDock_->setWidget(requestEditor_);
    addDockWidget(Qt::RightDockWidgetArea, requestDock_);

    responseDock_ = new QDockWidget(t("inspector.responseTitle"), this);
    responseDock_->setObjectName(QStringLiteral("response_dock"));
    responseEditor_ = new PacketEditor(this);
    responseEditor_->setPlaceholderText(t("inspector.responsePlaceholder"));
    responseDock_->setWidget(responseEditor_);
    addDockWidget(Qt::RightDockWidgetArea, responseDock_);

    tabifyDockWidget(requestDock_, responseDock_);
    requestDock_->raise();
    resizeDocks({packetDock_}, {500}, Qt::Horizontal);
    resizeDocks({requestDock_, responseDock_}, {420, 420}, Qt::Horizontal);
}

void MainWindow::createStatusBar() {
    statusLabel_ = new QLabel(t("status.ready"), this);
    statusLabel_->setObjectName(QStringLiteral("statusText"));
    proxyStatusLabel_ = new QLabel(t("status.proxyOffline"), this);
    proxyStatusLabel_->setObjectName(QStringLiteral("pill"));
    proxyStatusLabel_->setProperty("state", "offline");
    packetCountLabel_ = new QLabel(t("status.packetCount").arg(0), this);
    packetCountLabel_->setObjectName(QStringLiteral("statusMeta"));
    themeLabel_ = new QLabel(QStringLiteral("🌓"), this);
    themeLabel_->setObjectName(QStringLiteral("statusMeta"));
    auto *languageLabel = new QLabel(Translator::instance().code().toUpper(), this);
    languageLabel->setObjectName(QStringLiteral("statusMeta"));

    statusBar()->setSizeGripEnabled(false);
    statusBar()->addWidget(statusLabel_);
    statusBar()->addPermanentWidget(languageLabel);
    statusBar()->addPermanentWidget(themeLabel_);
    statusBar()->addPermanentWidget(proxyStatusLabel_);
    statusBar()->addPermanentWidget(packetCountLabel_);
    statusBar()->addPermanentWidget(new QSizeGrip(this));
}

QWidget *MainWindow::createDashboard() {
    auto *page = new QWidget(this);
    page->setObjectName(QStringLiteral("dashboard"));

    auto *layout = new QVBoxLayout(page);
    layout->setContentsMargins(24, 20, 24, 20);
    layout->setSpacing(12);

    auto *eyebrow = new QLabel(t("dashboard.eyebrow"), page);
    eyebrow->setObjectName(QStringLiteral("eyebrow"));
    auto *title = new QLabel(t("dashboard.title"), page);
    title->setObjectName(QStringLiteral("pageTitle"));
    auto *subtitle = new QLabel(t("dashboard.subtitle"), page);
    subtitle->setObjectName(QStringLiteral("muted"));
    subtitle->setWordWrap(true);

    layout->addWidget(eyebrow);
    layout->addWidget(title);
    layout->addWidget(subtitle);

    auto *actions = new QHBoxLayout();
    actions->setSpacing(8);
    proxyPill_ = new QLabel(t("dashboard.proxy.offline"), page);
    proxyPill_->setObjectName(QStringLiteral("pill"));
    proxyPill_->setProperty("state", "offline");
    primaryStartButton_ = new QPushButton(t("dashboard.startProxy"), page);
    primaryStartButton_->setObjectName(QStringLiteral("primaryButton"));
    primaryStartButton_->setCursor(Qt::PointingHandCursor);
    connect(primaryStartButton_, &QPushButton::clicked, this, &MainWindow::startProxy);
    actions->addWidget(proxyPill_);
    actions->addStretch(1);
    actions->addWidget(primaryStartButton_);
    layout->addLayout(actions);

    auto *grid = new QGridLayout();
    grid->setHorizontalSpacing(12);
    grid->setVerticalSpacing(12);
    grid->setColumnStretch(0, 1);
    grid->setColumnStretch(1, 1);

    auto *proxyCard = createMetricCard(
        t("dashboard.card.proxy"), t("dashboard.card.proxy.value"),
        t("dashboard.card.proxy.detail"), &proxyCardValue_, &proxyCardDetail_);
    grid->addWidget(proxyCard, 0, 0, 1, 2);
    auto *sessionCard = createMetricCard(
        t("dashboard.card.sessions"), t("dashboard.card.sessions.value"),
        t("dashboard.card.sessions.detail"), &sessionCardValue_);
    grid->addWidget(sessionCard, 1, 0);
    auto *packetCard = createMetricCard(
        t("dashboard.card.packets"), t("dashboard.card.packets.value"),
        t("dashboard.card.packets.detail"), &packetCardValue_);
    grid->addWidget(packetCard, 1, 1);

    auto *quickPanel = new QFrame(page);
    quickPanel->setObjectName(QStringLiteral("panel"));
    auto *quickLayout = new QVBoxLayout(quickPanel);
    quickLayout->setContentsMargins(16, 14, 16, 14);
    quickLayout->setSpacing(8);
    auto *quickTitle = new QLabel(t("dashboard.quick.title"), quickPanel);
    quickTitle->setObjectName(QStringLiteral("sectionTitle"));
    auto *quickBody = new QLabel(t("dashboard.quick.body"), quickPanel);
    quickBody->setObjectName(QStringLiteral("muted"));
    quickBody->setWordWrap(true);
    auto *quickButton = new QPushButton(t("dashboard.quick.button"), quickPanel);
    quickButton->setObjectName(QStringLiteral("ghostButton"));
    quickButton->setCursor(Qt::PointingHandCursor);
    connect(quickButton, &QPushButton::clicked, this, &MainWindow::startProxy);
    auto *quickActions = new QHBoxLayout();
    quickActions->addStretch(1);
    quickActions->addWidget(quickButton);
    quickLayout->addWidget(quickTitle);
    quickLayout->addWidget(quickBody);
    quickLayout->addLayout(quickActions);
    grid->addWidget(quickPanel, 2, 0, 1, 2);

    layout->addLayout(grid);
    layout->addStretch(1);

    auto *footer = new QLabel(t("dashboard.footer"), page);
    footer->setObjectName(QStringLiteral("caption"));
    layout->addWidget(footer);

    return page;
}

QFrame *MainWindow::createMetricCard(const QString &title, const QString &value,
                                     const QString &detail, QLabel **valueLabel,
                                     QLabel **detailLabel) {
    auto *card = new QFrame(this);
    card->setObjectName(QStringLiteral("metricCard"));
    card->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Preferred);
    card->setMinimumHeight(92);

    auto *layout = new QVBoxLayout(card);
    layout->setContentsMargins(16, 14, 16, 14);
    layout->setSpacing(4);

    auto *titleLabel = new QLabel(title, card);
    titleLabel->setObjectName(QStringLiteral("metricLabel"));
    auto *valueWidget = new QLabel(value, card);
    valueWidget->setObjectName(QStringLiteral("metricValue"));
    auto *detailWidget = new QLabel(detail, card);
    detailWidget->setObjectName(QStringLiteral("metricDetail"));
    detailWidget->setWordWrap(true);

    layout->addWidget(titleLabel);
    layout->addWidget(valueWidget);
    layout->addWidget(detailWidget);
    *valueLabel = valueWidget;
    if (detailLabel) {
        *detailLabel = detailWidget;
    }
    return card;
}

QString MainWindow::findProxyBinary() const {
    QStringList candidates;
    const QString environment = qEnvironmentVariable("NETTOOL_BIN");
    if (!environment.isEmpty()) {
        candidates << environment;
    }
    const QString appDir = QCoreApplication::applicationDirPath();
    candidates << appDir + QStringLiteral("/nettoold");
    candidates << appDir + QStringLiteral("/nettoold.exe");
    candidates << appDir + QStringLiteral("/../../target/release/nettoold");
    candidates << appDir + QStringLiteral("/../../target/debug/nettoold");
    candidates << appDir + QStringLiteral("/../../target/release/nettoold.exe");
    candidates << appDir + QStringLiteral("/../../target/debug/nettoold.exe");
    const QString fromPath = QStandardPaths::findExecutable(QStringLiteral("nettoold"));
    if (!fromPath.isEmpty()) {
        candidates << fromPath;
    }
    for (const QString &candidate : candidates) {
        const QFileInfo info(candidate);
        if (info.exists() && info.isFile() && info.isExecutable()) {
            return info.absoluteFilePath();
        }
    }
    return QString();
}

void MainWindow::setProxyRunning(bool running) {
    proxyRunning_ = running;
    if (startAction_) {
        startAction_->setEnabled(!running);
    }
    if (stopAction_) {
        stopAction_->setEnabled(running);
    }
    if (primaryStartButton_) {
        primaryStartButton_->setEnabled(!running);
    }

    const QString state = running ? QStringLiteral("online") : QStringLiteral("offline");
    for (QLabel *label : {proxyPill_, proxyStatusLabel_}) {
        if (!label) {
            continue;
        }
        label->setText(running ? t("dashboard.proxy.online") : t("dashboard.proxy.offline"));
        label->setProperty("state", state);
        label->style()->unpolish(label);
        label->style()->polish(label);
    }
    if (proxyCardValue_) {
        proxyCardValue_->setText(running ? t("dashboard.card.proxy.running")
                                        : t("dashboard.card.proxy.value"));
    }
    statusLabel_->setText(running ? t("status.proxyRunning") : t("status.proxyStopped"));
    updateWebDashboard();
}

void MainWindow::configureWebDashboard() {
    if (!webDashboard_) {
        return;
    }
    if (webDashboard_->isRunning()) {
        webDashboard_->stop();
    }
    QSettings settings;
    if (!settings.value(QStringLiteral("web/enabled"), true).toBool()) {
        return;
    }
    const int port = settings.value(QStringLiteral("web/port"), 2679).toInt();
    webDashboard_->start(static_cast<quint16>(port));
}

void MainWindow::updateWebDashboard() {
    if (!webDashboard_ || !webDashboard_->isRunning() || !packetTable_) {
        return;
    }
    QStringList rows;
    const int first = qMax(0, packetTable_->rowCount() - 100);
    for (int row = first; row < packetTable_->rowCount(); ++row) {
        QStringList cells;
        for (int column = 0; column < packetTable_->columnCount(); ++column) {
            const QTableWidgetItem *item = packetTable_->item(row, column);
            cells << (item ? item->text() : QString());
        }
        rows << cells.join(QLatin1Char('\t'));
    }
    webDashboard_->setSnapshot(proxyRunning_, 0, packetCount_, rows);
}

void MainWindow::addPacketLine(const QString &line) {
    static const QRegularExpression pattern(
        QStringLiteral("^#(\\d+) (C->S|S->C) (\\d+)B (\\S+)(?: (.*))?$"));
    const QRegularExpressionMatch match = pattern.match(line);
    if (!match.hasMatch()) {
        return;
    }

    const int row = packetTable_->rowCount();
    packetTable_->insertRow(row);
    const QString direction = match.captured(2) == QStringLiteral("C->S")
                                  ? QStringLiteral("C→S")
                                  : QStringLiteral("S→C");
    QString summary = match.captured(4);
    if (!match.captured(5).isEmpty()) {
        summary += QStringLiteral(" · ") + match.captured(5);
    }

    const QStringList values = {
        match.captured(1),
        QTime::currentTime().toString(QStringLiteral("HH:mm:ss")),
        direction,
        match.captured(3) + QStringLiteral(" B"),
        t("table.forwarded"),
        summary,
    };
    for (int column = 0; column < values.size(); ++column) {
        auto *item = new QTableWidgetItem(values.at(column));
        item->setTextAlignment(Qt::AlignLeft | Qt::AlignVCenter);
        packetTable_->setItem(row, column, item);
    }
    if (QTableWidgetItem *summaryItem = packetTable_->item(row, 5)) {
        summaryItem->setData(Qt::UserRole, line.toUtf8());
    }
    packetTable_->scrollToBottom();
    if (emptyHint_) {
        emptyHint_->hide();
    }
    packetCount_ += 1;
    if (packetCardValue_) {
        packetCardValue_->setText(QString::number(packetCount_));
    }
    packetCountLabel_->setText(t("status.packetCount").arg(packetCount_));
    statusLabel_->setText(t("status.packetCount").arg(packetCount_));
    updateWebDashboard();
}

void MainWindow::startProxy() {
    if (proxyRunning_) {
        return;
    }
    const QString binary = findProxyBinary();
    if (binary.isEmpty()) {
        QMessageBox::warning(this, t("menu.startProxy"), t("status.proxyBinaryMissing"));
        notifications_->show(t("status.proxyBinaryMissing"), NotificationCenter::Level::Error);
        return;
    }

    QSettings settings;
    const int listenPort = settings.value(QStringLiteral("proxy/listenPort"), 25565).toInt();
    const QString targetHost = settings.value(QStringLiteral("proxy/targetHost"), QStringLiteral("localhost")).toString();
    const int targetPort = settings.value(QStringLiteral("proxy/targetPort"), 25566).toInt();
    QString protocol = settings.value(QStringLiteral("proxy/protocol"), QStringLiteral("auto")).toString();
    if (protocol == QStringLiteral("https")) {
        protocol = QStringLiteral("tls");
    } else if (protocol == QStringLiteral("minecraft")) {
        protocol = QStringLiteral("mc");
    }

    proxyProcess_ = new QProcess(this);
    connect(proxyProcess_, &QProcess::readyReadStandardOutput, this, [this] {
        while (proxyProcess_ && proxyProcess_->canReadLine()) {
            const QString line = QString::fromUtf8(proxyProcess_->readLine()).trimmed();
            if (!line.isEmpty()) {
                addPacketLine(line);
            }
        }
    });
    connect(proxyProcess_, QOverload<int, QProcess::ExitStatus>::of(&QProcess::finished),
            this, [this](int, QProcess::ExitStatus) {
                if (proxyRunning_) {
                    setProxyRunning(false);
                }
                if (proxyProcess_) {
                    proxyProcess_->deleteLater();
                    proxyProcess_ = nullptr;
                }
            });

    QStringList arguments = {
        QStringLiteral("proxy"),
        QStringLiteral("--listen"), QStringLiteral("127.0.0.1:%1").arg(listenPort),
        QStringLiteral("--target"), QStringLiteral("%1:%2").arg(targetHost).arg(targetPort),
        QStringLiteral("--protocol"), protocol,
    };
    const QStringList savedRules = settings.value(QStringLiteral("rules/items")).toStringList();
    for (const QString &line : savedRules) {
        const QStringList parts = line.split(QLatin1Char('\t'));
        if (parts.size() < 2) {
            continue;
        }
        QString expression = parts.at(1).trimmed();
        const QString action = parts.size() > 2 ? parts.at(2).trimmed() : QString();
        if (expression.isEmpty()) {
            continue;
        }
        if (!expression.startsWith(QStringLiteral("drop ")) &&
            !expression.startsWith(QStringLiteral("allow ")) && !action.isEmpty()) {
            expression = action + QLatin1Char(' ') + expression;
        }
        arguments << QStringLiteral("--rule") << expression;
    }
    proxyProcess_->start(binary, arguments);
    if (!proxyProcess_->waitForStarted(3000)) {
        const QString error = t("status.proxyStartFailed").arg(proxyProcess_->errorString());
        QMessageBox::warning(this, t("menu.startProxy"), error);
        notifications_->show(error, NotificationCenter::Level::Error);
        proxyProcess_->deleteLater();
        proxyProcess_ = nullptr;
        return;
    }
    setProxyRunning(true);
    notifications_->show(t("notification.proxyStarted"), NotificationCenter::Level::Success);
}

void MainWindow::stopProxy() {
    if (proxyProcess_) {
        if (proxyProcess_->state() != QProcess::NotRunning) {
            proxyProcess_->terminate();
            if (!proxyProcess_->waitForFinished(1500)) {
                proxyProcess_->kill();
                proxyProcess_->waitForFinished(1000);
            }
        }
        if (proxyProcess_) {
            proxyProcess_->deleteLater();
            proxyProcess_ = nullptr;
        }
    }
    setProxyRunning(false);
    notifications_->show(t("notification.proxyStopped"), NotificationCenter::Level::Info);
}

void MainWindow::clearPackets() {
    packetTable_->setRowCount(0);
    packetCount_ = 0;
    packetCountLabel_->setText(t("status.packetCount").arg(0));
    if (packetCardValue_) {
        packetCardValue_->setText(QStringLiteral("0"));
    }
    if (emptyHint_) {
        emptyHint_->show();
    }
    requestEditor_->clear();
    responseEditor_->clear();
    statusLabel_->setText(t("status.cleared"));
    updateWebDashboard();
}

void MainWindow::exportPackets() {
    if (packetTable_->rowCount() == 0) {
        QMessageBox::information(this, t("dialog.export.title"), t("dialog.export.empty"));
        return;
    }

    SmartExportDialog dialog(this);
    if (dialog.exec() != QDialog::Accepted) {
        return;
    }
    const QString format = dialog.format();
    const bool visibleOnly = dialog.visibleOnly();
    const QString defaultName = QStringLiteral("packets.") + format;
    QString filter = t("dialog.export.filter");
    if (format == QStringLiteral("json")) {
        filter = t("dialog.export.filterJson");
    } else if (format == QStringLiteral("tsv")) {
        filter = t("dialog.export.filterTsv");
    }
    const QString path = QFileDialog::getSaveFileName(
        this, t("dialog.export.title"), defaultName, filter);
    if (path.isEmpty()) {
        return;
    }

    QFile file(path);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Text)) {
        QMessageBox::warning(this, t("dialog.export.title"), t("dialog.export.error"));
        return;
    }

    const auto cellText = [this](int row, int column) {
        const QTableWidgetItem *item = packetTable_->item(row, column);
        return item ? item->text() : QString();
    };
    const auto headerText = [this](int column) {
        const QTableWidgetItem *item = packetTable_->horizontalHeaderItem(column);
        return item ? item->text() : QString::number(column);
    };
    const auto escaped = [](QString text) {
        text.replace(QLatin1Char('"'), QStringLiteral("\"\""));
        return text;
    };

    if (format == QStringLiteral("json")) {
        QJsonArray rows;
        for (int row = 0; row < packetTable_->rowCount(); ++row) {
            if (visibleOnly && packetTable_->isRowHidden(row)) {
                continue;
            }
            QJsonObject object;
            for (int column = 0; column < packetTable_->columnCount(); ++column) {
                object.insert(headerText(column), cellText(row, column));
            }
            rows.append(object);
        }
        const QByteArray json = QJsonDocument(rows).toJson(QJsonDocument::Indented);
        if (file.write(json) < 0) {
            QMessageBox::warning(this, t("dialog.export.title"), t("dialog.export.error"));
            return;
        }
    } else {
        const QChar delimiter = format == QStringLiteral("tsv") ? QLatin1Char('\t') : QLatin1Char(',');
        QTextStream stream(&file);
        stream.setEncoding(QStringConverter::Utf8);
        for (int column = 0; column < packetTable_->columnCount(); ++column) {
            if (column > 0) {
                stream << delimiter;
            }
            stream << '"' << escaped(headerText(column)) << '"';
        }
        stream << '\n';
        for (int row = 0; row < packetTable_->rowCount(); ++row) {
            if (visibleOnly && packetTable_->isRowHidden(row)) {
                continue;
            }
            for (int column = 0; column < packetTable_->columnCount(); ++column) {
                if (column > 0) {
                    stream << delimiter;
                }
                stream << '"' << escaped(cellText(row, column)) << '"';
            }
            stream << '\n';
        }
    }

    statusLabel_->setText(t("status.exported"));
    notifications_->show(t("notification.exportSaved"), NotificationCenter::Level::Success);
}

void MainWindow::showProxyConfig() {
    QSettings settings;
    ProxySettings current;
    current.listenPort = settings.value(QStringLiteral("proxy/listenPort"), 25565).toInt();
    current.targetHost = settings.value(QStringLiteral("proxy/targetHost"), QStringLiteral("localhost")).toString();
    current.targetPort = settings.value(QStringLiteral("proxy/targetPort"), 25566).toInt();
    current.protocol = settings.value(QStringLiteral("proxy/protocol"), QStringLiteral("auto")).toString();
    current.mode = settings.value(QStringLiteral("proxy/mode"), QStringLiteral("intercept")).toString();

    ProxyConfigDialog dialog(current, this);
    if (dialog.exec() != QDialog::Accepted) {
        return;
    }
    const ProxySettings next = dialog.settings();
    settings.setValue(QStringLiteral("proxy/listenPort"), next.listenPort);
    settings.setValue(QStringLiteral("proxy/targetHost"), next.targetHost);
    settings.setValue(QStringLiteral("proxy/targetPort"), next.targetPort);
    settings.setValue(QStringLiteral("proxy/protocol"), next.protocol);
    settings.setValue(QStringLiteral("proxy/mode"), next.mode);

    if (proxyCardDetail_) {
        proxyCardDetail_->setText(
            QStringLiteral("%1:%2  →  %3:%4")
                .arg(QStringLiteral("127.0.0.1"))
                .arg(next.listenPort)
                .arg(next.targetHost)
                .arg(next.targetPort));
    }
    statusLabel_->setText(t("dialog.settings.saved"));
    notifications_->show(t("notification.settingsSaved"), NotificationCenter::Level::Success);
}

void MainWindow::showRules() {
    RuleManagerDialog dialog(this);
    dialog.exec();
}

void MainWindow::showRepeater() {
    RepeaterDialog dialog(this);
    dialog.exec();
}

void MainWindow::showIntruder() {
    IntruderDialog dialog(this);
    dialog.exec();
}

void MainWindow::showScanner() {
    ScannerDialog dialog(this);
    dialog.exec();
}

void MainWindow::showSettings() {
    SettingsDialog dialog(this);
    if (dialog.exec() != QDialog::Accepted) {
        return;
    }
    setTheme(dialog.selectedTheme());
    QFont font = QApplication::font();
    font.setPointSize(dialog.selectedFontSize());
    QApplication::setFont(font);
    applyControlsSide(dialog.controlsSide());
    statusLabel_->setText(t("dialog.settings.saved"));
    notifications_->show(t("notification.settingsSaved"), NotificationCenter::Level::Success);
    configureWebDashboard();
    updateWebDashboard();
}

void MainWindow::showTutorial() {
    QMessageBox::information(this, t("dialog.tutorial.title"), t("dialog.tutorial.body"));
}

void MainWindow::showAbout() {
    QMessageBox::about(this, t("dialog.about.title"), t("dialog.about.body"));
}

void MainWindow::applyFilter(const QString &text) {
    const QString needle = text.trimmed().toLower();
    for (int row = 0; row < packetTable_->rowCount(); ++row) {
        bool matches = needle.isEmpty();
        if (!matches) {
            for (int column = 0; column < packetTable_->columnCount(); ++column) {
                const QTableWidgetItem *item = packetTable_->item(row, column);
                if (item && item->text().toLower().contains(needle)) {
                    matches = true;
                    break;
                }
            }
        }
        packetTable_->setRowHidden(row, !matches);
    }
}

void MainWindow::onThemeComboChanged(int index) {
    switch (index) {
    case 0:
        setTheme(QStringLiteral("light"));
        break;
    case 1:
        setTheme(QStringLiteral("dark"));
        break;
    default:
        setTheme(QStringLiteral("system"));
        break;
    }
}

void MainWindow::setTheme(const QString &theme) {
    theme_ = theme;
    applyTheme(theme_);

    if (themeCombo_) {
        const QSignalBlocker blocker(themeCombo_);
        if (theme_ == QStringLiteral("light")) {
            themeCombo_->setCurrentIndex(0);
        } else if (theme_ == QStringLiteral("dark")) {
            themeCombo_->setCurrentIndex(1);
        } else {
            themeCombo_->setCurrentIndex(2);
        }
    }
}

void MainWindow::applyTheme(const QString &theme) {
    if (theme == QStringLiteral("dark")) {
        dark_ = true;
    } else if (theme == QStringLiteral("light")) {
        dark_ = false;
    } else {
        dark_ = QGuiApplication::styleHints()->colorScheme() == Qt::ColorScheme::Dark;
    }
    qApp->setStyleSheet(dark_ ? darkStyle() : lightStyle());
    if (themeLabel_) {
        themeLabel_->setText(dark_ ? QStringLiteral("🌙") : QStringLiteral("☀️"));
    }
}

void MainWindow::applyZoom() {
    QFont font = QApplication::font();
    const int pointSize = qBound(6, static_cast<int>(9.0 * zoom_) + 1, 20);
    font.setPointSize(pointSize);
    QApplication::setFont(font);
}

void MainWindow::zoomIn() {
    zoom_ = qMin(2.0, zoom_ * 1.05);
    applyZoom();
    statusBar()->showMessage(t("status.zoomed").arg(static_cast<int>(zoom_ * 100)), 1500);
}

void MainWindow::zoomOut() {
    zoom_ = qMax(0.5, zoom_ * 0.95);
    applyZoom();
    statusBar()->showMessage(t("status.zoomed").arg(static_cast<int>(zoom_ * 100)), 1500);
}

void MainWindow::resetZoom() {
    zoom_ = 1.0;
    applyZoom();
    statusBar()->showMessage(t("status.zoomReset"), 1500);
}

void MainWindow::wheelEvent(QWheelEvent *event) {
    if (event->modifiers() == Qt::ControlModifier) {
        if (event->angleDelta().y() > 0) {
            zoomIn();
        } else {
            zoomOut();
        }
        event->accept();
        return;
    }
    QMainWindow::wheelEvent(event);
}

void MainWindow::showPacketContextMenu(const QPoint &position) {
    const int row = packetTable_->rowAt(position.y());
    if (row < 0) {
        return;
    }
    QMenu menu(this);
    QAction *copyAction = menu.addAction(t("context.copySummary"));
    QAction *inspectAction = menu.addAction(t("context.inspect"));
    QAction *selected = menu.exec(packetTable_->viewport()->mapToGlobal(position));
    if (selected == copyAction) {
        const QTableWidgetItem *summary = packetTable_->item(row, 5);
        QApplication::clipboard()->setText(summary ? summary->text() : QString());
    } else if (selected == inspectAction) {
        onPacketDoubleClicked(row, 0);
    }
}

void MainWindow::onPacketDoubleClicked(int row, int column) {
    Q_UNUSED(column);
    const QTableWidgetItem *summary = packetTable_->item(row, 5);
    QByteArray raw = summary ? summary->data(Qt::UserRole).toByteArray() : QByteArray();
    if (raw.isEmpty()) {
        raw = t("inspector.packet")
                  .arg(packetTable_->item(row, 2) ? packetTable_->item(row, 2)->text() : QString(),
                       packetTable_->item(row, 3) ? packetTable_->item(row, 3)->text() : QString(),
                       summary ? summary->text() : QString())
                  .toUtf8();
    }
    requestEditor_->setData(raw);
    responseEditor_->clear();
    requestDock_->raise();
}
