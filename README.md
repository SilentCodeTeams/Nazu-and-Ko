# NetTool 2 (net2)

Rust core for the next generation of NetTool. Phase 0 is a working TCP proxy
with Minecraft framing, real pcap capture and an offline pcap analyzer.

The Python prototype in the parent directory (NetTool delta) stays as a
behavioral reference. This workspace is dependency-free so it builds offline
on macOS, Windows x64, Windows ARM64 and Linux.

## Status

Phase 0 complete:

- VarInt codec and Minecraft length-prefixed framer (split and coalesced reads)
- First-bytes protocol detection: TLS, HTTP, Minecraft handshake
- Transparent TCP relay with per-direction capture
- Real classic pcap writer (Ethernet + IPv4 + TCP) and reader
- CLI: proxy and analyze subcommands
- Unit and integration tests, zero external crates

Not in Phase 0: GUI, TLS MITM, Minecraft compression, packet-id drop rules,
master/agent control plane, plugin runtime.

## Build

    cd net2
    cargo test --workspace
    cargo build --release

## CLI

    # proxy: listen locally, forward to a Minecraft server, dump a real pcap
    ./target/release/nettoold proxy \
      --listen 127.0.0.1:25565 \
      --target 127.0.0.1:25566 \
      --protocol auto \
      --pcap capture.pcap

    # stop accepting after 10 seconds (useful in scripts)
    ./target/release/nettoold proxy --duration 10 ...

    # analyze the pcap offline
    ./target/release/nettoold analyze capture.pcap --hex

Options:

    --listen <addr>          default 127.0.0.1:25565
    --target <host:port>     default 127.0.0.1:25566
    --protocol <mode>        auto | mc | http | tls | raw
    --pcap <file>            write a real pcap (Wireshark/tcpdump compatible)
    --duration <seconds>     stop accepting after N seconds
    --drop-direction <dir>   drop c2s or s2c (still captured and logged)
    --quiet                  suppress per-packet output

## Layout

    crates/nettool-proto     VarInt, MCQ framing, protocol detection
    crates/nettool-capture   pcap writer/reader, synthetic TCP frames
    crates/nettool-proxy     TCP relay, direction drops, capture sink
    crates/nettoold          CLI entry point
    docs/PROTOCOL.md         wire formats and proxy semantics
    docs/MIGRATION.md        Python -> Rust mapping and phases
    docs/PLUGIN-ABI.md       C ABI design for DLL/dylib plugins

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

## License

Apache-2.0. See LICENSE.
