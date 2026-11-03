# NetTool ζ

English | [简体中文](language/README.zh-CN.md)

NetTool is the Rust core for network protocol analysis and authorized security
testing. It succeeds the Python NetTool prototype with a dependency-free,
cross-platform implementation and a clean process boundary between the backend
and UI clients.

## Status

- Phase 1 foundation is complete: protocol registry, transparent TCP proxy,
  real pcap capture, PacketEvent/EventSink, control types.
- Protocol dissectors: TLS/SSL, HTTP, DNS, QUIC, WebSocket, Minecraft.
- 44 tests; rustfmt and clippy clean; CI green on macOS, Windows and Linux.
- Not yet: daemon API and Qt client, privileged MAC helper, TLS MITM,
  HTTP/2 HPACK, HTTP/3 over QUIC, Minecraft compression and login state,
  plugin runtime.

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

Example output:

    [conn 127.0.0.1:52950] tls TLS 1.3 sni=mc.example alpn=h2,http/1.1 ciphers=1
    #1 C->S 98B tls
    #2 S->C 98B tls

## Layout

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
