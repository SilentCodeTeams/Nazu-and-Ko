#include "Dialogs.h"

#include "Translator.h"

#include <QCheckBox>
#include <QComboBox>
#include <QDialogButtonBox>
#include <QFormLayout>
#include <QHBoxLayout>
#include <QLabel>
#include <QLineEdit>
#include <QMessageBox>
#include <QPushButton>
#include <QSettings>
#include <QSpinBox>
#include <QTabWidget>
#include <QVBoxLayout>
#include <QWidget>

namespace {

QString translate(const QString &key) {
    return Translator::instance().text(key);
}

void addProtocolItems(QComboBox *combo) {
    combo->addItem(translate("dialog.proxy.protocolAuto"), QStringLiteral("auto"));
    combo->addItem(translate("dialog.proxy.protocolHttp"), QStringLiteral("http"));
    combo->addItem(translate("dialog.proxy.protocolHttps"), QStringLiteral("https"));
    combo->addItem(translate("dialog.proxy.protocolMc"), QStringLiteral("minecraft"));
}

void addModeItems(QComboBox *combo) {
    combo->addItem(translate("dialog.proxy.modeIntercept"), QStringLiteral("intercept"));
    combo->addItem(translate("dialog.proxy.modeMitm"), QStringLiteral("mitm"));
    combo->addItem(translate("dialog.proxy.modeMonitor"), QStringLiteral("monitor"));
    combo->addItem(translate("dialog.proxy.modeTransparent"), QStringLiteral("transparent"));
}

QString themeKey(int index) {
    switch (index) {
    case 0:
        return QStringLiteral("light");
    case 1:
        return QStringLiteral("dark");
    default:
        return QStringLiteral("system");
    }
}

} // namespace

ProxyConfigDialog::ProxyConfigDialog(const ProxySettings &current, QWidget *parent)
    : QDialog(parent) {
    setWindowTitle(translate("dialog.proxyConfig.title"));
    setModal(true);
    resize(480, 320);

    auto *layout = new QVBoxLayout(this);
    auto *form = new QFormLayout();

    listenPort_ = new QSpinBox(this);
    listenPort_->setRange(1, 65535);
    listenPort_->setValue(current.listenPort);
    form->addRow(translate("dialog.proxy.listenPort"), listenPort_);

    targetHost_ = new QLineEdit(current.targetHost, this);
    form->addRow(translate("dialog.proxy.targetHost"), targetHost_);

    targetPort_ = new QSpinBox(this);
    targetPort_->setRange(1, 65535);
    targetPort_->setValue(current.targetPort);
    form->addRow(translate("dialog.proxy.targetPort"), targetPort_);

    protocol_ = new QComboBox(this);
    addProtocolItems(protocol_);
    protocol_->setCurrentIndex(qMax(0, protocol_->findData(current.protocol)));
    form->addRow(translate("dialog.proxy.protocol"), protocol_);

    mode_ = new QComboBox(this);
    addModeItems(mode_);
    mode_->setCurrentIndex(qMax(0, mode_->findData(current.mode)));
    form->addRow(translate("dialog.proxy.mode"), mode_);

    layout->addLayout(form);
    layout->addStretch();

    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel, this);
    buttons->button(QDialogButtonBox::Ok)->setText(translate("button.ok"));
    buttons->button(QDialogButtonBox::Cancel)->setText(translate("button.cancel"));
    connect(buttons, &QDialogButtonBox::accepted, this, &QDialog::accept);
    connect(buttons, &QDialogButtonBox::rejected, this, &QDialog::reject);
    layout->addWidget(buttons);
}

ProxySettings ProxyConfigDialog::settings() const {
    ProxySettings result;
    result.listenPort = listenPort_->value();
    result.targetHost = targetHost_->text().trimmed();
    result.targetPort = targetPort_->value();
    result.protocol = protocol_->currentData().toString();
    result.mode = mode_->currentData().toString();
    return result;
}

SmartExportDialog::SmartExportDialog(QWidget *parent) : QDialog(parent) {
    setWindowTitle(translate("dialog.export.title"));
    setModal(true);
    resize(420, 220);

    auto *layout = new QVBoxLayout(this);
    auto *form = new QFormLayout();

    format_ = new QComboBox(this);
    format_->addItem(translate("dialog.export.csv"), QStringLiteral("csv"));
    format_->addItem(translate("dialog.export.json"), QStringLiteral("json"));
    format_->addItem(translate("dialog.export.tsv"), QStringLiteral("tsv"));
    form->addRow(translate("dialog.export.format"), format_);

    scope_ = new QComboBox(this);
    scope_->addItem(translate("dialog.export.all"), QStringLiteral("all"));
    scope_->addItem(translate("dialog.export.visible"), QStringLiteral("visible"));
    form->addRow(translate("dialog.export.scope"), scope_);

    layout->addLayout(form);
    layout->addStretch();

    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel, this);
    buttons->button(QDialogButtonBox::Ok)->setText(translate("button.ok"));
    buttons->button(QDialogButtonBox::Cancel)->setText(translate("button.cancel"));
    connect(buttons, &QDialogButtonBox::accepted, this, &QDialog::accept);
    connect(buttons, &QDialogButtonBox::rejected, this, &QDialog::reject);
    layout->addWidget(buttons);
}

QString SmartExportDialog::format() const {
    return format_->currentData().toString();
}

bool SmartExportDialog::visibleOnly() const {
    return scope_->currentData().toString() == QStringLiteral("visible");
}

SettingsDialog::SettingsDialog(QWidget *parent) : QDialog(parent) {
    setWindowTitle(translate("dialog.settings.title"));
    setModal(true);
    resize(620, 480);

    QSettings settings;
    auto *layout = new QVBoxLayout(this);
    auto *tabs = new QTabWidget(this);

    auto *general = new QWidget(tabs);
    auto *generalForm = new QFormLayout(general);
    theme_ = new QComboBox(general);
    theme_->addItems({translate("theme.light"), translate("theme.dark"), translate("theme.system")});
    const QString theme = settings.value(QStringLiteral("ui/theme"), QStringLiteral("system")).toString();
    theme_->setCurrentIndex(theme == QStringLiteral("light") ? 0 : theme == QStringLiteral("dark") ? 1 : 2);
    generalForm->addRow(translate("dialog.settings.theme"), theme_);

    fontSize_ = new QSpinBox(general);
    fontSize_->setRange(8, 20);
    fontSize_->setValue(settings.value(QStringLiteral("ui/fontSize"), 10).toInt());
    generalForm->addRow(translate("dialog.settings.fontSize"), fontSize_);

    language_ = new QComboBox(general);
    const QString language = settings.value(QStringLiteral("ui/language")).toString();
    for (const auto &entry : Translator::instance().languages()) {
        language_->addItem(entry.second, entry.first);
    }
    const int languageIndex = language_->findData(language);
    language_->setCurrentIndex(languageIndex >= 0 ? languageIndex : language_->findData(QStringLiteral("en")));
    generalForm->addRow(translate("dialog.settings.language"), language_);
    tabs->addTab(general, translate("dialog.settings.general"));

    auto *notifications = new QWidget(tabs);
    auto *notificationForm = new QFormLayout(notifications);
    notificationEnabled_ = new QCheckBox(notifications);
    notificationEnabled_->setChecked(settings.value(QStringLiteral("notification/enabled"), true).toBool());
    notificationForm->addRow(translate("dialog.settings.notificationEnabled"), notificationEnabled_);
    notificationDuration_ = new QSpinBox(notifications);
    notificationDuration_->setRange(1000, 10000);
    notificationDuration_->setSingleStep(500);
    notificationDuration_->setSuffix(QStringLiteral(" ms"));
    notificationDuration_->setValue(settings.value(QStringLiteral("notification/duration"), 3000).toInt());
    notificationForm->addRow(translate("dialog.settings.notificationDuration"), notificationDuration_);
    tabs->addTab(notifications, translate("dialog.settings.notifications"));

    auto *proxy = new QWidget(tabs);
    auto *proxyForm = new QFormLayout(proxy);
    listenPort_ = new QSpinBox(proxy);
    listenPort_->setRange(1, 65535);
    listenPort_->setValue(settings.value(QStringLiteral("proxy/listenPort"), 25565).toInt());
    proxyForm->addRow(translate("dialog.proxy.listenPort"), listenPort_);
    targetHost_ = new QLineEdit(settings.value(QStringLiteral("proxy/targetHost"), QStringLiteral("localhost")).toString(), proxy);
    proxyForm->addRow(translate("dialog.proxy.targetHost"), targetHost_);
    targetPort_ = new QSpinBox(proxy);
    targetPort_->setRange(1, 65535);
    targetPort_->setValue(settings.value(QStringLiteral("proxy/targetPort"), 25566).toInt());
    proxyForm->addRow(translate("dialog.proxy.targetPort"), targetPort_);
    autoStart_ = new QCheckBox(proxy);
    autoStart_->setChecked(settings.value(QStringLiteral("proxy/autoStart"), false).toBool());
    proxyForm->addRow(translate("dialog.settings.autoStart"), autoStart_);
    tabs->addTab(proxy, translate("dialog.settings.proxy"));

    auto *web = new QWidget(tabs);
    auto *webForm = new QFormLayout(web);
    webEnabled_ = new QCheckBox(web);
    webEnabled_->setChecked(settings.value(QStringLiteral("web/enabled"), true).toBool());
    webForm->addRow(translate("dialog.settings.webEnabled"), webEnabled_);
    webPort_ = new QSpinBox(web);
    webPort_->setRange(1024, 65535);
    webPort_->setValue(settings.value(QStringLiteral("web/port"), 2679).toInt());
    webForm->addRow(translate("dialog.settings.webPort"), webPort_);
    tabs->addTab(web, translate("dialog.settings.web"));

    layout->addWidget(tabs);

    auto *buttonRow = new QHBoxLayout();
    buttonRow->addStretch(1);
    auto *cancel = new QPushButton(translate("button.cancel"), this);
    auto *save = new QPushButton(translate("button.save"), this);
    save->setObjectName(QStringLiteral("primaryButton"));
    connect(cancel, &QPushButton::clicked, this, &QDialog::reject);
    connect(save, &QPushButton::clicked, this, &SettingsDialog::saveSettings);
    buttonRow->addWidget(cancel);
    buttonRow->addWidget(save);
    layout->addLayout(buttonRow);
}

QString SettingsDialog::selectedTheme() const {
    return themeKey(theme_->currentIndex());
}

int SettingsDialog::selectedFontSize() const {
    return fontSize_->value();
}

void SettingsDialog::saveSettings() {
    QSettings settings;
    settings.setValue(QStringLiteral("ui/theme"), selectedTheme());
    settings.setValue(QStringLiteral("ui/fontSize"), fontSize_->value());
    const QString previousLanguage = Translator::instance().code();
    const QString selectedLanguage = language_->currentData().toString();
    settings.setValue(QStringLiteral("ui/language"), selectedLanguage);
    settings.setValue(QStringLiteral("notification/enabled"), notificationEnabled_->isChecked());
    settings.setValue(QStringLiteral("notification/duration"), notificationDuration_->value());
    settings.setValue(QStringLiteral("proxy/listenPort"), listenPort_->value());
    settings.setValue(QStringLiteral("proxy/targetHost"), targetHost_->text().trimmed());
    settings.setValue(QStringLiteral("proxy/targetPort"), targetPort_->value());
    settings.setValue(QStringLiteral("proxy/autoStart"), autoStart_->isChecked());
    settings.setValue(QStringLiteral("web/enabled"), webEnabled_->isChecked());
    settings.setValue(QStringLiteral("web/port"), webPort_->value());

    if (selectedLanguage != previousLanguage) {
        QMessageBox::information(this, translate("language.title"), translate("language.restart"));
    }
    accept();
}
