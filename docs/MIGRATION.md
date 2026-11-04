# Python -> Rust migration

## Decisions

- Rust-first; C only for OS shims.
- Apache-2.0.
- Qt 6 Widgets client, a separate process, over a daemon API.
- UI and backend are separate processes. The Qt client never links the core;
  a UI crash or restart does not stop capture, proxy sessions or MAC leases.
- MAC identity is owned by a privileged helper service, not by the UI and not
  by the daemon. Leases are released only by an explicit user command or an
  explicit helper shutdown, and every change is audited.
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
    pcap + console side effects         PacketEvent + EventSink
    SmartExportDialog (GUI)             nettoold analyze (offline CLI)
    Intruder / Scanner dialogs          future: Rust engine + UI client
    Web dashboard                       future: daemon API + client
    GUI state tied to the process       Qt client over daemon API
    MAC change in the UI process        nettool-macd helper service + leases
    Utils/project, nbt_editor           future: data model / editor plugin

## Phase plan

Phase 0

- protocol core, proxy relay, real pcap, CLI, tests
- verified on macOS and Windows x64/ARM64 build hosts

Phase 1 (done)

- protocol registry: TLS/SSL, HTTP, DNS, QUIC, WebSocket, Minecraft
- PacketEvent + EventSink boundary
- CI, security and authorized-use docs

Phase 2 (in progress)

- rule engine with packet predicates (done)
- daemon API over a local authenticated transport (control types first)
- framed forwarding
- MC compression and login state
- HTTP/2 frames

Phase 3

- capture backends: BPF (macOS), AF_PACKET (Linux), Npcap/WinDivert (Windows)
- kernel QoS backends: pfctl/dnctl, tc, WFP
- nettool-macd privileged helper with leases, audit and restore
- plugin host with C ABI and signed manifests

Phase 4

- Qt 6 Widgets client over the daemon API
- master/agent control plane with Ed25519 pairing and audit chain

## Intentional behavior changes

- No automatic target-port switch. Detection is a label, not a rewrite.
  This removes the Python race where server-to-client forwarding died after
  auto-detection.
- Real pcap, not raw bytes with a .pcap name.
- Packets are framed before being reported, not equated with TCP reads.
- No GUI in the core. The daemon, helper and CLI are the product surface; UI
  clients are separate processes.
- No implicit release of privileged state on UI disconnect. A lease ends only
  by an explicit release or an explicit helper shutdown.
