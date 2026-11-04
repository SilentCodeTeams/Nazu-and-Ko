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
