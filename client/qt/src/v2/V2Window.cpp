#include "V2Window.h"

#include "Translator.h"

#include <QApplication>
#include <QButtonGroup>
#include <QFrame>
#include <QGridLayout>
#include <QGuiApplication>
#include <QHBoxLayout>
#include <QHeaderView>
#include <QLabel>
#include <QLineEdit>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QScrollArea>
#include <QSplitter>
#include <QStackedWidget>
#include <QStyleHints>
#include <QTableWidget>
#include <QToolButton>
#include <QVBoxLayout>

namespace {

QString translate(const QString &key) {
    return Translator::instance().text(key);
}

QString lightStyle() {
    return QStringLiteral(R"STYLE(
QMainWindow { background: #F6F7F9; }
QWidget { background: #F6F7F9; color: #111827; }
QFrame#topBar, QFrame#rail, QFrame#panel { background: #FFFFFF; }
QFrame#topBar { border-bottom: 1px solid #E5E8EC; }
QFrame#rail { border-right: 1px solid #E5E8EC; }
QFrame#panel { border: 1px solid #E5E8EC; border-radius: 12px; }
QLabel { background: transparent; }
QLabel#brand { color: #111827; font-size: 14px; font-weight: 700; }
QLabel#brandSub { color: #98A2B3; font-size: 9px; font-weight: 600; }
QLabel#muted { color: #667085; }
QLabel#eyebrow { color: #0F766E; font-size: 10px; font-weight: 700; }
QLabel#pageTitle { color: #111827; font-size: 34px; font-weight: 700; }
QLabel#pageSubtitle { color: #667085; font-size: 12px; }
QLabel#sectionLabel { color: #667085; font-size: 10px; font-weight: 700; }
QLabel#emptyHint { color: #667085; font-size: 11px; }
QLabel#statusPill { color: #B54708; background: #FFFAEB; border: 1px solid #FEDF89; border-radius: 10px; padding: 3px 9px; font-size: 11px; font-weight: 600; }
QLabel#logo { color: #FFFFFF; background: #0F766E; border-radius: 8px; font-size: 18px; font-weight: 700; }
QLabel#actionIndex { color: #98A2B3; font-size: 10px; font-weight: 600; }
QLabel#actionTitle { color: #111827; font-size: 15px; font-weight: 700; }
QLabel#actionDesc { color: #667085; font-size: 11px; }
QToolButton#railButton { color: #667085; background: transparent; border: none; border-radius: 9px; font-size: 11px; font-weight: 600; }
QToolButton#railButton:hover { background: #F2F4F7; }
QToolButton#railButton:checked { background: #E6FFFA; color: #0F766E; }
QToolButton#themeButton { color: #475467; background: #FFFFFF; border: 1px solid #E5E8EC; border-radius: 8px; font-size: 15px; }
QToolButton#themeButton:hover { background: #F9FAFB; }
QLineEdit#commandInput { background: #F9FAFB; border: 1px solid #E5E8EC; border-radius: 8px; padding: 7px 11px; color: #475467; }
QLineEdit#promptInput { background: #FFFFFF; border: 1px solid #D0D5DD; border-radius: 9px; padding: 11px 13px; color: #111827; font-size: 13px; }
QPushButton#primary { background: #0F766E; color: #FFFFFF; border: none; border-radius: 8px; padding: 9px 16px; font-weight: 600; }
QPushButton#primary:hover { background: #0D9488; }
QPushButton#actionCard { background: #FFFFFF; border: 1px solid #E5E8EC; border-radius: 12px; text-align: left; }
QPushButton#actionCard:hover { border-color: #0F766E; background: #F0FDFA; }
QPushButton#toolCard { background: #FFFFFF; border: 1px solid #E5E8EC; border-radius: 12px; text-align: left; }
QPushButton#toolCard:hover { border-color: #0F766E; background: #F0FDFA; }
QPushButton#segment { background: #FFFFFF; color: #667085; border: 1px solid #E5E8EC; border-radius: 7px; padding: 7px 12px; font-size: 11px; font-weight: 600; }
QPushButton#segment:hover { background: #F9FAFB; }
QPushButton#segment:checked { background: #E6FFFA; color: #0F766E; border-color: #99F6E4; }
QTableWidget { background: #FFFFFF; alternate-background-color: #F9FAFB; border: none; gridline-color: transparent; selection-background-color: #CCFBF1; selection-color: #111827; }
QHeaderView::section { background: #FFFFFF; color: #667085; border: none; border-bottom: 1px solid #E5E8EC; padding: 8px 6px; font-weight: 600; }
QPlainTextEdit#inspector { background: #FFFFFF; border: none; color: #475467; }
QSplitter::handle { background: #F6F7F9; width: 8px; }
QScrollArea { border: none; background: #F6F7F9; }
)STYLE");
}

QString darkStyle() {
    return QStringLiteral(R"STYLE(
QMainWindow { background: #0B0E13; }
QWidget { background: #0B0E13; color: #E8EDF3; }
QFrame#topBar, QFrame#rail, QFrame#panel { background: #11161F; }
QFrame#topBar { border-bottom: 1px solid #222A35; }
QFrame#rail { border-right: 1px solid #222A35; }
QFrame#panel { border: 1px solid #222A35; border-radius: 12px; }
QLabel { background: transparent; }
QLabel#brand { color: #E8EDF3; font-size: 14px; font-weight: 700; }
QLabel#brandSub { color: #64748B; font-size: 9px; font-weight: 600; }
QLabel#muted { color: #8B96A7; }
QLabel#eyebrow { color: #2DD4BF; font-size: 10px; font-weight: 700; }
QLabel#pageTitle { color: #E8EDF3; font-size: 34px; font-weight: 700; }
QLabel#pageSubtitle { color: #8B96A7; font-size: 12px; }
QLabel#sectionLabel { color: #8B96A7; font-size: 10px; font-weight: 700; }
QLabel#emptyHint { color: #8B96A7; font-size: 11px; }
QLabel#statusPill { color: #FDB022; background: #2A2114; border: 1px solid #4E3A1A; border-radius: 10px; padding: 3px 9px; font-size: 11px; font-weight: 600; }
QLabel#logo { color: #042F2E; background: #2DD4BF; border-radius: 8px; font-size: 18px; font-weight: 700; }
QLabel#actionIndex { color: #64748B; font-size: 10px; font-weight: 600; }
QLabel#actionTitle { color: #E8EDF3; font-size: 15px; font-weight: 700; }
QLabel#actionDesc { color: #8B96A7; font-size: 11px; }
QToolButton#railButton { color: #8B96A7; background: transparent; border: none; border-radius: 9px; font-size: 11px; font-weight: 600; }
QToolButton#railButton:hover { background: #1A2230; }
QToolButton#railButton:checked { background: #0F2A2A; color: #2DD4BF; }
QToolButton#themeButton { color: #A7B3C2; background: #11161F; border: 1px solid #222A35; border-radius: 8px; font-size: 15px; }
QToolButton#themeButton:hover { background: #1A2230; }
QLineEdit#commandInput { background: #0F141B; border: 1px solid #222A35; border-radius: 8px; padding: 7px 11px; color: #A7B3C2; }
QLineEdit#promptInput { background: #11161F; border: 1px solid #334155; border-radius: 9px; padding: 11px 13px; color: #E8EDF3; font-size: 13px; }
QPushButton#primary { background: #2DD4BF; color: #042F2E; border: none; border-radius: 8px; padding: 9px 16px; font-weight: 700; }
QPushButton#primary:hover { background: #5EEAD4; }
QPushButton#actionCard { background: #11161F; border: 1px solid #222A35; border-radius: 12px; text-align: left; }
QPushButton#actionCard:hover { border-color: #2DD4BF; background: #0F2A2A; }
QPushButton#toolCard { background: #11161F; border: 1px solid #222A35; border-radius: 12px; text-align: left; }
QPushButton#toolCard:hover { border-color: #2DD4BF; background: #0F2A2A; }
QPushButton#segment { background: #11161F; color: #8B96A7; border: 1px solid #222A35; border-radius: 7px; padding: 7px 12px; font-size: 11px; font-weight: 600; }
QPushButton#segment:hover { background: #1A2230; }
QPushButton#segment:checked { background: #0F2A2A; color: #2DD4BF; border-color: #134E4A; }
QTableWidget { background: #11161F; alternate-background-color: #0F141B; border: none; gridline-color: transparent; selection-background-color: #134E4A; selection-color: #E8EDF3; }
QHeaderView::section { background: #11161F; color: #8B96A7; border: none; border-bottom: 1px solid #222A35; padding: 8px 6px; font-weight: 600; }
QPlainTextEdit#inspector { background: #11161F; border: none; color: #A7B3C2; }
QSplitter::handle { background: #0B0E13; width: 8px; }
QScrollArea { border: none; background: #0B0E13; }
)STYLE");
}

} // namespace

V2Window::V2Window(QWidget *parent) : QMainWindow(parent) {
    setWindowTitle(QStringLiteral("NetTool ζ"));
    resize(1440, 900);
    setMinimumSize(1080, 700);

    auto *root = new QWidget(this);
    auto *rootLayout = new QVBoxLayout(root);
    rootLayout->setContentsMargins(0, 0, 0, 0);
    rootLayout->setSpacing(0);
    rootLayout->addWidget(createTopBar());

    auto *body = new QHBoxLayout();
    body->setContentsMargins(0, 0, 0, 0);
    body->setSpacing(0);
    body->addWidget(createRail());
    stack_ = new QStackedWidget(root);
    stack_->addWidget(createHomePage());
    stack_->addWidget(createTrafficPage());
    stack_->addWidget(createPlaceholderPage(QStringLiteral("v2.attack.title"), QStringLiteral("v2.attack.desc")));
    stack_->addWidget(createPlaceholderPage(QStringLiteral("v2.scan.title"), QStringLiteral("v2.scan.desc")));
    stack_->addWidget(createToolsPage());
    stack_->addWidget(createSettingsPage());
    body->addWidget(stack_, 1);
    rootLayout->addLayout(body, 1);
    setCentralWidget(root);

    dark_ = QGuiApplication::styleHints()->colorScheme() == Qt::ColorScheme::Dark;
    applyTheme(dark_);
    navigate(0);
}

QWidget *V2Window::createTopBar() {
    auto *bar = new QFrame(this);
    bar->setObjectName(QStringLiteral("topBar"));
    bar->setFixedHeight(64);
    auto *layout = new QHBoxLayout(bar);
    layout->setContentsMargins(20, 0, 20, 0);
    layout->setSpacing(12);

    auto *logo = new QLabel(QStringLiteral("ζ"), bar);
    logo->setObjectName(QStringLiteral("logo"));
    logo->setFixedSize(30, 30);
    logo->setAlignment(Qt::AlignCenter);
    layout->addWidget(logo);

    auto *brand = new QVBoxLayout();
    brand->setSpacing(0);
    auto *name = new QLabel(QStringLiteral("NetTool ζ"), bar);
    name->setObjectName(QStringLiteral("brand"));
    auto *sub = new QLabel(translate("v2.brandSub"), bar);
    sub->setObjectName(QStringLiteral("brandSub"));
    brand->addWidget(name);
    brand->addWidget(sub);
    layout->addLayout(brand);
    layout->addSpacing(18);

    statusPill_ = new QLabel(translate("v2.status.offline"), bar);
    statusPill_->setObjectName(QStringLiteral("statusPill"));
    layout->addWidget(statusPill_);
    layout->addStretch(1);

    auto *command = new QLineEdit(bar);
    command->setObjectName(QStringLiteral("commandInput"));
    command->setPlaceholderText(translate("v2.command.placeholder"));
    command->setFixedWidth(310);
    connect(command, &QLineEdit::returnPressed, this, [this] { navigate(1); });
    layout->addWidget(command);

    auto *theme = new QToolButton(bar);
    theme->setObjectName(QStringLiteral("themeButton"));
    theme->setText(QStringLiteral("☾"));
    theme->setFixedSize(34, 30);
    connect(theme, &QToolButton::clicked, this, [this] {
        dark_ = !dark_;
        applyTheme(dark_);
    });
    themeButton_ = nullptr;
    layout->addWidget(theme);

    auto *web = new QLabel(QStringLiteral("127.0.0.1:2679"), bar);
    web->setObjectName(QStringLiteral("muted"));
    layout->addWidget(web);
    return bar;
}

QWidget *V2Window::createRail() {
    auto *rail = new QFrame(this);
    rail->setObjectName(QStringLiteral("rail"));
    rail->setFixedWidth(88);
    auto *layout = new QVBoxLayout(rail);
    layout->setContentsMargins(10, 18, 10, 14);
    layout->setSpacing(6);

    railGroup_ = new QButtonGroup(this);
    railGroup_->setExclusive(true);
    const QStringList keys = {
        QStringLiteral("v2.nav.home"), QStringLiteral("v2.nav.traffic"),
        QStringLiteral("v2.nav.attack"), QStringLiteral("v2.nav.scan"),
        QStringLiteral("v2.nav.tools"), QStringLiteral("v2.nav.settings"),
    };
    for (int index = 0; index < keys.size(); ++index) {
        auto *button = new QToolButton(rail);
        button->setObjectName(QStringLiteral("railButton"));
        button->setText(translate(keys.at(index)));
        button->setCheckable(true);
        button->setFixedSize(68, 50);
        button->setToolButtonStyle(Qt::ToolButtonTextOnly);
        railGroup_->addButton(button, index);
        layout->addWidget(button, 0, Qt::AlignHCenter);
    }
    connect(railGroup_, &QButtonGroup::idClicked, this, &V2Window::navigate);
    layout->addStretch(1);
    auto *version = new QLabel(translate("v2.previewLabel"), rail);
    version->setObjectName(QStringLiteral("muted"));
    version->setAlignment(Qt::AlignCenter);
    layout->addWidget(version);
    return rail;
}

QWidget *V2Window::createHomePage() {
    auto *page = new QWidget(this);
    auto *outer = new QHBoxLayout(page);
    outer->setContentsMargins(40, 0, 40, 0);
    outer->addStretch(1);
    auto *center = new QWidget(page);
    center->setMaximumWidth(940);
    auto *layout = new QVBoxLayout(center);
    layout->setContentsMargins(0, 44, 0, 24);
    layout->setSpacing(14);

    auto *eyebrow = new QLabel(translate("v2.home.eyebrow"), center);
    eyebrow->setObjectName(QStringLiteral("eyebrow"));
    auto *title = new QLabel(translate("v2.home.title"), center);
    title->setObjectName(QStringLiteral("pageTitle"));
    auto *subtitle = new QLabel(translate("v2.home.subtitle"), center);
    subtitle->setObjectName(QStringLiteral("pageSubtitle"));
    subtitle->setWordWrap(true);
    layout->addWidget(eyebrow);
    layout->addWidget(title);
    layout->addWidget(subtitle);

    auto *promptRow = new QHBoxLayout();
    promptRow->setSpacing(8);
    auto *prompt = new QLineEdit(center);
    prompt->setObjectName(QStringLiteral("promptInput"));
    prompt->setPlaceholderText(translate("v2.home.prompt"));
    auto *start = new QPushButton(translate("v2.home.start"), center);
    start->setObjectName(QStringLiteral("primary"));
    connect(start, &QPushButton::clicked, this, [this] { navigate(1); });
    connect(prompt, &QLineEdit::returnPressed, this, [this] { navigate(1); });
    promptRow->addWidget(prompt, 1);
    promptRow->addWidget(start);
    layout->addSpacing(4);
    layout->addLayout(promptRow);
    layout->addSpacing(12);

    auto *section = new QLabel(translate("v2.home.actions"), center);
    section->setObjectName(QStringLiteral("sectionLabel"));
    layout->addWidget(section);

    auto *grid = new QGridLayout();
    grid->setSpacing(12);
    grid->addWidget(createActionCard(QStringLiteral("01"), QStringLiteral("v2.card.traffic.title"),
                                     QStringLiteral("v2.card.traffic.desc"), 1), 0, 0);
    grid->addWidget(createActionCard(QStringLiteral("02"), QStringLiteral("v2.card.attack.title"),
                                     QStringLiteral("v2.card.attack.desc"), 2), 0, 1);
    grid->addWidget(createActionCard(QStringLiteral("03"), QStringLiteral("v2.card.scan.title"),
                                     QStringLiteral("v2.card.scan.desc"), 3), 1, 0);
    grid->addWidget(createActionCard(QStringLiteral("04"), QStringLiteral("v2.card.custom.title"),
                                     QStringLiteral("v2.card.custom.desc"), 1), 1, 1);
    layout->addLayout(grid);
    layout->addStretch(1);

    auto *footer = new QLabel(translate("v2.home.footer"), center);
    footer->setObjectName(QStringLiteral("muted"));
    layout->addWidget(footer);

    outer->addWidget(center, 1);
    outer->addStretch(1);
    return page;
}

QWidget *V2Window::createActionCard(const QString &index, const QString &titleKey,
                                    const QString &descriptionKey, int page) {
    auto *button = new QPushButton(this);
    button->setObjectName(QStringLiteral("actionCard"));
    button->setMinimumHeight(142);
    button->setCursor(Qt::PointingHandCursor);
    auto *layout = new QVBoxLayout(button);
    layout->setContentsMargins(18, 16, 18, 16);
    layout->setSpacing(6);
    auto *indexLabel = new QLabel(index, button);
    indexLabel->setObjectName(QStringLiteral("actionIndex"));
    auto *title = new QLabel(translate(titleKey), button);
    title->setObjectName(QStringLiteral("actionTitle"));
    auto *description = new QLabel(translate(descriptionKey), button);
    description->setObjectName(QStringLiteral("actionDesc"));
    description->setWordWrap(true);
    layout->addWidget(indexLabel);
    layout->addWidget(title);
    layout->addWidget(description);
    layout->addStretch(1);
    connect(button, &QPushButton::clicked, this, [this, page] { navigate(page); });
    return button;
}

QWidget *V2Window::createTrafficPage() {
    auto *page = new QWidget(this);
    auto *layout = new QVBoxLayout(page);
    layout->setContentsMargins(32, 28, 32, 24);
    layout->setSpacing(16);

    auto *header = new QHBoxLayout();
    auto *titles = new QVBoxLayout();
    auto *title = new QLabel(translate("v2.traffic.title"), page);
    title->setObjectName(QStringLiteral("pageTitle"));
    auto *subtitle = new QLabel(translate("v2.traffic.subtitle"), page);
    subtitle->setObjectName(QStringLiteral("pageSubtitle"));
    titles->addWidget(title);
    titles->addWidget(subtitle);
    header->addLayout(titles, 1);
    auto *start = new QPushButton(translate("v2.traffic.start"), page);
    start->setObjectName(QStringLiteral("primary"));
    header->addWidget(start);
    layout->addLayout(header);

    auto *splitter = new QSplitter(Qt::Horizontal, page);
    auto *tablePanel = new QFrame(splitter);
    tablePanel->setObjectName(QStringLiteral("panel"));
    auto *tableLayout = new QVBoxLayout(tablePanel);
    tableLayout->setContentsMargins(0, 0, 0, 0);
    auto *hint = new QLabel(translate("v2.traffic.empty"), tablePanel);
    hint->setObjectName(QStringLiteral("emptyHint"));
    hint->setContentsMargins(14, 10, 14, 10);
    auto *table = new QTableWidget(tablePanel);
    table->setColumnCount(6);
    table->setHorizontalHeaderLabels({translate("table.id"), translate("table.time"), translate("table.direction"),
                                      translate("table.length"), translate("table.status"), translate("table.summary")});
    table->verticalHeader()->setVisible(false);
    table->setShowGrid(false);
    table->setAlternatingRowColors(true);
    table->horizontalHeader()->setSectionResizeMode(5, QHeaderView::Stretch);
    tableLayout->addWidget(hint);
    tableLayout->addWidget(table, 1);
    splitter->addWidget(tablePanel);

    auto *inspector = new QFrame(splitter);
    inspector->setObjectName(QStringLiteral("panel"));
    auto *inspectorLayout = new QVBoxLayout(inspector);
    inspectorLayout->setContentsMargins(14, 12, 14, 12);
    auto *label = new QLabel(translate("v2.traffic.inspector"), inspector);
    label->setObjectName(QStringLiteral("sectionLabel"));
    auto *content = new QPlainTextEdit(inspector);
    content->setObjectName(QStringLiteral("inspector"));
    content->setReadOnly(true);
    content->setPlaceholderText(translate("v2.traffic.inspectorEmpty"));
    inspectorLayout->addWidget(label);
    inspectorLayout->addWidget(content, 1);
    splitter->addWidget(inspector);
    splitter->setStretchFactor(0, 3);
    splitter->setStretchFactor(1, 2);
    layout->addWidget(splitter, 1);
    return page;
}

QWidget *V2Window::createToolsPage() {
    auto *page = new QWidget(this);
    auto *layout = new QVBoxLayout(page);
    layout->setContentsMargins(32, 28, 32, 24);
    layout->setSpacing(14);
    auto *title = new QLabel(translate("v2.tools.title"), page);
    title->setObjectName(QStringLiteral("pageTitle"));
    auto *subtitle = new QLabel(translate("v2.tools.subtitle"), page);
    subtitle->setObjectName(QStringLiteral("pageSubtitle"));
    layout->addWidget(title);
    layout->addWidget(subtitle);
    layout->addSpacing(10);

    auto *grid = new QGridLayout();
    grid->setSpacing(12);
    const QStringList titles = {translate("v2.tools.repeater"), translate("v2.tools.intruder"),
                                translate("v2.tools.scanner"), translate("v2.tools.rules")};
    const QStringList descriptions = {translate("v2.tools.repeaterDesc"), translate("v2.tools.intruderDesc"),
                                      translate("v2.tools.scannerDesc"), translate("v2.tools.rulesDesc")};
    for (int index = 0; index < titles.size(); ++index) {
        auto *card = new QPushButton(page);
        card->setObjectName(QStringLiteral("toolCard"));
        card->setMinimumHeight(120);
        card->setCursor(Qt::PointingHandCursor);
        auto *cardLayout = new QVBoxLayout(card);
        cardLayout->setContentsMargins(18, 16, 18, 16);
        auto *heading = new QLabel(titles.at(index), card);
        heading->setObjectName(QStringLiteral("actionTitle"));
        auto *description = new QLabel(descriptions.at(index), card);
        description->setObjectName(QStringLiteral("actionDesc"));
        description->setWordWrap(true);
        cardLayout->addWidget(heading);
        cardLayout->addWidget(description);
        cardLayout->addStretch(1);
        grid->addWidget(card, index / 2, index % 2);
    }
    layout->addLayout(grid);
    layout->addStretch(1);
    return page;
}

QWidget *V2Window::createSettingsPage() {
    auto *page = new QWidget(this);
    auto *layout = new QVBoxLayout(page);
    layout->setContentsMargins(32, 28, 32, 24);
    layout->setSpacing(14);
    auto *title = new QLabel(translate("v2.settings.title"), page);
    title->setObjectName(QStringLiteral("pageTitle"));
    auto *subtitle = new QLabel(translate("v2.settings.subtitle"), page);
    subtitle->setObjectName(QStringLiteral("pageSubtitle"));
    layout->addWidget(title);
    layout->addWidget(subtitle);
    layout->addSpacing(14);

    auto *panel = new QFrame(page);
    panel->setObjectName(QStringLiteral("panel"));
    auto *panelLayout = new QVBoxLayout(panel);
    panelLayout->setContentsMargins(20, 18, 20, 18);
    panelLayout->setSpacing(10);

    auto addSection = [&](const QString &label) {
        auto *section = new QLabel(label, panel);
        section->setObjectName(QStringLiteral("sectionLabel"));
        panelLayout->addWidget(section);
    };
    addSection(translate("v2.settings.appearance"));
    auto *themeRow = new QHBoxLayout();
    auto *themeGroup = new QButtonGroup(panel);
    themeGroup->setExclusive(true);
    const QStringList themes = {translate("theme.light"), translate("theme.dark"), translate("theme.system")};
    for (int index = 0; index < themes.size(); ++index) {
        auto *button = new QPushButton(themes.at(index), panel);
        button->setObjectName(QStringLiteral("segment"));
        button->setCheckable(true);
        if (index == 2) {
            button->setChecked(true);
        }
        themeGroup->addButton(button, index);
        themeRow->addWidget(button);
    }
    connect(themeGroup, &QButtonGroup::idClicked, this, [this](int index) {
        if (index == 0) {
            dark_ = false;
        } else if (index == 1) {
            dark_ = true;
        } else {
            dark_ = QGuiApplication::styleHints()->colorScheme() == Qt::ColorScheme::Dark;
        }
        applyTheme(dark_);
    });
    themeRow->addStretch(1);
    panelLayout->addLayout(themeRow);

    addSection(translate("v2.settings.language"));
    auto *languageRow = new QHBoxLayout();
    auto *languageGroup = new QButtonGroup(panel);
    languageGroup->setExclusive(true);
    const QStringList languages = {translate("v2.language.zh"), translate("v2.language.en")};
    for (int index = 0; index < languages.size(); ++index) {
        auto *button = new QPushButton(languages.at(index), panel);
        button->setObjectName(QStringLiteral("segment"));
        button->setCheckable(true);
        if (index == 0) {
            button->setChecked(true);
        }
        languageGroup->addButton(button, index);
        languageRow->addWidget(button);
    }
    languageRow->addStretch(1);
    panelLayout->addLayout(languageRow);

    addSection(translate("v2.settings.mode"));
    auto *modeRow = new QHBoxLayout();
    auto *modeGroup = new QButtonGroup(panel);
    modeGroup->setExclusive(true);
    const QStringList modes = {translate("dialog.proxy.modeIntercept"), translate("dialog.proxy.modeMitm"),
                               translate("dialog.proxy.modeMonitor"), translate("dialog.proxy.modeTransparent")};
    for (int index = 0; index < modes.size(); ++index) {
        auto *button = new QPushButton(modes.at(index), panel);
        button->setObjectName(QStringLiteral("segment"));
        button->setCheckable(true);
        if (index == 0) {
            button->setChecked(true);
        }
        modeGroup->addButton(button, index);
        modeRow->addWidget(button);
    }
    modeRow->addStretch(1);
    panelLayout->addLayout(modeRow);

    layout->addWidget(panel);
    layout->addStretch(1);
    return page;
}

QWidget *V2Window::createPlaceholderPage(const QString &titleKey, const QString &subtitleKey) {
    auto *page = new QWidget(this);
    auto *layout = new QVBoxLayout(page);
    layout->setContentsMargins(32, 28, 32, 24);
    auto *title = new QLabel(translate(titleKey), page);
    title->setObjectName(QStringLiteral("pageTitle"));
    auto *subtitle = new QLabel(translate(subtitleKey), page);
    subtitle->setObjectName(QStringLiteral("pageSubtitle"));
    subtitle->setWordWrap(true);
    layout->addWidget(title);
    layout->addWidget(subtitle);
    layout->addSpacing(20);
    auto *panel = new QFrame(page);
    panel->setObjectName(QStringLiteral("panel"));
    panel->setMinimumHeight(240);
    auto *panelLayout = new QVBoxLayout(panel);
    auto *preview = new QLabel(translate("v2.placeholder"), panel);
    preview->setObjectName(QStringLiteral("emptyHint"));
    preview->setAlignment(Qt::AlignCenter);
    panelLayout->addWidget(preview);
    layout->addWidget(panel);
    layout->addStretch(1);
    return page;
}

void V2Window::setPage(int page) {
    navigate(page);
}

void V2Window::navigate(int page) {
    if (!stack_ || page < 0 || page >= stack_->count()) {
        return;
    }
    stack_->setCurrentIndex(page);
    if (railGroup_ && railGroup_->button(page)) {
        railGroup_->button(page)->setChecked(true);
    }
}

void V2Window::applyTheme(bool dark) {
    qApp->setStyleSheet(dark ? darkStyle() : lightStyle());
    if (statusPill_) {
        statusPill_->setText(translate("v2.status.offline"));
    }
}
