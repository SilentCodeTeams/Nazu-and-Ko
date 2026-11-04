#pragma once

#include <QList>
#include <QObject>

class QWidget;

class NotificationCenter : public QObject {
public:
    enum class Level {
        Info,
        Success,
        Error,
    };

    explicit NotificationCenter(QObject *parent = nullptr);
    void show(const QString &message, Level level = Level::Info);

private:
    void reposition();
    QList<QWidget *> active_;
};
