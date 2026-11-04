#include "MainWindow.h"

#include <QAbstractItemView>
#include <QAction>
#include <QApplication>
#include <QClipboard>
#include <QComboBox>
#include <QDockWidget>
#include <QFile>
#include <QFileDialog>
#include <QFont>
#include <QFontDatabase>
#include <QGuiApplication>
#include <QHeaderView>
#include <QKeySequence>
#include <QLabel>
#include <QLineEdit>
#include <QMenu>
#include <QMenuBar>
#include <QMessageBox>
#include <QPlainTextEdit>
#include <QPoint>
#include <QSignalBlocker>
#include <QStatusBar>
#include <QStringConverter>
#include <QStyleHints>
#include <QTableWidget>
#include <QTableWidgetItem>
#include <QTextStream>
#include <QToolBar>
#include <QWheelEvent>
#include <QWidget>

namespace {

QString darkStyle() {
    return QStringLiteral(R"STYLE(
QMainWindow, QDialog { background-color: #1e1e1e; }
QWidget { background-color: #1e1e1e; color: #d4d4d4; }
QDockWidget::title { background-color: #2d2d2d; color: #ffffff; padding: 5px; }
QTableWidget { background-color: #252525; color: #d4d4d4; gridline-color: #3c3c3c; alternate-background-color: #2d2d2d; }
QTableWidget::item:selected { background-color: #264f78; }
QHeaderView::section { background-color: #2d2d2d; color: #ffffff; padding: 5px; border: none; }
QPlainTextEdit, QTextEdit { background-color: #1e1e1e; color: #d4d4d4; border: 1px solid #3c3c3c; }
QLineEdit { background-color: #3c3c3c; color: #ffffff; border: 1px solid #555; padding: 4px; border-radius: 3px; }
QLabel { color: #d4d4d4; }
QMenuBar { background-color: #2d2d2d; color: #ffffff; }
QMenuBar::item:selected { background-color: #3c3c3c; }
QMenu { background-color: #2d2d2d; color: #ffffff; border: 1px solid #3c3c3c; }
QMenu::item:selected { background-color: #3c3c3c; }
QComboBox { background-color: #3c3c3c; color: #ffffff; border: 1px solid #555; padding: 4px; border-radius: 3px; }
QComboBox QAbstractItemView { background-color: #2d2d2d; color: #ffffff; selection-background-color: #3c3c3c; }
QToolBar { background-color: #2d2d2d; border: none; spacing: 3px; }
QStatusBar { background-color: #2d2d2d; color: #ffffff; }
QScrollBar:vertical { background-color: #1e1e1e; width: 12px; margin: 0; }
QScrollBar::handle:vertical { background-color: #3c3c3c; border-radius: 6px; min-height: 20px; }
)STYLE");
}

QString lightStyle() {
    return QStringLiteral(R"STYLE(
QMainWindow, QDialog { background-color: #f0f0f0; }
QWidget { background-color: #f0f0f0; color: #1e1e1e; }
QDockWidget::title { background-color: #e0e0e0; color: #1e1e1e; padding: 5px; }
QTableWidget { background-color: #ffffff; color: #1e1e1e; gridline-color: #d0d0d0; alternate-background-color: #f7f7f7; }
QTableWidget::item:selected { background-color: #cce8ff; color: #1e1e1e; }
QHeaderView::section { background-color: #e0e0e0; color: #1e1e1e; padding: 5px; border: none; }
QPlainTextEdit, QTextEdit { background-color: #ffffff; color: #1e1e1e; border: 1px solid #c0c0c0; }
QLineEdit { background-color: #ffffff; color: #1e1e1e; border: 1px solid #c0c0c0; padding: 4px; border-radius: 3px; }
QLabel { color: #1e1e1e; }
QMenuBar { background-color: #f0f0f0; color: #1e1e1e; }
QMenuBar::item:selected { background-color: #e0e0e0; }
QMenu { background-color: #ffffff; color: #1e1e1e; border: 1px solid #c0c0c0; }
QMenu::item:selected { background-color: #e0e0e0; }
QComboBox { background-color: #ffffff; color: #1e1e1e; border: 1px solid #c0c0c0; padding: 4px; border-radius: 3px; }
QComboBox QAbstractItemView { background-color: #ffffff; color: #1e1e1e; selection-background-color: #cce8ff; }
QToolBar { background-color: #f0f0f0; border: none; spacing: 3px; }
QStatusBar { background-color: #f0f0f0; color: #1e1e1e; }
QScrollBar:vertical { background-color: #f0f0f0; width: 12px; margin: 0; }
QScrollBar::handle:vertical { background-color: #c0c0c0; border-radius: 6px; min-height: 20px; }
)STYLE");
}

QPlainTextEdit *createPacketViewer(QWidget *parent) {
    auto *viewer = new QPlainTextEdit(parent);
    QFont font = QFontDatabase::systemFont(QFontDatabase::FixedFont);
    font.setPointSize(11);
    viewer->setFont(font);
    viewer->setPlaceholderText(QStringLiteral("数据包内容"));
    return viewer;
}

} // namespace

MainWindow::MainWindow(QWidget *parent) : QMainWindow(parent) {
    setWindowTitle(QStringLiteral("SilentStudio© NetTool ζ - 终极协议安全测试平台"));
    setGeometry(50, 50, 1400, 900);
    setDockOptions(QMainWindow::AnimatedDocks | QMainWindow::AllowTabbedDocks);
    setCentralWidget(new QWidget(this));

    createMenus();
    createToolBar();
    createDocks();
    createStatusBar();
    applyTheme(theme_);
}

void MainWindow::createMenus() {
    QMenu *fileMenu = menuBar()->addMenu(QStringLiteral("文件"));
    QAction *exportAction = fileMenu->addAction(QStringLiteral("导出数据包"));
    exportAction->setShortcut(QKeySequence(QStringLiteral("Ctrl+E")));
    connect(exportAction, &QAction::triggered, this, &MainWindow::exportPackets);
    QAction *clearAction = fileMenu->addAction(QStringLiteral("清空历史"));
    clearAction->setShortcut(QKeySequence(QStringLiteral("Ctrl+L")));
    connect(clearAction, &QAction::triggered, this, &MainWindow::clearPackets);
    fileMenu->addSeparator();
    QAction *exitAction = fileMenu->addAction(QStringLiteral("退出"));
    exitAction->setShortcut(QKeySequence(QStringLiteral("Alt+F4")));
    connect(exitAction, &QAction::triggered, this, &QWidget::close);

    QMenu *proxyMenu = menuBar()->addMenu(QStringLiteral("代理"));
    startAction_ = proxyMenu->addAction(QStringLiteral("启动代理"));
    connect(startAction_, &QAction::triggered, this, &MainWindow::startProxy);
    stopAction_ = proxyMenu->addAction(QStringLiteral("停止代理"));
    stopAction_->setEnabled(false);
    connect(stopAction_, &QAction::triggered, this, &MainWindow::stopProxy);
    proxyMenu->addSeparator();
    QAction *configAction = proxyMenu->addAction(QStringLiteral("⚙️ 代理配置"));
    connect(configAction, &QAction::triggered, this, &MainWindow::showProxyConfig);
    QAction *rulesAction = proxyMenu->addAction(QStringLiteral("规则管理"));
    connect(rulesAction, &QAction::triggered, this, &MainWindow::showRules);

    QMenu *toolsMenu = menuBar()->addMenu(QStringLiteral("工具"));
    QAction *repeaterAction = toolsMenu->addAction(QStringLiteral("Repeater"));
    repeaterAction->setShortcut(QKeySequence(QStringLiteral("Ctrl+R")));
    connect(repeaterAction, &QAction::triggered, this, &MainWindow::showRepeater);
    QAction *intruderAction = toolsMenu->addAction(QStringLiteral("Intruder"));
    intruderAction->setShortcut(QKeySequence(QStringLiteral("Ctrl+T")));
    connect(intruderAction, &QAction::triggered, this, &MainWindow::showIntruder);
    QAction *scannerAction = toolsMenu->addAction(QStringLiteral("Scanner"));
    scannerAction->setShortcut(QKeySequence(QStringLiteral("Ctrl+S")));
    connect(scannerAction, &QAction::triggered, this, &MainWindow::showScanner);

    QMenu *viewMenu = menuBar()->addMenu(QStringLiteral("视图"));
    QMenu *themeMenu = viewMenu->addMenu(QStringLiteral("主题"));
    QAction *lightAction = themeMenu->addAction(QStringLiteral("浅色"));
    connect(lightAction, &QAction::triggered, this, [this] { setTheme(QStringLiteral("light")); });
    QAction *darkAction = themeMenu->addAction(QStringLiteral("深色"));
    connect(darkAction, &QAction::triggered, this, [this] { setTheme(QStringLiteral("dark")); });
    QAction *systemAction = themeMenu->addAction(QStringLiteral("跟随系统"));
    connect(systemAction, &QAction::triggered, this, [this] { setTheme(QStringLiteral("system")); });
    viewMenu->addSeparator();
    QAction *resetZoomAction = viewMenu->addAction(QStringLiteral("重置缩放"));
    resetZoomAction->setShortcut(QKeySequence(QStringLiteral("Ctrl+0")));
    connect(resetZoomAction, &QAction::triggered, this, &MainWindow::resetZoom);
    QAction *zoomInAction = viewMenu->addAction(QStringLiteral("放大"));
    zoomInAction->setShortcut(QKeySequence(QStringLiteral("Ctrl++")));
    connect(zoomInAction, &QAction::triggered, this, &MainWindow::zoomIn);
    QAction *zoomOutAction = viewMenu->addAction(QStringLiteral("缩小"));
    zoomOutAction->setShortcut(QKeySequence(QStringLiteral("Ctrl+-")));
    connect(zoomOutAction, &QAction::triggered, this, &MainWindow::zoomOut);

    QMenu *settingsMenu = menuBar()->addMenu(QStringLiteral("设置"));
    QAction *preferencesAction = settingsMenu->addAction(QStringLiteral("偏好设置"));
    preferencesAction->setShortcut(QKeySequence(QStringLiteral("Ctrl+,")));
    connect(preferencesAction, &QAction::triggered, this, &MainWindow::showSettings);

    QMenu *helpMenu = menuBar()->addMenu(QStringLiteral("帮助"));
    QAction *tutorialAction = helpMenu->addAction(QStringLiteral("📖 教程"));
    connect(tutorialAction, &QAction::triggered, this, &MainWindow::showTutorial);
    helpMenu->addSeparator();
    QAction *aboutAction = helpMenu->addAction(QStringLiteral("关于"));
    connect(aboutAction, &QAction::triggered, this, &MainWindow::showAbout);
}

void MainWindow::createToolBar() {
    QToolBar *toolbar = addToolBar(QStringLiteral("主要"));
    toolbar->setObjectName(QStringLiteral("main_toolbar"));
    toolbar->setMovable(false);

    toolbar->addAction(startAction_);
    toolbar->addAction(stopAction_);
    toolbar->addSeparator();

    QAction *repeaterAction = toolbar->addAction(QStringLiteral("🔄 Repeater"));
    connect(repeaterAction, &QAction::triggered, this, &MainWindow::showRepeater);
    QAction *intruderAction = toolbar->addAction(QStringLiteral("💣 Intruder"));
    connect(intruderAction, &QAction::triggered, this, &MainWindow::showIntruder);
    QAction *scannerAction = toolbar->addAction(QStringLiteral("🔍 Scanner"));
    connect(scannerAction, &QAction::triggered, this, &MainWindow::showScanner);
    toolbar->addSeparator();

    QAction *clearAction = toolbar->addAction(QStringLiteral("🗑 清空"));
    connect(clearAction, &QAction::triggered, this, &MainWindow::clearPackets);
    toolbar->addSeparator();

    toolbar->addWidget(new QLabel(QStringLiteral("  过滤: "), this));
    filterInput_ = new QLineEdit(this);
    filterInput_->setPlaceholderText(QStringLiteral("过滤条件..."));
    filterInput_->setMinimumWidth(200);
    connect(filterInput_, &QLineEdit::textChanged, this, &MainWindow::applyFilter);
    toolbar->addWidget(filterInput_);
    toolbar->addSeparator();

    toolbar->addWidget(new QLabel(QStringLiteral("  主题: "), this));
    themeCombo_ = new QComboBox(this);
    themeCombo_->addItems({
        QStringLiteral("☀️ 浅色"),
        QStringLiteral("🌙 深色"),
        QStringLiteral("🖥️ 跟随系统"),
    });
    connect(themeCombo_, QOverload<int>::of(&QComboBox::currentIndexChanged), this, &MainWindow::onThemeComboChanged);
    toolbar->addWidget(themeCombo_);
}

void MainWindow::createDocks() {
    packetDock_ = new QDockWidget(QStringLiteral("数据包历史"), this);
    packetDock_->setObjectName(QStringLiteral("packet_dock"));
    packetTable_ = new QTableWidget(this);
    packetTable_->setColumnCount(6);
    packetTable_->setHorizontalHeaderLabels({
        QStringLiteral("#"),
        QStringLiteral("时间"),
        QStringLiteral("方向"),
        QStringLiteral("长度"),
        QStringLiteral("状态"),
        QStringLiteral("摘要"),
    });
    packetTable_->setAlternatingRowColors(true);
    packetTable_->setSelectionBehavior(QAbstractItemView::SelectRows);
    packetTable_->setSelectionMode(QAbstractItemView::SingleSelection);
    packetTable_->setEditTriggers(QAbstractItemView::NoEditTriggers);
    packetTable_->horizontalHeader()->setSectionResizeMode(5, QHeaderView::Stretch);
    packetTable_->setColumnWidth(0, 48);
    packetTable_->setColumnWidth(1, 138);
    packetTable_->setColumnWidth(2, 66);
    packetTable_->setColumnWidth(3, 66);
    packetTable_->setColumnWidth(4, 78);
    packetTable_->setContextMenuPolicy(Qt::CustomContextMenu);
    connect(packetTable_, &QTableWidget::customContextMenuRequested, this, &MainWindow::showPacketContextMenu);
    connect(packetTable_, &QTableWidget::cellDoubleClicked, this, &MainWindow::onPacketDoubleClicked);
    packetDock_->setWidget(packetTable_);
    addDockWidget(Qt::LeftDockWidgetArea, packetDock_);

    requestDock_ = new QDockWidget(QStringLiteral("请求编辑器"), this);
    requestDock_->setObjectName(QStringLiteral("request_dock"));
    requestEditor_ = createPacketViewer(this);
    requestDock_->setWidget(requestEditor_);
    addDockWidget(Qt::RightDockWidgetArea, requestDock_);

    responseDock_ = new QDockWidget(QStringLiteral("响应编辑器"), this);
    responseDock_->setObjectName(QStringLiteral("response_dock"));
    responseEditor_ = createPacketViewer(this);
    responseDock_->setWidget(responseEditor_);
    addDockWidget(Qt::RightDockWidgetArea, responseDock_);

    tabifyDockWidget(requestDock_, responseDock_);
    requestDock_->raise();
    resizeDocks({packetDock_}, {520}, Qt::Horizontal);
    resizeDocks({requestDock_, responseDock_}, {470, 470}, Qt::Horizontal);
}

void MainWindow::createStatusBar() {
    statusLabel_ = new QLabel(QStringLiteral("就绪"), this);
    proxyStatusLabel_ = new QLabel(QStringLiteral("代理: 未启动"), this);
    packetCountLabel_ = new QLabel(QStringLiteral("数据包: 0"), this);
    themeLabel_ = new QLabel(QStringLiteral("🌓"), this);

    statusBar()->addWidget(statusLabel_);
    statusBar()->addPermanentWidget(themeLabel_);
    statusBar()->addPermanentWidget(proxyStatusLabel_);
    statusBar()->addPermanentWidget(packetCountLabel_);
}

void MainWindow::startProxy() {
    statusBar()->showMessage(
        QStringLiteral("代理后端尚未接入 Qt 客户端；下一步接 daemon start_proxy。"), 5000);
}

void MainWindow::stopProxy() {
    statusBar()->showMessage(
        QStringLiteral("代理后端尚未接入 Qt 客户端。"), 5000);
}

void MainWindow::clearPackets() {
    packetTable_->setRowCount(0);
    packetCount_ = 0;
    packetCountLabel_->setText(QStringLiteral("数据包: 0"));
    requestEditor_->clear();
    responseEditor_->clear();
    statusLabel_->setText(QStringLiteral("已清空数据包历史"));
}

void MainWindow::exportPackets() {
    if (packetTable_->rowCount() == 0) {
        QMessageBox::information(this, QStringLiteral("导出数据包"),
                                 QStringLiteral("当前没有数据包可导出。"));
        return;
    }

    const QString path = QFileDialog::getSaveFileName(
        this, QStringLiteral("导出数据包"), QStringLiteral("packets.csv"),
        QStringLiteral("CSV (*.csv);;所有文件 (*)"));
    if (path.isEmpty()) {
        return;
    }

    QFile file(path);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Text)) {
        QMessageBox::warning(this, QStringLiteral("导出失败"),
                             QStringLiteral("无法写入文件。"));
        return;
    }
    QTextStream stream(&file);
    stream.setEncoding(QStringConverter::Utf8);
    stream << "id,time,direction,length,status,summary\n";
    for (int row = 0; row < packetTable_->rowCount(); ++row) {
        for (int column = 0; column < packetTable_->columnCount(); ++column) {
            if (column > 0) {
                stream << ',';
            }
            const QTableWidgetItem *item = packetTable_->item(row, column);
            QString text = item ? item->text() : QString();
            text.replace('"', QStringLiteral("\"\""));
            stream << '"' << text << '"';
        }
        stream << '\n';
    }
    statusLabel_->setText(QStringLiteral("数据包已导出"));
}

void MainWindow::showProxyConfig() {
    QMessageBox::information(this, QStringLiteral("代理配置"),
                             QStringLiteral("代理配置对话框正在从 PySide6 1:1 移植。"));
}

void MainWindow::showRules() {
    QMessageBox::information(this, QStringLiteral("规则管理"),
                             QStringLiteral("规则管理对话框正在从 PySide6 1:1 移植。"));
}

void MainWindow::showRepeater() {
    QMessageBox::information(this, QStringLiteral("Repeater"),
                             QStringLiteral("Repeater 窗口正在从 PySide6 1:1 移植。"));
}

void MainWindow::showIntruder() {
    QMessageBox::information(this, QStringLiteral("Intruder"),
                             QStringLiteral("Intruder 窗口正在从 PySide6 1:1 移植。"));
}

void MainWindow::showScanner() {
    QMessageBox::information(this, QStringLiteral("Scanner"),
                             QStringLiteral("Scanner 窗口正在从 PySide6 1:1 移植。"));
}

void MainWindow::showSettings() {
    QMessageBox::information(this, QStringLiteral("偏好设置"),
                             QStringLiteral("设置对话框正在从 PySide6 1:1 移植。"));
}

void MainWindow::showTutorial() {
    QMessageBox::information(
        this, QStringLiteral("教程"),
        QStringLiteral("NetTool ζ 核心功能：\n"
                       "1. 代理：拦截和转发 Minecraft 协议数据包\n"
                       "2. Repeater：重放 HTTP 请求\n"
                       "3. Intruder：批量测试\n"
                       "4. Scanner：漏洞扫描\n"
                       "5. 规则引擎：过滤和修改数据包\n"
                       "6. Web 仪表盘：远程监控（端口 2679）"));
}

void MainWindow::showAbout() {
    QMessageBox::about(
        this, QStringLiteral("关于 NetTool ζ"),
        QStringLiteral("SilentStudio© NetTool ζ\n"
                       "终极协议安全测试平台\n\n"
                       "Qt 6 Widgets 客户端（PySide6 1:1 移植中）"));
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

    const QSignalBlocker blocker(themeCombo_);
    if (theme_ == QStringLiteral("light")) {
        themeCombo_->setCurrentIndex(0);
    } else if (theme_ == QStringLiteral("dark")) {
        themeCombo_->setCurrentIndex(1);
    } else {
        themeCombo_->setCurrentIndex(2);
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
    setStyleSheet(dark_ ? darkStyle() : lightStyle());
    if (themeLabel_) {
        themeLabel_->setText(dark_ ? QStringLiteral("🌙") : QStringLiteral("☀️"));
    }
}

void MainWindow::applyZoom() {
    QFont font = QApplication::font();
    const int pointSize = qBound(6, static_cast<int>(9.0 * zoom_), 20);
    font.setPointSize(pointSize);
    QApplication::setFont(font);
}

void MainWindow::zoomIn() {
    zoom_ = qMin(2.0, zoom_ * 1.05);
    applyZoom();
    statusBar()->showMessage(QStringLiteral("缩放: %1%").arg(static_cast<int>(zoom_ * 100)), 1500);
}

void MainWindow::zoomOut() {
    zoom_ = qMax(0.5, zoom_ * 0.95);
    applyZoom();
    statusBar()->showMessage(QStringLiteral("缩放: %1%").arg(static_cast<int>(zoom_ * 100)), 1500);
}

void MainWindow::resetZoom() {
    zoom_ = 1.0;
    applyZoom();
    statusBar()->showMessage(QStringLiteral("缩放已重置"), 1500);
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
    QAction *copyAction = menu.addAction(QStringLiteral("复制摘要"));
    QAction *inspectAction = menu.addAction(QStringLiteral("查看请求/响应"));
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
    const QTableWidgetItem *direction = packetTable_->item(row, 2);
    const QTableWidgetItem *length = packetTable_->item(row, 3);
    requestEditor_->setPlainText(
        QStringLiteral("数据包摘要\n\n方向: %1\n长度: %2\n摘要: %3")
            .arg(direction ? direction->text() : QString(),
                 length ? length->text() : QString(),
                 summary ? summary->text() : QString()));
    responseEditor_->clear();
    requestDock_->raise();
}
