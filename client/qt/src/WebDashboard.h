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

#include <QObject>
#include <QStringList>

class QTcpServer;
class QTcpSocket;

class WebDashboardServer : public QObject {
public:
    explicit WebDashboardServer(QObject *parent = nullptr);

    bool start(quint16 port);
    void stop();
    bool isRunning() const;
    QString url() const;
    void setSnapshot(bool proxyRunning, int sessions, int packets, const QStringList &rows);

private:
    void handleConnections();
    void handleRequest(QTcpSocket *socket, const QByteArray &request);
    QString html() const;
    QByteArray json() const;

    QTcpServer *server_ = nullptr;
    bool proxyRunning_ = false;
    int sessions_ = 0;
    int packets_ = 0;
    QStringList rows_;
};
