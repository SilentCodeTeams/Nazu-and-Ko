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

#include "WebDashboard.h"

#include "Translator.h"

#include <QHostAddress>
#include <QList>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QTcpServer>
#include <QTcpSocket>

namespace {

QString translate(const QString &key) {
    return Translator::instance().text(key);
}

QString escape(const QString &value) {
    QString result = value;
    result.replace(QLatin1Char('&'), QStringLiteral("&amp;"));
    result.replace(QLatin1Char('<'), QStringLiteral("&lt;"));
    result.replace(QLatin1Char('>'), QStringLiteral("&gt;"));
    result.replace(QLatin1Char('"'), QStringLiteral("&quot;"));
    return result;
}

} // namespace

WebDashboardServer::WebDashboardServer(QObject *parent) : QObject(parent) {}

bool WebDashboardServer::start(quint16 port) {
    if (server_) {
        return true;
    }
    server_ = new QTcpServer(this);
    if (!server_->listen(QHostAddress::LocalHost, port)) {
        delete server_;
        server_ = nullptr;
        return false;
    }
    connect(server_, &QTcpServer::newConnection, this, &WebDashboardServer::handleConnections);
    return true;
}

void WebDashboardServer::stop() {
    if (!server_) {
        return;
    }
    server_->close();
    server_->deleteLater();
    server_ = nullptr;
}

bool WebDashboardServer::isRunning() const {
    return server_ != nullptr && server_->isListening();
}

QString WebDashboardServer::url() const {
    if (!isRunning()) {
        return QString();
    }
    return QStringLiteral("http://127.0.0.1:%1").arg(server_->serverPort());
}

void WebDashboardServer::setSnapshot(bool proxyRunning, int sessions, int packets,
                                     const QStringList &rows) {
    proxyRunning_ = proxyRunning;
    sessions_ = sessions;
    packets_ = packets;
    rows_ = rows;
}

void WebDashboardServer::handleConnections() {
    while (server_ && server_->hasPendingConnections()) {
        QTcpSocket *socket = server_->nextPendingConnection();
        connect(socket, &QTcpSocket::readyRead, this, [this, socket] {
            const QByteArray request = socket->readAll();
            if (request.contains("\r\n")) {
                handleRequest(socket, request);
            }
        });
        connect(socket, &QTcpSocket::disconnected, socket, &QObject::deleteLater);
    }
}

void WebDashboardServer::handleRequest(QTcpSocket *socket, const QByteArray &request) {
    const QByteArray firstLine = request.left(request.indexOf("\r\n"));
    const QList<QByteArray> parts = firstLine.split(' ');
    const QString path = parts.size() >= 2 ? QString::fromUtf8(parts.at(1)) : QStringLiteral("/");

    QByteArray response;
    if (path.startsWith(QStringLiteral("/api/status"))) {
        response = json();
        socket->write("HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\n"
                      "Connection: close\r\nCache-Control: no-store\r\n\r\n");
    } else if (path == QStringLiteral("/") || path.startsWith(QStringLiteral("/?"))) {
        response = html().toUtf8();
        socket->write("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n"
                      "Connection: close\r\nCache-Control: no-store\r\n\r\n");
    } else {
        response = "not found";
        socket->write("HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n"
                      "Connection: close\r\n\r\n");
    }
    socket->write(response);
    socket->disconnectFromHost();
}

QByteArray WebDashboardServer::json() const {
    QJsonObject root;
    root.insert(QStringLiteral("proxyRunning"), proxyRunning_);
    root.insert(QStringLiteral("sessions"), sessions_);
    root.insert(QStringLiteral("packets"), packets_);
    QJsonArray rows;
    for (const QString &row : rows_) {
        const QStringList cells = row.split(QLatin1Char('\t'));
        QJsonObject item;
        item.insert(QStringLiteral("id"), cells.value(0));
        item.insert(QStringLiteral("time"), cells.value(1));
        item.insert(QStringLiteral("direction"), cells.value(2));
        item.insert(QStringLiteral("length"), cells.value(3));
        item.insert(QStringLiteral("status"), cells.value(4));
        item.insert(QStringLiteral("summary"), cells.value(5));
        rows.append(item);
    }
    root.insert(QStringLiteral("recent"), rows);
    return QJsonDocument(root).toJson(QJsonDocument::Compact);
}

QString WebDashboardServer::html() const {
    QString tableRows;
    for (const QString &row : rows_) {
        const QStringList cells = row.split(QLatin1Char('\t'));
        tableRows += QStringLiteral("<tr><td>%1</td><td>%2</td><td>%3</td><td>%4</td><td>%5</td><td>%6</td></tr>")
                         .arg(escape(cells.value(0)), escape(cells.value(1)), escape(cells.value(2)),
                              escape(cells.value(3)), escape(cells.value(4)), escape(cells.value(5)));
    }
    if (tableRows.isEmpty()) {
        tableRows = QStringLiteral("<tr><td colspan=\"6\" class=\"empty\">%1</td></tr>")
                        .arg(escape(translate("web.empty")));
    }
    const QString state = proxyRunning_ ? translate("web.running") : translate("web.offline");
    return QStringLiteral(R"HTML(<!doctype html>
<html lang="zh-CN"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>%1</title>
<style>
body{margin:0;background:#0b0f14;color:#e6eaf0;font:14px -apple-system,BlinkMacSystemFont,sans-serif}
main{max-width:1100px;margin:0 auto;padding:32px 24px}
h1{font-size:24px;margin:0 0 6px}p{color:#8b96a7;margin:0 0 24px}
.cards{display:grid;grid-template-columns:repeat(3,1fr);gap:12px;margin-bottom:20px}
.card{background:#11161f;border:1px solid #232b36;border-radius:12px;padding:16px}
.label{color:#8b96a7;font-size:11px;text-transform:uppercase;letter-spacing:.08em}
.value{font-size:26px;font-weight:700;margin-top:6px}
table{width:100%;border-collapse:collapse;background:#11161f;border:1px solid #232b36;border-radius:12px;overflow:hidden}
th,td{text-align:left;padding:9px 10px;border-bottom:1px solid #232b36;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
th{color:#8b96a7;font-size:11px;text-transform:uppercase;letter-spacing:.06em}
td{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:12px}
.empty{color:#8b96a7;text-align:center;padding:24px}
footer{color:#64748b;font-size:11px;margin-top:18px}
</style></head><body><main>
<h1>%1</h1><p>%2</p>
<div class="cards">
<div class="card"><div class="label">%3</div><div class="value">%4</div></div>
<div class="card"><div class="label">%5</div><div class="value">%6</div></div>
<div class="card"><div class="label">%7</div><div class="value">%8</div></div>
</div>
<table><thead><tr><th>%9</th><th>%10</th><th>%11</th><th>%12</th><th>%13</th><th>%14</th></tr></thead>
<tbody>%15</tbody></table>
<footer>%16</footer>
</main></body></html>)HTML")
        .arg(escape(translate("web.title")), escape(translate("web.subtitle")),
             escape(translate("web.proxy")), escape(state),
             escape(translate("web.sessions")), QString::number(sessions_),
             escape(translate("web.packets")), QString::number(packets_),
             escape(translate("table.id")), escape(translate("table.time")),
             escape(translate("table.direction")), escape(translate("table.length")),
             escape(translate("table.status")), escape(translate("table.summary")),
             tableRows, escape(translate("web.footer")));
}
