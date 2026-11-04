// 本文件由 SilentStudio 监管与负责。
// 本文件属于 SilentCodeTeams 产品 Nazu&Ko 的源码部分
// 本项目由 SilentCodeTeams 对法律、版权事宜及相关咨询承担责任。
// 版权所有 © SilentStudio。
// 以 AGPL-3.0-or-later 授权。
// This document is supervised and responsible by SilentStudio.
// This file is part of the source code of the SilentCodeTeams product Nazu&Ko
// SilentCodeTeams is responsible for legal and copyright issues and related consultation in this project.
// Copyright © SilentStudio.
// Licensed as AGPL-3.0-or-later.

#pragma once

#include <QDialog>
#include <QString>

class QCheckBox;
class QComboBox;
class QLineEdit;
class QRadioButton;
class QSpinBox;

class SmartExportDialog : public QDialog {
    Q_OBJECT

public:
    explicit SmartExportDialog(QWidget *parent = nullptr);
    QString format() const;
    bool visibleOnly() const;

private:
    QComboBox *format_ = nullptr;
    QComboBox *scope_ = nullptr;
};

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
    QString controlsSide() const;

private:
    void saveSettings();

    QComboBox *theme_ = nullptr;
    QSpinBox *fontSize_ = nullptr;
    QComboBox *language_ = nullptr;
    QRadioButton *controlsLeft_ = nullptr;
    QRadioButton *controlsRight_ = nullptr;
    QCheckBox *notificationEnabled_ = nullptr;
    QSpinBox *notificationDuration_ = nullptr;
    QComboBox *notificationPosition_ = nullptr;
    QComboBox *notificationAnimation_ = nullptr;
    QSpinBox *listenPort_ = nullptr;
    QLineEdit *targetHost_ = nullptr;
    QSpinBox *targetPort_ = nullptr;
    QCheckBox *autoStart_ = nullptr;
    QCheckBox *webEnabled_ = nullptr;
    QSpinBox *webPort_ = nullptr;
};
