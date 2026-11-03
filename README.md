# NetTool 2 (net2)

Rust core for the next generation of NetTool. Phase 1 is a working TCP proxy
with a version-aware protocol registry, real pcap capture and an offline
analyzer.

The Python prototype in the parent directory (NetTool delta) stays as a
behavioral reference. This workspace is dependency-free so it builds offline
on macOS, Windows x64, Windows ARM64 and Linux.

## Status

Phase 1 foundation complete:

- VarInt codec and Minecraft length-prefixed framer (split and coalesced reads)
- Protocol registry with confidence-ranked dissectors
- TLS / SSL: SSL 3.0, TLS 1.0, 1.1, 1.2, 1.3; ClientHello / ServerHello;
  SNI, ALPN, cipher suites
- HTTP: HTTP/0.9, 1.0, 1.1, HTTP/2 preface; request/response start lines and
  headers; WebSocket upgrade detection
- DNS question parsing
- QUIC long-header / version detection
- Transparent TCP relay with per-direction capture
- Real classic pcap writer (Ethernet + IPv4 + TCP) and reader
- CLI: proxy, analyze (with --dissect), version
- 38 tests, zero external crates, clippy clean with -D warnings

Not yet: TLS MITM/decryption, HTTP/2 HPACK, HTTP/3 over QUIC, Minecraft
compression and login state, packet-id drop rules, master/agent control plane,
plugin runtime, GUI.

## Protocol matrix

    Protocol   Versions detected            Parsed in Phase 1
    --------   --------------------------   -------------------------------
    TLS/SSL    SSL3, TLS 1.0, 1.1, 1.2, 1.3  record, ClientHello, ServerHello,
                                             SNI, ALPN, cipher suites
    HTTP       0.9, 1.0, 1.1, 2 (preface)    start line, headers, WebSocket upgrade
    DNS        standard query               question name, qtype, qclass
    QUIC       v1, v2, draft-29             long header, version, DCID/SCID
    WebSocket  RFC 6455                     upgrade handshake
    Minecraft  handshake protocol version   VarInt framing, packet id

HTTP/3 is detected only as QUIC in Phase 1. Full HTTP/3 needs a QUIC stack and
TLS 1.3 key schedule; that is a later phase.

## Build

    cd net2
    cargo test --workspace
    cargo build --release

Quality gates used in CI:

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

Options:

    --listen <addr>          default 127.0.0.1:25565
    --target <host:port>     default 127.0.0.1:25566
    --protocol <mode>        auto | mc | http | tls | dns | ws | quic | raw
    --pcap <file>            write a real pcap (Wireshark/tcpdump compatible)
    --duration <seconds>     stop accepting after N seconds
    --drop-direction <dir>   drop c2s or s2c (still captured and logged)
    --quiet                  suppress per-packet output

Example detection output:

    [conn 127.0.0.1:52950] tls TLS 1.3 sni=mc.example alpn=h2,http/1.1 ciphers=1
    #1 C->S 98B tls
    #2 S->C 98B tls

## Layout

    crates/nettool-proto     VarInt, MC framing, registry, TLS/HTTP/DNS/QUIC
    crates/nettool-capture   pcap writer/reader, synthetic TCP frames
    crates/nettool-proxy     TCP relay, direction drops, capture sink
    crates/nettoold          CLI entry point
    docs/PROTOCOL.md         wire formats, versions and proxy semantics
    docs/ARCHITECTURE.md     crate graph, detection flow, extension points
    docs/MIGRATION.md        Python -> Rust mapping and phases
    docs/PLUGIN-ABI.md       C ABI design for DLL/dylib plugins
    docs/AUTHORIZED-USE.md   allowed and prohibited use
    SECURITY.md              vulnerability reporting and safety boundary

## Design decisions

- Rust-first. C is reserved for OS shims (BPF/pfctl, tc, Npcap/WinDivert/WFP),
  not for parsers of untrusted bytes.
- Stable C ABI for plugins. Rust cdylib builds produce .dylib/.dll/.so; no Rust
  types cross the boundary.
- Apache-2.0.
- GUI will be a separate client (Tauri preferred) over a daemon API. The
  Python web dashboard is not ported into the core.
- The proxy forwards raw bytes immediately. Framing is for display and capture
  metadata; it never delays traffic.
- Protocol detection never silently switches the target port. That behavior
  caused the Python prototype's post-detection disconnect bug.
- Multi-version parsing is metadata-first. We do not hand-roll cryptography;
  TLS MITM will use a vetted library in a later phase.

## License

Apache-2.0. See LICENSE.
