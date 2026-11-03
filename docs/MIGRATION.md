# Python -> Rust migration

## Decisions

- Rust-first; C only for OS shims.
- Apache-2.0.
- GUI later as a separate client (Tauri preferred) over a daemon API.
- Python web dashboard is not ported into the core.
- Python code is frozen as a reference implementation.

## Mapping

    Python (NetTool delta)              Rust (net2)
    ----------------------------------  --------------------------------------
    VarIntCodec                         nettool-proto::varint
    MinecraftPacketParser + chunk table nettool-proto::mc (framing first)
    TCP chunk = packet assumption       PacketFramer (real length framing)
    ProxyServer / ConnectionHandler     nettool-proxy::Proxy
    detect-and-reconnect race           removed: target port is fixed
    fake .pcap export                   nettool-capture::PcapWriter (real pcap)
    SmartExportDialog (GUI)             nettoold analyze (offline CLI)
    Intruder / Scanner dialogs          future: Rust engine + UI client
    Web dashboard                       future: daemon API + separate client
    Utils/project, nbt_editor           future: data model / editor plugin

## Phase plan

Phase 0 (this workspace)

- protocol core, proxy relay, real pcap, CLI, tests
- verified on macOS and Windows x64/ARM64 build hosts

Phase 1

- compression + login state
- HTTP/1.1 parser and TLS ClientHello/SNI
- rule engine with packet-id and field predicates
- daemon API skeleton (JSON over local socket)

Phase 2

- capture backends: BPF (macOS), AF_PACKET (Linux), Npcap/WinDivert (Windows)
- kernel QoS backends: pfctl/dnctl, tc, WFP
- plugin host with C ABI and signed manifests

Phase 3

- GUI client (Tauri) reusing the Python web templates as a design reference
- master/agent control plane with Ed25519 pairing and audit chain

## Intentional behavior changes

- No automatic target-port switch. Detection is a label, not a rewrite.
  This removes the Python race where server-to-client forwarding died after
  auto-detection.
- Real pcap, not raw bytes with a .pcap name.
- Packets are framed before being reported, not equated with TCP reads.
- No GUI in the core. The daemon and CLI are the product surface; UI clients
  are separate.
