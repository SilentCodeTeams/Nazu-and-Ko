#pragma once

#include <QDialog>
#include <QString>

class QCheckBox;
class QComboBox;
class QLineEdit;
class QSpinBox;

struct ProxySettings {
    int listenPort = 25565;
    QString targetHost = QStringLiteral("localhost");
    int targetPort = 25566;
    QString protocol = QStringLiteral("auto");
    QString mode = QStringLiteral("intercept");
};

class ProxyConfigDialog : public QDialog {
    Q_OBJECT

public:
    explicit ProxyConfigDialog(const ProxySettings &current, QWidget *parent = nullptr);
    ProxySettings settings() const;

private:
    QSpinBox *listenPort_ = nullptr;
    QLineEdit *targetHost_ = nullptr;
    QSpinBox *targetPort_ = nullptr;
    QComboBox *protocol_ = nullptr;
    QComboBox *mode_ = nullptr;
};

class SettingsDialog : public QDialog {
    Q_OBJECT

public:
    explicit SettingsDialog(QWidget *parent = nullptr);
    QString selectedTheme() const;
    int selectedFontSize() const;

private:
    void saveSettings();

    QComboBox *theme_ = nullptr;
    QSpinBox *fontSize_ = nullptr;
    QComboBox *language_ = nullptr;
    QCheckBox *notificationEnabled_ = nullptr;
    QSpinBox *notificationDuration_ = nullptr;
    QSpinBox *listenPort_ = nullptr;
    QLineEdit *targetHost_ = nullptr;
    QSpinBox *targetPort_ = nullptr;
    QCheckBox *autoStart_ = nullptr;
    QCheckBox *webEnabled_ = nullptr;
    QSpinBox *webPort_ = nullptr;
};
