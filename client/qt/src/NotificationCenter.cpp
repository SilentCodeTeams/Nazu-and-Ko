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

#include "NotificationCenter.h"

#include <algorithm>

#include <QEasingCurve>
#include <QFrame>
#include <QGraphicsOpacityEffect>
#include <QGuiApplication>
#include <QHBoxLayout>
#include <QLabel>
#include <QPropertyAnimation>
#include <QScreen>
#include <QSettings>
#include <QTimer>
#include <QWidget>

NotificationCenter::NotificationCenter(QObject *parent) : QObject(parent) {}

void NotificationCenter::show(const QString &message, Level level) {
    QSettings settings;
    if (!settings.value(QStringLiteral("notification/enabled"), true).toBool()) {
        return;
    }
    if (active_.size() >= 4) {
        active_.first()->close();
    }

    auto *toast = new QFrame;
    toast->setObjectName(QStringLiteral("toast"));
    toast->setWindowFlags(Qt::Tool | Qt::FramelessWindowHint | Qt::WindowStaysOnTopHint);
    toast->setAttribute(Qt::WA_ShowWithoutActivating);
    toast->setAttribute(Qt::WA_DeleteOnClose);
    toast->setMaximumWidth(380);

    QString background = QStringLiteral("#1F2937");
    if (level == Level::Success) {
        background = QStringLiteral("#111111");
    } else if (level == Level::Error) {
        background = QStringLiteral("#B42318");
    }
    toast->setStyleSheet(QStringLiteral(
        "QFrame#toast { background: %1; border-radius: 10px; }"
        "QLabel { color: #FFFFFF; font-size: 12px; }")
                             .arg(background));

    auto *layout = new QHBoxLayout(toast);
    layout->setContentsMargins(14, 10, 14, 10);
    auto *label = new QLabel(message, toast);
    label->setWordWrap(true);
    layout->addWidget(label);

    auto *effect = new QGraphicsOpacityEffect(toast);
    effect->setOpacity(0.0);
    toast->setGraphicsEffect(effect);

    active_.append(toast);
    toast->adjustSize();
    reposition();

    const QString animation = settings.value(QStringLiteral("notification/animation"),
                                              QStringLiteral("fade")).toString();
    const QPoint finalPosition = toast->pos();
    QPoint startPosition = finalPosition;
    if (animation == QStringLiteral("slide") || animation == QStringLiteral("bounce")) {
        startPosition += QPoint(24, 0);
    } else if (animation == QStringLiteral("pop")) {
        startPosition += QPoint(0, 8);
    }
    toast->move(startPosition);
    toast->show();
    toast->raise();

    auto *fade = new QPropertyAnimation(effect, "opacity", toast);
    fade->setDuration(180);
    fade->setStartValue(0.0);
    fade->setEndValue(1.0);
    fade->setEasingCurve(animation == QStringLiteral("bounce") ? QEasingCurve::OutBounce
                                                               : QEasingCurve::OutCubic);
    fade->start(QAbstractAnimation::DeleteWhenStopped);

    if (startPosition != finalPosition) {
        auto *move = new QPropertyAnimation(toast, "pos", toast);
        move->setDuration(animation == QStringLiteral("bounce") ? 420 : 220);
        move->setStartValue(startPosition);
        move->setEndValue(finalPosition);
        move->setEasingCurve(animation == QStringLiteral("bounce") ? QEasingCurve::OutBounce
                                                                   : QEasingCurve::OutCubic);
        move->start(QAbstractAnimation::DeleteWhenStopped);
    }

    connect(toast, &QObject::destroyed, this, [this, toast] {
        active_.removeAll(toast);
        reposition();
    });
    const int duration = settings.value(QStringLiteral("notification/duration"), 3000).toInt();
    QTimer::singleShot(qBound(500, duration, 15000), toast, &QWidget::close);
}

void NotificationCenter::reposition() {
    QScreen *screen = QGuiApplication::primaryScreen();
    if (!screen) {
        return;
    }
    const QRect area = screen->availableGeometry();
    QString position = QSettings().value(QStringLiteral("notification/position"),
                                         QStringLiteral("auto")).toString();
    if (position == QStringLiteral("auto")) {
#ifdef Q_OS_DARWIN
        position = QStringLiteral("top-right");
#else
        position = QStringLiteral("bottom-right");
#endif
    }
    const bool top = position.startsWith(QStringLiteral("top"));
    const bool left = position.endsWith(QStringLiteral("left"));

    QList<QWidget *> ordered = active_;
    if (!top) {
        std::reverse(ordered.begin(), ordered.end());
    }
    int cursor = top ? area.top() + 24 : area.bottom() - 24;
    for (QWidget *toast : ordered) {
        toast->adjustSize();
        const int x = left ? area.left() + 24 : area.right() - toast->width() - 24;
        const int y = top ? cursor : cursor - toast->height();
        toast->move(qMax(area.left() + 8, x), qMax(area.top() + 8, y));
        cursor = top ? y + toast->height() + 8 : y - 8;
    }
}
