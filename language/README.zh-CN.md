# Nazu&Ko（简体中文）

> We don't sleep because packets never do.

[English](README.en.md) | 简体中文

Nazu&Ko 是用于网络协议分析与授权安全测试的 Rust 核心。它接替 Python 原型，
做到零外部依赖、跨平台，并且把后台与 UI 客户端拆成独立进程。

## 状态

- Phase 1 地基完成：协议注册表、透明 TCP 代理、真实 pcap、PacketEvent/
  EventSink、控制类型。
- Phase 2 规则引擎完成：按方向、协议、端口、长度和载荷字节 drop/allow，
  按捕获块求值。
- Phase 2 daemon 控制服务器 MVP：loopback TCP、0600 token state file、
  JSON lines 鉴权、dry-run MAC 租约。
- Qt 6 Widgets 客户端：重做 dashboard、key=value 国际化（zh-CN/en）、
  真实 nazuko proxy 输出与数据包行、Repeater/规则/Intruder/Scanner、
  PacketEditor Raw/Hex/JSON/HTTP 标签、通知、智能导出和代理规则接入。
- 协议解析：TLS/SSL、HTTP、DNS、QUIC、WebSocket、Minecraft。
- 62 个测试；rustfmt 与 clippy 通过；macOS、Windows、Linux 三平台 CI 全绿。
- 尚未完成：更深的工具选项对齐、daemon proxy session 命令、特权 MAC helper、
  TLS MITM、HTTP/2 HPACK、HTTP/3、Minecraft 压缩与登录状态、插件运行时。

## 协议支持

| 协议 | 可检测版本 | 已解析 |
| --- | --- | --- |
| TLS/SSL | SSL 3.0、TLS 1.0、1.1、1.2、1.3 | record、ClientHello/ServerHello、SNI、ALPN、cipher suites |
| HTTP | 0.9、1.0、1.1、2（preface） | 请求/响应行、头部、WebSocket Upgrade |
| DNS | 标准查询 | question name、qtype、qclass |
| QUIC | v1、v2、draft-29 | long header、version、DCID/SCID |
| WebSocket | RFC 6455 | upgrade handshake |
| Minecraft | 握手协议版本 | VarInt 分帧、packet id |

HTTP/3 目前只做 QUIC 检测。完整 HTTP/3 需要 QUIC 栈和 TLS 1.3 密钥调度，
属于后续阶段。

## 构建

在仓库根目录执行：

    cargo test --workspace
    cargo build --release

Qt 客户端：

    cmake -S client/qt -B build/qt \
      -DCMAKE_PREFIX_PATH=/Users/apple/Qt/6.11.2/macos
    cmake --build build/qt -j 4

质量门禁：

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

## 命令行

`nazuko` 是唯一安装的用户 CLI。旧 `nettoold` 名称已退役，不再生成同名二进制。当前本地控制服务器通过 `nazuko serve` 启动；如果以后拆成独立 daemon，内部进程名为 `nazukod`，不占用用户 CLI。

    ./target/release/nazuko proxy \
      --listen 127.0.0.1:25565 \
      --target 127.0.0.1:25566 \
      --protocol auto \
      --pcap capture.pcap

    ./target/release/nazuko proxy --duration 10 ...

    ./target/release/nazuko proxy \
      --rule 'drop dir=c2s proto=mc contains=hello' ...

    ./target/release/nazuko serve --state nazuko.state

    ./target/release/nazuko analyze capture.pcap --hex --dissect

| 参数 | 默认值 | 说明 |
| --- | --- | --- |
| --listen | 127.0.0.1:25565 | 本地监听地址 |
| --target | 127.0.0.1:25566 | 上游目标 |
| --protocol | auto | auto、mc、http、tls、dns、ws、quic、raw |
| --pcap | 无 | 写入真实 pcap |
| --duration | 无 | N 秒后停止接收新连接 |
| --drop-direction | 无 | 丢弃 c2s 或 s2c，仍会记录 |
| --rule | 无 | 可重复的 drop/allow 谓词；见 docs/RULES.md |
| --quiet | 关闭 | 不输出逐包日志 |

### 规则

规则按捕获块求值；第一条匹配的规则生效，没有匹配则放行。可重复使用
--rule，前面的 allow 可用于在后面的 drop 之前放行指定流量。

    --rule 'drop proto=http prefix=GET'
    --rule 'drop dir=s2c min_len=4096'

语法、事件字段和安全边界见 docs/RULES.md。

### 控制服务器

    ./target/release/nazuko serve --state nazuko.state

仅监听 loopback。daemon 把 port 与 token 写入 0600 state file，使用 JSON lines
通信。MVP 已实现 list_sessions（空）、acquire_mac dry-run、release_mac、
list_leases 和 shutdown；proxy session 命令下一步接入。

消息模型见 docs/CONTROL-API.md。

示例输出：

    [conn 127.0.0.1:52950] tls TLS 1.3 sni=mc.example alpn=h2,http/1.1 ciphers=1
    #1 C->S 98B tls
    #2 S->C 98B tls

## 目录

    crates/nettool-proto     VarInt、MC 分帧、协议注册表、TLS/HTTP/DNS/QUIC
    crates/nettool-rules     分组谓词与有序规则集
    crates/nettool-capture   pcap 读写、合成 TCP 帧
    crates/nettool-proxy     TCP 转发、EventSink 分发、方向丢弃、规则判定
    crates/nettool-control   控制类型、会话与 MAC 租约状态机
    crates/nazuko          CLI 与本地控制服务器
    client/qt                Qt 6 Widgets 客户端（主窗口移植）
    docs/PROTOCOL.md         线格式、版本与代理语义
    docs/RULES.md            规则语法、语义与安全边界
    docs/ARCHITECTURE.md     架构、检测流程、扩展点
    docs/PROCESS-MODEL.md    UI / daemon / 特权 helper、崩溃隔离
    docs/CONTROL-API.md      daemon API 消息模型
    docs/MIGRATION.md        Python -> Rust 映射与阶段
    docs/ROADMAP.md          产品阶段与平台边界
    docs/PLUGIN-ABI.md       DLL/dylib 插件 C ABI 设计
    docs/AUTHORIZED-USE.md   允许与禁止的使用
    SECURITY.md              漏洞报告与安全边界

## 设计

- Rust 优先；C 只用于操作系统薄垫片，不用于解析不可信输入。
- UI 与后台分离：Qt 6 客户端 -> nazuko daemon -> 特权 helper。UI 崩溃不会
  停止抓包或释放 MAC 租约。
- MAC 由独立特权 helper 管理：显式租约、审计、可恢复；只有显式 release 或
  helper shutdown 才结束。
- 原始字节立即转发；分帧只做展示，不延迟转发。
- 协议检测不会偷偷切换目标端口。
- 多版本解析以元数据为先；不手写密码学。

## 贡献

欢迎提交 issue 和 pull request。提交前请阅读 CONTRIBUTING.md。改动必须聚焦、
带测试，并且不得重写、删除或覆盖其他人的工作。禁止 force-push 到 main。

## 安全

Nazu&Ko 仅用于自有系统或获得书面授权的测试，不提供隐蔽、规避监控或未经授权
的远程控制能力。详见 SECURITY.md 和 docs/AUTHORIZED-USE.md。

## 贡献者（Contributors）

| 头像 | 贡献者 | 角色 |
|------|--------|------|
| <img src="../assets/deepseek-2.svg" width="64" alt="DeepSeek-Work-In-SilentStudio"/> | **DeepSeek-Work-In-SilentStudio** (@DeepSeek-Work-In-SilentStudio) | AI 开发（贡献者身份） |
| | **Silent-xiaomiao** | 项目发起 / 发布（GitHub: [Silent-xiaomiao](https://github.com/Silent-xiaomiao)） |

## 许可证

AGPL-3.0-or-later。见 LICENSE 与 NOTICE。

---

本项目由 SilentStudio 监管与负责。<br>
本项目由 SilentCodeTeams 对法律、版权事宜及相关咨询承担责任。<br>
版权所有 © SilentStudio。<br>
以 AGPL-3.0-or-later 授权。

This project is supervised and administered by SilentStudio.<br>
This project is covered by SilentCodeTeams, which assumes responsibility for legal and copyright matters and related inquiries.<br>
Copyright © SilentStudio.<br>
Licensed under AGPL-3.0-or-later.
