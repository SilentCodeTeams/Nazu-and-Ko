# NetTool ζ

English | [简体中文](language/README.zh-CN.md)

NetTool is the Rust core for network protocol analysis and authorized security
testing. It succeeds the Python NetTool prototype with a dependency-free,
cross-platform implementation and a clean process boundary between the backend
and UI clients.

## Status

- Phase 1 foundation is complete: protocol registry, transparent TCP proxy,
  real pcap capture, PacketEvent/EventSink, control types.
- Phase 2 packet rules: drop/allow by direction, protocol, ports, length and
  payload bytes, evaluated per captured chunk.
- Phase 2 daemon control server MVP: loopback TCP, 0600 token state file,
  JSON-lines authentication and dry-run MAC leases.
- Qt 6 Widgets client: redesigned dashboard, key=value i18n (zh-CN/en), live
  `nettoold proxy` output with packet rows, Repeater/Rules/Intruder/Scanner,
  PacketEditor Raw/Hex/JSON/HTTP tabs, notifications, smart export and proxy
  rule wiring.
- Protocol dissectors: TLS/SSL, HTTP, DNS, QUIC, WebSocket, Minecraft.
- 62 tests; rustfmt and clippy clean; CI green on macOS, Windows and Linux.
- Not yet: full legacy tool-option parity, Qt web dashboard, daemon
  proxy-session commands, privileged MAC helper, TLS MITM, HTTP/2 HPACK,
  HTTP/3 over QUIC, Minecraft compression and login state, plugin runtime.

## Protocol support

| Protocol | Versions detected | Parsed |
| --- | --- | --- |
| TLS/SSL | SSL 3.0, TLS 1.0, 1.1, 1.2, 1.3 | record, ClientHello/ServerHello, SNI, ALPN, cipher suites |
| HTTP | 0.9, 1.0, 1.1, 2 (preface) | start line, headers, WebSocket upgrade |
| DNS | standard query | question name, qtype, qclass |
| QUIC | v1, v2, draft-29 | long header, version, DCID/SCID |
| WebSocket | RFC 6455 | upgrade handshake |
| Minecraft | handshake protocol version | VarInt framing, packet id |

HTTP/3 is detected only as QUIC. Full HTTP/3 needs a QUIC stack and the TLS 1.3
key schedule; that is a later phase.

## Build

From the repository root:

    cargo test --workspace
    cargo build --release

Qt client:

    cmake -S client/qt -B build/qt \
      -DCMAKE_PREFIX_PATH=/Users/apple/Qt/6.11.2/macos
    cmake --build build/qt -j 4

Quality gates:

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

## CLI

    ./target/release/nettoold proxy \
      --listen 127.0.0.1:25565 \
      --target 127.0.0.1:25566 \
      --protocol auto \
      --pcap capture.pcap

    ./target/release/nettoold proxy --duration 10 ...

    ./target/release/nettoold proxy \
      --rule 'drop dir=c2s proto=mc contains=hello' ...

    ./target/release/nettoold serve --state nettoold.state

    ./target/release/nettoold analyze capture.pcap --hex --dissect

| Option | Default | Description |
| --- | --- | --- |
| --listen | 127.0.0.1:25565 | Local listen address |
| --target | 127.0.0.1:25566 | Upstream target |
| --protocol | auto | auto, mc, http, tls, dns, ws, quic, raw |
| --pcap | none | Write a real pcap |
| --duration | none | Stop accepting after N seconds |
| --drop-direction | none | Drop c2s or s2c; still captured |
| --rule | none | Repeatable drop/allow predicate; see docs/RULES.md |
| --quiet | off | Suppress per-packet output |

### Packet rules

Rules are evaluated per captured chunk; the first match wins and no match
allows. Use --rule repeatedly. An earlier allow can whitelist traffic before a
later drop.

    --rule 'drop proto=http prefix=GET'
    --rule 'drop dir=s2c min_len=4096'

See docs/RULES.md for syntax, event reporting and safety.

### Control server

    ./target/release/nettoold serve --state nettoold.state

Loopback only. The daemon writes port and token to a 0600 state file and speaks
JSON lines. The MVP implements list_sessions (empty), acquire_mac dry-run,
release_mac, list_leases and shutdown. Proxy-session commands are next.

See docs/CONTROL-API.md for the message model.

Example output:

    [conn 127.0.0.1:52950] tls TLS 1.3 sni=mc.example alpn=h2,http/1.1 ciphers=1
    #1 C->S 98B tls
    #2 S->C 98B tls

## Layout

    crates/nettool-proto     VarInt, MC framing, registry, TLS/HTTP/DNS/QUIC
    crates/nettool-rules     packet predicates and ordered rule set
    crates/nettool-capture   pcap writer/reader, synthetic TCP frames
    crates/nettool-proxy     TCP relay, EventSink fan-out, direction drops, rules
    crates/nettool-control   control types, session and MAC lease state machine
    crates/nettoold          CLI and local control server
    client/qt                Qt 6 Widgets client (main window port)
    docs/PROTOCOL.md         wire formats, versions and proxy semantics
    docs/RULES.md            packet rule syntax, semantics and safety
    docs/ARCHITECTURE.md     crate graph, detection flow, extension points
    docs/PROCESS-MODEL.md    UI / daemon / privileged helper, crash isolation
    docs/CONTROL-API.md      daemon API message model
    docs/MIGRATION.md        Python -> Rust mapping and phases
    docs/PLUGIN-ABI.md       C ABI design for DLL/dylib plugins
    docs/AUTHORIZED-USE.md   allowed and prohibited use
    SECURITY.md              vulnerability reporting and safety boundary

## Design

- Rust-first. C is reserved for OS shims, not for parsing untrusted bytes.
- UI and backend are separate processes: Qt 6 client -> nettoold daemon ->
  privileged helper. A UI crash does not stop capture or MAC leases.
- MAC identity is owned by a privileged helper with explicit leases, audit and
  restore. Leases end only on explicit release or helper shutdown.
- Raw bytes are forwarded immediately; framing never delays traffic.
- Protocol detection never silently changes the target port.
- Multi-version parsing is metadata-first. No hand-written cryptography.

## Contributing

Issues and pull requests are welcome. Read CONTRIBUTING.md before opening a PR.
Contributions must be focused and tested, and must not rewrite, delete or
overwrite another contributor's work. Never force-push to main.

## Security

NetTool is for systems you own or are explicitly authorized to test. It does
not ship stealth, evasion or unauthorized remote-control features. See
SECURITY.md and docs/AUTHORIZED-USE.md.

## License

Apache-2.0. See LICENSE.

---

本项目由 SilentCodeTeams 监管与负责。
本项目由 SilentCodeTeams 对法律、版权事宜及相关咨询承担责任。
版权所有 © SilentCodeTeams。
保留所有权利。

This project is supervised and administered by SilentCodeTeams.
This project is covered by SilentCodeTeams, which assumes responsibility for legal and copyright matters and related inquiries.
Copyright © SilentCodeTeams.
All rights reserved.
