#include "Tools.h"

#include "Translator.h"

#include <QAbstractItemView>
#include <QComboBox>
#include <QDialogButtonBox>
#include <QFont>
#include <QFontDatabase>
#include <QHBoxLayout>
#include <QHeaderView>
#include <QLabel>
#include <QLineEdit>
#include <QMessageBox>
#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QSettings>
#include <QSplitter>
#include <QTableWidget>
#include <QTableWidgetItem>
#include <QUrl>
#include <QVBoxLayout>

namespace {

QString translate(const QString &key) {
    return Translator::instance().text(key);
}

QFont fixedFont() {
    QFont font = QFontDatabase::systemFont(QFontDatabase::FixedFont);
    font.setPointSize(11);
    return font;
}

QPlainTextEdit *editor(QWidget *parent, const QString &placeholder) {
    auto *view = new QPlainTextEdit(parent);
    view->setFont(fixedFont());
    view->setPlaceholderText(placeholder);
    return view;
}

QPushButton *primaryButton(const QString &text, QWidget *parent) {
    auto *button = new QPushButton(text, parent);
    button->setObjectName(QStringLiteral("primaryButton"));
    button->setCursor(Qt::PointingHandCursor);
    return button;
}

void addCloseButton(QVBoxLayout *layout, QDialog *dialog) {
    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Close, dialog);
    buttons->button(QDialogButtonBox::Close)->setText(translate("common.close"));
    QObject::connect(buttons, &QDialogButtonBox::rejected, dialog, &QDialog::reject);
    layout->addWidget(buttons);
}

QTableWidget *resultTable(QWidget *parent, const QStringList &headers) {
    auto *table = new QTableWidget(parent);
    table->setColumnCount(headers.size());
    table->setHorizontalHeaderLabels(headers);
    table->setEditTriggers(QAbstractItemView::NoEditTriggers);
    table->setSelectionBehavior(QAbstractItemView::SelectRows);
    table->setAlternatingRowColors(true);
    table->setShowGrid(false);
    table->verticalHeader()->setVisible(false);
    table->horizontalHeader()->setSectionResizeMode(QHeaderView::Stretch);
    return table;
}

QString httpStatus(QNetworkReply *reply) {
    const int code = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
    if (code > 0) {
        return QStringLiteral("HTTP %1").arg(code);
    }
    return reply->errorString();
}

} // namespace

RepeaterDialog::RepeaterDialog(QWidget *parent) : QDialog(parent) {
    setWindowTitle(translate("tools.repeater.title"));
    resize(820, 620);

    auto *layout = new QVBoxLayout(this);
    auto *top = new QHBoxLayout();
    method_ = new QComboBox(this);
    method_->addItems({QStringLiteral("GET"), QStringLiteral("POST"), QStringLiteral("PUT"),
                       QStringLiteral("DELETE")});
    method_->setMinimumWidth(90);
    url_ = new QLineEdit(this);
    url_->setPlaceholderText(translate("tools.repeater.urlPlaceholder"));
    send_ = primaryButton(translate("tools.repeater.send"), this);
    connect(send_, &QPushButton::clicked, this, &RepeaterDialog::sendRequest);
    top->addWidget(method_);
    top->addWidget(url_, 1);
    top->addWidget(send_);
    layout->addLayout(top);

    auto *splitter = new QSplitter(Qt::Vertical, this);
    auto *request = new QWidget(splitter);
    auto *requestLayout = new QVBoxLayout(request);
    requestLayout->setContentsMargins(0, 0, 0, 0);
    auto *headersLabel = new QLabel(translate("tools.repeater.headers"), request);
    headersLabel->setObjectName(QStringLiteral("sectionTitle"));
    headers_ = editor(request, translate("tools.repeater.headersPlaceholder"));
    headers_->setMaximumHeight(90);
    auto *bodyLabel = new QLabel(translate("tools.repeater.body"), request);
    bodyLabel->setObjectName(QStringLiteral("sectionTitle"));
    body_ = editor(request, translate("tools.repeater.bodyPlaceholder"));
    requestLayout->addWidget(headersLabel);
    requestLayout->addWidget(headers_);
    requestLayout->addWidget(bodyLabel);
    requestLayout->addWidget(body_, 1);

    auto *responsePanel = new QWidget(splitter);
    auto *responseLayout = new QVBoxLayout(responsePanel);
    responseLayout->setContentsMargins(0, 0, 0, 0);
    status_ = new QLabel(translate("tools.repeater.ready"), responsePanel);
    status_->setObjectName(QStringLiteral("muted"));
    response_ = editor(responsePanel, translate("tools.repeater.responsePlaceholder"));
    response_->setReadOnly(true);
    responseLayout->addWidget(status_);
    responseLayout->addWidget(response_, 1);

    splitter->addWidget(request);
    splitter->addWidget(responsePanel);
    splitter->setStretchFactor(0, 1);
    splitter->setStretchFactor(1, 2);
    layout->addWidget(splitter, 1);
    addCloseButton(layout, this);

    manager_ = new QNetworkAccessManager(this);
    connect(manager_, &QNetworkAccessManager::finished, this, &RepeaterDialog::onFinished);
}

void RepeaterDialog::sendRequest() {
    const QUrl url = QUrl::fromUserInput(url_->text().trimmed());
    if (!url.isValid() || url.scheme().isEmpty()) {
        status_->setText(translate("tools.invalidUrl"));
        return;
    }

    QNetworkRequest request(url);
    request.setHeader(QNetworkRequest::UserAgentHeader, QStringLiteral("NetTool/0.1"));
    const QStringList headerLines = headers_->toPlainText().split(QLatin1Char('\n'));
    for (const QString &line : headerLines) {
        const int separator = line.indexOf(QLatin1Char(':'));
        if (separator <= 0) {
            continue;
        }
        const QByteArray name = line.left(separator).trimmed().toUtf8();
        const QByteArray value = line.mid(separator + 1).trimmed().toUtf8();
        if (!name.isEmpty()) {
            request.setRawHeader(name, value);
        }
    }

    const QByteArray payload = body_->toPlainText().toUtf8();
    const QString method = method_->currentText();
    if (method == QStringLiteral("GET")) {
        manager_->get(request);
    } else if (method == QStringLiteral("POST")) {
        manager_->post(request, payload);
    } else if (method == QStringLiteral("PUT")) {
        manager_->sendCustomRequest(request, "PUT", payload);
    } else {
        manager_->sendCustomRequest(request, "DELETE", payload);
    }
    send_->setEnabled(false);
    status_->setText(translate("tools.sending"));
}

void RepeaterDialog::onFinished(QNetworkReply *reply) {
    const QByteArray body = reply->readAll();
    QString text = httpStatus(reply) + QStringLiteral("\n\n");
    for (const auto &pair : reply->rawHeaderPairs()) {
        text += QString::fromUtf8(pair.first) + QStringLiteral(": ") +
                QString::fromUtf8(pair.second) + QLatin1Char('\n');
    }
    text += QLatin1Char('\n') + QString::fromUtf8(body);
    response_->setPlainText(text);
    status_->setText(httpStatus(reply) + QStringLiteral(" · %1 B").arg(body.size()));
    send_->setEnabled(true);
    reply->deleteLater();
}

RuleManagerDialog::RuleManagerDialog(QWidget *parent) : QDialog(parent) {
    setWindowTitle(translate("tools.rules.title"));
    resize(780, 520);

    auto *layout = new QVBoxLayout(this);
    table_ = new QTableWidget(this);
    table_->setColumnCount(3);
    table_->setHorizontalHeaderLabels(
        {translate("tools.rules.name"), translate("tools.rules.expression"), translate("tools.rules.action")});
    table_->horizontalHeader()->setSectionResizeMode(QHeaderView::Stretch);
    table_->verticalHeader()->setVisible(false);
    table_->setAlternatingRowColors(true);
    table_->setShowGrid(false);
    layout->addWidget(table_, 1);

    auto *actions = new QHBoxLayout();
    auto *add = new QPushButton(translate("tools.rules.add"), this);
    auto *remove = new QPushButton(translate("tools.rules.remove"), this);
    auto *save = primaryButton(translate("button.save"), this);
    auto *close = new QPushButton(translate("common.close"), this);
    connect(add, &QPushButton::clicked, this, &RuleManagerDialog::addRule);
    connect(remove, &QPushButton::clicked, this, &RuleManagerDialog::removeRule);
    connect(save, &QPushButton::clicked, this, &RuleManagerDialog::saveRules);
    connect(close, &QPushButton::clicked, this, &QDialog::reject);
    actions->addWidget(add);
    actions->addWidget(remove);
    actions->addStretch(1);
    actions->addWidget(save);
    actions->addWidget(close);
    layout->addLayout(actions);

    QSettings settings;
    const QStringList saved = settings.value(QStringLiteral("rules/items")).toStringList();
    for (const QString &line : saved) {
        const QStringList parts = line.split(QLatin1Char('\t'));
        if (parts.size() != 3) {
            continue;
        }
        const int row = table_->rowCount();
        table_->insertRow(row);
        for (int column = 0; column < 3; ++column) {
            table_->setItem(row, column, new QTableWidgetItem(parts.at(column)));
        }
    }
}

void RuleManagerDialog::addRule() {
    const int row = table_->rowCount();
    table_->insertRow(row);
    table_->setItem(row, 0, new QTableWidgetItem(QStringLiteral("Rule %1").arg(ruleCounter_++)));
    table_->setItem(row, 1, new QTableWidgetItem(QStringLiteral("drop proto=http")));
    table_->setItem(row, 2, new QTableWidgetItem(QStringLiteral("drop")));
}

void RuleManagerDialog::removeRule() {
    const int row = table_->currentRow();
    if (row >= 0) {
        table_->removeRow(row);
    }
}

void RuleManagerDialog::saveRules() {
    QStringList saved;
    for (int row = 0; row < table_->rowCount(); ++row) {
        QStringList parts;
        for (int column = 0; column < 3; ++column) {
            const QTableWidgetItem *item = table_->item(row, column);
            parts << (item ? item->text() : QString());
        }
        saved << parts.join(QLatin1Char('\t'));
    }
    QSettings().setValue(QStringLiteral("rules/items"), saved);
    accept();
}

IntruderDialog::IntruderDialog(QWidget *parent) : QDialog(parent) {
    setWindowTitle(translate("tools.intruder.title"));
    resize(780, 600);

    auto *layout = new QVBoxLayout(this);
    auto *urlRow = new QHBoxLayout();
    url_ = new QLineEdit(this);
    url_->setPlaceholderText(translate("tools.intruder.urlPlaceholder"));
    start_ = primaryButton(translate("tools.intruder.start"), this);
    connect(start_, &QPushButton::clicked, this, &IntruderDialog::start);
    urlRow->addWidget(url_, 1);
    urlRow->addWidget(start_);
    layout->addLayout(urlRow);

    auto *payloadLabel = new QLabel(translate("tools.intruder.payloads"), this);
    payloadLabel->setObjectName(QStringLiteral("sectionTitle"));
    payloads_ = editor(this, translate("tools.intruder.payloadsPlaceholder"));
    payloads_->setMaximumHeight(110);
    layout->addWidget(payloadLabel);
    layout->addWidget(payloads_);

    results_ = resultTable(this, {translate("tools.intruder.payload"), translate("tools.intruder.status"),
                                  translate("tools.intruder.length")});
    layout->addWidget(results_, 1);
    status_ = new QLabel(translate("tools.intruder.ready"), this);
    status_->setObjectName(QStringLiteral("muted"));
    layout->addWidget(status_);
    addCloseButton(layout, this);

    manager_ = new QNetworkAccessManager(this);
}

void IntruderDialog::start() {
    if (running_) {
        return;
    }
    if (!url_->text().contains(QStringLiteral("FUZZ"))) {
        status_->setText(translate("tools.intruder.marker"));
        return;
    }
    queue_ = payloads_->toPlainText().split(QLatin1Char('\n'), Qt::SkipEmptyParts);
    for (QString &payload : queue_) {
        payload = payload.trimmed();
    }
    queue_.removeAll(QString());
    if (queue_.isEmpty()) {
        status_->setText(translate("tools.intruder.noPayloads"));
        return;
    }
    results_->setRowCount(0);
    index_ = 0;
    running_ = true;
    start_->setEnabled(false);
    sendNext();
}

void IntruderDialog::sendNext() {
    if (index_ >= queue_.size()) {
        finishRun();
        return;
    }
    const QString payload = queue_.at(index_++);
    const QString encoded = QString::fromUtf8(QUrl::toPercentEncoding(payload));
    QNetworkRequest request(QUrl::fromUserInput(url_->text().replace(QStringLiteral("FUZZ"), encoded)));
    request.setHeader(QNetworkRequest::UserAgentHeader, QStringLiteral("NetTool/0.1"));
    QNetworkReply *reply = manager_->get(request);
    status_->setText(translate("tools.intruder.progress").arg(index_).arg(queue_.size()));
    connect(reply, &QNetworkReply::finished, this, [this, reply, payload] {
        const int row = results_->rowCount();
        results_->insertRow(row);
        results_->setItem(row, 0, new QTableWidgetItem(payload));
        results_->setItem(row, 1, new QTableWidgetItem(httpStatus(reply)));
        results_->setItem(row, 2, new QTableWidgetItem(QString::number(reply->readAll().size())));
        reply->deleteLater();
        sendNext();
    });
}

void IntruderDialog::finishRun() {
    running_ = false;
    start_->setEnabled(true);
    status_->setText(translate("tools.intruder.done").arg(queue_.size()));
}

ScannerDialog::ScannerDialog(QWidget *parent) : QDialog(parent) {
    setWindowTitle(translate("tools.scanner.title"));
    resize(760, 580);

    auto *layout = new QVBoxLayout(this);
    auto *urlRow = new QHBoxLayout();
    baseUrl_ = new QLineEdit(this);
    baseUrl_->setPlaceholderText(translate("tools.scanner.basePlaceholder"));
    start_ = primaryButton(translate("tools.scanner.start"), this);
    connect(start_, &QPushButton::clicked, this, &ScannerDialog::start);
    urlRow->addWidget(baseUrl_, 1);
    urlRow->addWidget(start_);
    layout->addLayout(urlRow);

    auto *pathsLabel = new QLabel(translate("tools.scanner.paths"), this);
    pathsLabel->setObjectName(QStringLiteral("sectionTitle"));
    paths_ = editor(this, translate("tools.scanner.pathsPlaceholder"));
    paths_->setMaximumHeight(110);
    layout->addWidget(pathsLabel);
    layout->addWidget(paths_);

    results_ = resultTable(this, {translate("tools.scanner.path"), translate("tools.scanner.status"),
                                  translate("tools.scanner.length")});
    layout->addWidget(results_, 1);
    status_ = new QLabel(translate("tools.scanner.ready"), this);
    status_->setObjectName(QStringLiteral("muted"));
    layout->addWidget(status_);
    addCloseButton(layout, this);

    manager_ = new QNetworkAccessManager(this);
}

void ScannerDialog::start() {
    if (running_) {
        return;
    }
    const QUrl base = QUrl::fromUserInput(baseUrl_->text().trimmed());
    if (!base.isValid() || base.scheme().isEmpty()) {
        status_->setText(translate("tools.invalidUrl"));
        return;
    }
    queue_ = paths_->toPlainText().split(QLatin1Char('\n'), Qt::SkipEmptyParts);
    for (QString &path : queue_) {
        path = path.trimmed();
    }
    queue_.removeAll(QString());
    if (queue_.isEmpty()) {
        status_->setText(translate("tools.scanner.noPaths"));
        return;
    }
    results_->setRowCount(0);
    index_ = 0;
    running_ = true;
    start_->setEnabled(false);
    sendNext();
}

void ScannerDialog::sendNext() {
    if (index_ >= queue_.size()) {
        finishRun();
        return;
    }
    const QString path = queue_.at(index_++);
    QUrl base = QUrl::fromUserInput(baseUrl_->text().trimmed());
    QUrl target = base.resolved(
        QUrl(path.startsWith(QLatin1Char('/')) ? path : QString(QLatin1Char('/')) + path));
    QNetworkRequest request(target);
    request.setHeader(QNetworkRequest::UserAgentHeader, QStringLiteral("NetTool/0.1"));
    QNetworkReply *reply = manager_->get(request);
    status_->setText(translate("tools.scanner.progress").arg(index_).arg(queue_.size()));
    connect(reply, &QNetworkReply::finished, this, [this, reply, path] {
        const int row = results_->rowCount();
        results_->insertRow(row);
        results_->setItem(row, 0, new QTableWidgetItem(path));
        results_->setItem(row, 1, new QTableWidgetItem(httpStatus(reply)));
        results_->setItem(row, 2, new QTableWidgetItem(QString::number(reply->readAll().size())));
        reply->deleteLater();
        sendNext();
    });
}

void ScannerDialog::finishRun() {
    running_ = false;
    start_->setEnabled(true);
    status_->setText(translate("tools.scanner.done").arg(queue_.size()));
}
