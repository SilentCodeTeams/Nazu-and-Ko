#pragma once

#include <QByteArray>
#include <QWidget>

class QPlainTextEdit;
class QPushButton;
class QTabWidget;

class PacketEditor : public QWidget {
    Q_OBJECT

public:
    explicit PacketEditor(QWidget *parent = nullptr);

    void setData(const QByteArray &data);
    void setPlaceholderText(const QString &text);
    void clear();
    QByteArray data() const;

private:
    void copyCurrent();
    void prettyJson();
    void updateViews();

    QTabWidget *tabs_ = nullptr;
    QPlainTextEdit *raw_ = nullptr;
    QPlainTextEdit *hex_ = nullptr;
    QPlainTextEdit *json_ = nullptr;
    QPlainTextEdit *http_ = nullptr;
    QPushButton *copy_ = nullptr;
    QPushButton *pretty_ = nullptr;
    QByteArray data_;
};
