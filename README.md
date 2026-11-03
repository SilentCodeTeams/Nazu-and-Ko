# NetTool

**Part of [SilentStudio](https://github.com/Silent-Studio-CN)**

**English** | **中文**

NetTool is the Rust core for network protocol analysis and authorized security
testing. It replaces the Python NetTool prototype with a dependency-free,
cross-platform implementation.

NetTool 是网络协议分析与授权安全测试的 Rust 核心，用零外部依赖、跨平台的
实现替代 Python 原型。

## Status / 状态

English:

- Phase 1 foundation is complete: protocol registry, transparent proxy, real
  pcap, PacketEvent/EventSink, control types.
- Protocol dissectors: TLS/SSL, HTTP, DNS, QUIC, WebSocket, Minecraft.
- 44 tests; fmt and clippy clean; CI green on macOS, Windows and Linux.
- Not yet: daemon API and Qt client, privileged MAC helper, TLS MITM,
  HTTP/2 HPACK, HTTP/3 over QUIC, Minecraft compression, plugin runtime.

中文：

- Phase 1 地基完成：协议注册表、透明代理、真实 pcap、PacketEvent/EventSink、
  控制类型。
- 协议解析：TLS/SSL、HTTP、DNS、QUIC、WebSocket、Minecraft。
- 44 个测试；fmt/clippy 通过；macOS、Windows、Linux 三平台 CI 全绿。
- 尚未完成：daemon API 与 Qt 客户端、特权 MAC helper、TLS MITM、
  HTTP/2 HPACK、HTTP/3、Minecraft 压缩、插件运行时。

## Protocol support / 协议支持

| Protocol | Versions detected | Parsed |
| --- | --- | --- |
| TLS/SSL | SSL 3.0, TLS 1.0, 1.1, 1.2, 1.3 | record, ClientHello/ServerHello, SNI, ALPN, cipher suites |
| HTTP | 0.9, 1.0, 1.1, 2 (preface) | start line, headers, WebSocket upgrade |
| DNS | standard query | question name, qtype, qclass |
| QUIC | v1, v2, draft-29 | long header, version, DCID/SCID |
| WebSocket | RFC 6455 | upgrade handshake |
| Minecraft | handshake protocol version | VarInt framing, packet id |

HTTP/3 is detected only as QUIC. Full HTTP/3 needs a QUIC stack and the TLS 1.3
key schedule; that is a later phase. HTTP/3 目前只做 QUIC 检测，完整实现需要
QUIC 栈和 TLS 1.3 密钥调度。

## Build / 构建

From the repository root / 在仓库根目录：

    cargo test --workspace
    cargo build --release

Quality gates / 质量门禁：

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

## CLI / 命令行

    ./target/release/nettoold proxy \
      --listen 127.0.0.1:25565 \
      --target 127.0.0.1:25566 \
      --protocol auto \
      --pcap capture.pcap

    ./target/release/nettoold proxy --duration 10 ...

    ./target/release/nettoold analyze capture.pcap --hex --dissect

| Option | Default | Description |
| --- | --- | --- |
| --listen | 127.0.0.1:25565 | Local listen address |
| --target | 127.0.0.1:25566 | Upstream target |
| --protocol | auto | auto, mc, http, tls, dns, ws, quic, raw |
| --pcap | none | Write a real pcap |
| --duration | none | Stop accepting after N seconds |
| --drop-direction | none | Drop c2s or s2c; still captured |
| --quiet | off | Suppress per-packet output |

Example output / 示例输出：

    [conn 127.0.0.1:52950] tls TLS 1.3 sni=mc.example alpn=h2,http/1.1 ciphers=1
    #1 C->S 98B tls
    #2 S->C 98B tls

## Layout / 目录

    crates/nettool-proto     VarInt, MC framing, registry, TLS/HTTP/DNS/QUIC
    crates/nettool-capture   pcap writer/reader, synthetic TCP frames
    crates/nettool-proxy     TCP relay, EventSink fan-out, direction drops
    crates/nettool-control   control types, session and MAC lease state machine
    crates/nettoold          CLI entry point, daemon next
    docs/PROTOCOL.md         wire formats, versions and proxy semantics
    docs/ARCHITECTURE.md     crate graph, detection flow, extension points
    docs/PROCESS-MODEL.md    UI / daemon / privileged helper, crash isolation
    docs/CONTROL-API.md      daemon API message model
    docs/MIGRATION.md        Python -> Rust mapping and phases
    docs/PLUGIN-ABI.md       C ABI design for DLL/dylib plugins
    docs/AUTHORIZED-USE.md   allowed and prohibited use
    SECURITY.md              vulnerability reporting and safety boundary

## Design / 设计

English:

- Rust-first. C is reserved for OS shims, not for parsing untrusted bytes.
- UI and backend are separate processes: Qt 6 client -> nettoold daemon ->
  privileged helper. A UI crash does not stop capture or MAC leases.
- MAC identity is owned by a privileged helper with explicit leases, audit and
  restore. Leases end only on explicit release or helper shutdown.
- Raw bytes are forwarded immediately; framing never delays traffic.
- Protocol detection never silently changes the target port.
- Multi-version parsing is metadata-first. No hand-written cryptography.

中文：

- Rust 优先；C 只用于操作系统薄垫片，不用于解析不可信输入。
- UI 与后台分离：Qt 6 客户端 -> nettoold daemon -> 特权 helper。UI 崩溃不会
  停止抓包或释放 MAC 租约。
- MAC 由独立特权 helper 管理：显式租约、审计、可恢复；只有显式 release 或
  helper shutdown 才结束。
- 原始字节立即转发；分帧只做展示，不延迟转发。
- 协议检测不会偷偷切换目标端口。
- 多版本解析以元数据为先；不手写密码学。

## Security / 安全

NetTool is for systems you own or are explicitly authorized to test. It does not
ship stealth, evasion or unauthorized remote-control features. See SECURITY.md
and docs/AUTHORIZED-USE.md.

NetTool 仅用于自有系统或获得书面授权的测试，不提供隐蔽、规避监控或未经授权
的远程控制能力。详见 SECURITY.md 和 docs/AUTHORIZED-USE.md。

## License / 许可证

Apache-2.0. See LICENSE.
