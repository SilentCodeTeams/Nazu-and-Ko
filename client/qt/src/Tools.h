#pragma once

#include <QDialog>
#include <QStringList>

class QCheckBox;
class QComboBox;
class QLabel;
class QLineEdit;
class QNetworkAccessManager;
class QNetworkReply;
class QPlainTextEdit;
class QPushButton;
class QSpinBox;
class QTableWidget;

class RepeaterDialog : public QDialog {
    Q_OBJECT

public:
    explicit RepeaterDialog(QWidget *parent = nullptr);

private:
    void sendRequest();
    void onFinished(QNetworkReply *reply);

    QComboBox *method_ = nullptr;
    QLineEdit *url_ = nullptr;
    QPlainTextEdit *headers_ = nullptr;
    QPlainTextEdit *body_ = nullptr;
    QPlainTextEdit *response_ = nullptr;
    QLabel *status_ = nullptr;
    QPushButton *send_ = nullptr;
    QCheckBox *followRedirects_ = nullptr;
    QSpinBox *timeout_ = nullptr;
    QNetworkAccessManager *manager_ = nullptr;
};

class RuleManagerDialog : public QDialog {
    Q_OBJECT

public:
    explicit RuleManagerDialog(QWidget *parent = nullptr);

private:
    void addRule();
    void removeRule();
    void saveRules();

    QTableWidget *table_ = nullptr;
    int ruleCounter_ = 1;
};

class IntruderDialog : public QDialog {
    Q_OBJECT

public:
    explicit IntruderDialog(QWidget *parent = nullptr);

private:
    void start();
    void sendNext();
    void finishRun();
    void stopRun();

    QLineEdit *url_ = nullptr;
    QPlainTextEdit *payloads_ = nullptr;
    QTableWidget *results_ = nullptr;
    QLabel *status_ = nullptr;
    QPushButton *start_ = nullptr;
    QPushButton *stop_ = nullptr;
    QSpinBox *delay_ = nullptr;
    QNetworkAccessManager *manager_ = nullptr;
    QStringList queue_;
    int index_ = 0;
    bool running_ = false;
    bool stopRequested_ = false;
};

class ScannerDialog : public QDialog {
    Q_OBJECT

public:
    explicit ScannerDialog(QWidget *parent = nullptr);

private:
    void start();
    void sendNext();
    void finishRun();

    QLineEdit *baseUrl_ = nullptr;
    QPlainTextEdit *paths_ = nullptr;
    QTableWidget *results_ = nullptr;
    QLabel *status_ = nullptr;
    QPushButton *start_ = nullptr;
    QNetworkAccessManager *manager_ = nullptr;
    QStringList queue_;
    int index_ = 0;
    bool running_ = false;
};
