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

#include "PacketEditor.h"

#include "Translator.h"

#include <QApplication>
#include <QClipboard>
#include <QColor>
#include <QFont>
#include <QFontDatabase>
#include <QList>
#include <QHBoxLayout>
#include <QJsonDocument>
#include <QJsonParseError>
#include <QLabel>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QRegularExpression>
#include <QSyntaxHighlighter>
#include <QTabWidget>
#include <QTextCharFormat>
#include <QTextDocument>
#include <QVBoxLayout>

namespace {

QString translate(const QString &key) {
    return Translator::instance().text(key);
}

QPlainTextEdit *viewer(QWidget *parent, bool readOnly) {
    auto *view = new QPlainTextEdit(parent);
    QFont font = QFontDatabase::systemFont(QFontDatabase::FixedFont);
    font.setPointSize(11);
    view->setFont(font);
    view->setReadOnly(readOnly);
    view->setLineWrapMode(QPlainTextEdit::NoWrap);
    return view;
}

class JsonHighlighter : public QSyntaxHighlighter {
public:
    explicit JsonHighlighter(QTextDocument *document) : QSyntaxHighlighter(document) {
        QTextCharFormat key;
        key.setForeground(QColor(QStringLiteral("#7DD3FC")));
        key.setFontWeight(QFont::DemiBold);
        rules_.append({QRegularExpression(QStringLiteral("\"(?:[^\"\\\\]|\\\\.)*\"(?=\\s*:)")), key});

        QTextCharFormat string;
        string.setForeground(QColor(QStringLiteral("#A7F3D0")));
        rules_.append({QRegularExpression(QStringLiteral("\"(?:[^\"\\\\]|\\\\.)*\"")), string});

        QTextCharFormat number;
        number.setForeground(QColor(QStringLiteral("#FCA5A5")));
        rules_.append({QRegularExpression(QStringLiteral("\\b-?\\d+(?:\\.\\d+)?(?:[eE][+-]?\\d+)?\\b")), number});

        QTextCharFormat literal;
        literal.setForeground(QColor(QStringLiteral("#C4B5FD")));
        rules_.append({QRegularExpression(QStringLiteral("\\b(?:true|false|null)\\b")), literal});
    }

protected:
    void highlightBlock(const QString &text) override {
        for (const auto &rule : rules_) {
            auto iterator = rule.pattern.globalMatch(text);
            while (iterator.hasNext()) {
                const QRegularExpressionMatch match = iterator.next();
                setFormat(match.capturedStart(), match.capturedLength(), rule.format);
            }
        }
    }

private:
    struct Rule {
        QRegularExpression pattern;
        QTextCharFormat format;
    };
    QList<Rule> rules_;
};

class HttpHighlighter : public QSyntaxHighlighter {
public:
    explicit HttpHighlighter(QTextDocument *document) : QSyntaxHighlighter(document) {
        QTextCharFormat start;
        start.setForeground(QColor(QStringLiteral("#7DD3FC")));
        start.setFontWeight(QFont::DemiBold);
        rules_.append({QRegularExpression(QStringLiteral("^(?:GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)\\s+\\S+\\s+HTTP/\\d(?:\\.\\d)?")), start});
        rules_.append({QRegularExpression(QStringLiteral("^HTTP/\\d(?:\\.\\d)?\\s+\\d{3}")), start});

        QTextCharFormat header;
        header.setForeground(QColor(QStringLiteral("#A7F3D0")));
        header.setFontWeight(QFont::DemiBold);
        rules_.append({QRegularExpression(QStringLiteral("^[A-Za-z0-9-]+:")), header});
    }

protected:
    void highlightBlock(const QString &text) override {
        for (const auto &rule : rules_) {
            const auto match = rule.pattern.match(text);
            if (match.hasMatch()) {
                setFormat(match.capturedStart(), match.capturedLength(), rule.format);
            }
        }
    }

private:
    struct Rule {
        QRegularExpression pattern;
        QTextCharFormat format;
    };
    QList<Rule> rules_;
};

QString hexDump(const QByteArray &data) {
    QStringList lines;
    for (int offset = 0; offset < data.size(); offset += 16) {
        const QByteArray chunk = data.mid(offset, 16);
        QString hexPart;
        QString asciiPart;
        for (int index = 0; index < 16; ++index) {
            if (index < chunk.size()) {
                const unsigned char byte = static_cast<unsigned char>(chunk.at(index));
                hexPart += QStringLiteral("%1 ").arg(byte, 2, 16, QLatin1Char('0')).toUpper();
                asciiPart += (byte >= 32 && byte < 127) ? QChar(byte) : QLatin1Char('.');
            } else {
                hexPart += QStringLiteral("   ");
                asciiPart += QLatin1Char(' ');
            }
            if (index == 7) {
                hexPart += QLatin1Char(' ');
            }
        }
        lines << QStringLiteral("%1  %2 |%3|")
                     .arg(offset, 8, 16, QLatin1Char('0'))
                     .arg(hexPart)
                     .arg(asciiPart);
    }
    return lines.join(QLatin1Char('\n'));
}

} // namespace

PacketEditor::PacketEditor(QWidget *parent) : QWidget(parent) {
    auto *layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(6);

    auto *actions = new QHBoxLayout();
    actions->setContentsMargins(0, 0, 0, 0);
    copy_ = new QPushButton(translate("editor.copy"), this);
    pretty_ = new QPushButton(translate("editor.pretty"), this);
    connect(copy_, &QPushButton::clicked, this, &PacketEditor::copyCurrent);
    connect(pretty_, &QPushButton::clicked, this, &PacketEditor::prettyJson);
    actions->addStretch(1);
    actions->addWidget(pretty_);
    actions->addWidget(copy_);
    layout->addLayout(actions);

    tabs_ = new QTabWidget(this);
    raw_ = viewer(tabs_, false);
    hex_ = viewer(tabs_, true);
    json_ = viewer(tabs_, false);
    http_ = viewer(tabs_, true);
    tabs_->addTab(raw_, translate("editor.raw"));
    tabs_->addTab(hex_, translate("editor.hex"));
    tabs_->addTab(json_, translate("editor.json"));
    tabs_->addTab(http_, translate("editor.http"));
    layout->addWidget(tabs_, 1);

    new HttpHighlighter(raw_->document());
    new HttpHighlighter(http_->document());
    new JsonHighlighter(json_->document());
}

void PacketEditor::setData(const QByteArray &data) {
    data_ = data;
    updateViews();
    tabs_->setCurrentIndex(0);
}

void PacketEditor::setPlaceholderText(const QString &text) {
    raw_->setPlaceholderText(text);
    hex_->setPlaceholderText(text);
    json_->setPlaceholderText(text);
    http_->setPlaceholderText(text);
}

void PacketEditor::clear() {
    setData(QByteArray());
    tabs_->setCurrentIndex(0);
}

QByteArray PacketEditor::data() const {
    return raw_->toPlainText().toUtf8();
}

void PacketEditor::updateViews() {
    const QString rawText = QString::fromUtf8(data_);
    raw_->setPlainText(rawText);
    hex_->setPlainText(hexDump(data_));

    QJsonParseError error;
    const QJsonDocument document = QJsonDocument::fromJson(data_, &error);
    if (error.error == QJsonParseError::NoError && (document.isObject() || document.isArray())) {
        json_->setPlainText(QString::fromUtf8(document.toJson(QJsonDocument::Indented)));
    } else {
        json_->setPlainText(rawText);
    }

    http_->setPlainText(rawText);
}

void PacketEditor::copyCurrent() {
    QWidget *current = tabs_->currentWidget();
    if (auto *editor = qobject_cast<QPlainTextEdit *>(current)) {
        QApplication::clipboard()->setText(editor->toPlainText());
    }
}

void PacketEditor::prettyJson() {
    QJsonParseError error;
    const QJsonDocument document = QJsonDocument::fromJson(raw_->toPlainText().toUtf8(), &error);
    if (error.error != QJsonParseError::NoError || (!document.isObject() && !document.isArray())) {
        return;
    }
    const QString pretty = QString::fromUtf8(document.toJson(QJsonDocument::Indented));
    raw_->setPlainText(pretty);
    json_->setPlainText(pretty);
    data_ = pretty.toUtf8();
    tabs_->setCurrentIndex(2);
}
