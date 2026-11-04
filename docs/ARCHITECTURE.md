# Architecture

## Crate graph

    nettool-proto
      varint / mc        Minecraft framing
      tls / http / dns    version-aware dissectors
      quic                QUIC detection
      detect              Detection + Dissector trait + Registry
        ^
        |
    nettool-capture       pcap writer/reader, synthetic TCP frames
        ^
        |
    nettool-proxy         TCP relay, EventSink fan-out, direction drops
        ^
        |
    nettool-control       control types, no I/O
    nettoold              CLI now, daemon API next
    clients               Qt 6 / CLI / tests, separate processes

Dependencies point one way. The protocol crate has no I/O and no external
crates; capture and proxy depend on it. This keeps parsers testable and
fuzzable in isolation.

## Detection flow

    first client bytes
        -> Registry::detect
            -> each Dissector::detect
                -> (protocol, version, detail, confidence)
        -> highest confidence wins
        -> Detection { protocol, version, detail, confidence }

Built-in dissectors:

- TlsDissector
- HttpDissector (also reports WebSocket upgrades)
- DnsDissector
- MinecraftDissector
- QuicDissector

Adding a protocol means implementing Dissector and registering it. Plugin
dissectors will use the same trait once the plugin host lands.

## Connection flow

    1. accept client
    2. read first chunk (5 s deadline)
    3. detect protocol label (does not change the target port)
    4. connect to the configured target
    5. spawn S->C relay, run C->S relay
    6. each chunk: frame (MC), record to pcap/console, forward raw bytes
    7. either side closes -> stop both relays

Framing is display metadata; it never delays forwarding. The exception will be
an explicit framed-forwarding mode for packet-id drop rules in a later phase.

## Process boundary

    Qt client  --control API-->  nettoold  --helper API-->  nettool-macd
                                 (capture,     (MAC leases,
                                  proxy)        audit, restore)

- Qt client: separate process, no link to the Rust core.
- nettoold: daemon. A crash stops capture/proxy sessions but does not release
  existing MAC leases.
- nettool-macd: privileged helper. It survives UI and daemon restarts; a lease
  ends only by explicit release or explicit helper shutdown.
- The Qt client never talks to the helper directly.

See docs/PROCESS-MODEL.md for the crash matrix and lease lifecycle.

## Event path today and next

The relay owns a Dispatcher that fans each PacketEvent out to registered
EventSink implementations:

    PacketEvent {
        id, timestamp, direction, protocol,
        src, dst, seq, ack,
        data, frames, dropped, matched_rule
    }

    SharedSink = Arc<Mutex<dyn EventSink>>

Built-in sinks: PcapSink and ConsoleSink. The daemon, Qt client and tests will
register their own sinks; the CLI is no longer a special case.

## Error strategy

- I/O boundaries return std::io::Result.
- Parsers return Option and never panic on short or hostile input.
- The registry returns Detection::unknown, not an error.
- CLI maps user errors to a message and exit code 1.

## Multi-version strategy

Each parser extracts a version and returns it in Detection.version. Version
specific details (for example TLS 1.3 supported_versions) are parsed inside the
dissector, not in callers. Protocol tables are data-driven where practical.

No cryptography is hand-written. TLS MITM, HTTP/2 HPACK and HTTP/3 QUIC will
use vetted libraries behind the same dissector boundary.

## Phases

Phase 0: MC framing, relay, real pcap, CLI.

Phase 1 (done): protocol registry, TLS/HTTP/DNS/QUIC detection, EventSink,
control types, docs, CI.

Phase 2 (in progress): rule engine with packet predicates done; daemon API and
control server, MC compression and login state, HTTP/2 frames, framed forwarding
next.

Phase 3: capture backends (BPF, AF_PACKET, Npcap/WinDivert), kernel QoS,
nettool-macd privileged helper, plugin host with C ABI and signed manifests.

Phase 4: Qt 6 Widgets client, master/agent control plane with Ed25519 pairing
and audit chain.
