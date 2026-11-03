# Protocol notes

## Minecraft VarInt

Minecraft VarInts are little-endian base-128 with a continuation bit. Maximum
five bytes for a 32-bit value. Negative i32 values use the five-byte two's
complement form.

Implementation: crates/nettool-proto/src/varint.rs

## Minecraft framing

Phase 0 handles the uncompressed wire format:

    [VarInt packet_length][VarInt packet_id][fields...]

packet_length counts the bytes after the length prefix (packet id + fields).

The framer is incremental:

- partial packets stay buffered
- multiple packets in one TCP read are all emitted
- a length above the 2 MiB cap drops the buffer (desync recovery)

Compression (1.8+ after login) uses:

    [VarInt packet_length][VarInt uncompressed_length][zlib data]

It is not decoded in Phase 0. Frames expose the raw payload so a later phase
can add a zlib backend and a login-state machine.

Implementation: crates/nettool-proto/src/mc.rs

## Protocol detection

Detection uses the first client bytes only, in this order:

1. TLS: first byte 0x16, second byte 0x03
2. HTTP: first four bytes are a known method (GET, POST, PUT, HEAD, DELETE,
   OPTIONS, CONNECT, TRACE, PATCH)
3. Minecraft: VarInt length, packet id 0x00, protocol version in 0..=1000

Detection labels traffic. It does not change the configured target port.

Implementation: crates/nettool-proto/src/detect.rs

## Proxy semantics

- Client speaks first. The proxy waits up to five seconds for the first chunk,
  detects the protocol label, then connects to the configured target.
- Raw bytes are forwarded immediately in both directions.
- Each direction has its own sequence counter for the pcap.
- A drop-direction rule still captures and logs the bytes; it just does not
  forward them.
- Either side closing stops both relay loops.

Known limits:

- server-first protocols are not supported yet (SMTP/FTP/SSH banners)
- TLS is passed through, not intercepted
- packet-id drop rules need a framed forwarding mode; not in Phase 0

## pcap output

The writer emits classic libpcap:

- magic 0xa1b2c3d4, version 2.4, snaplen 65535, link type 1 (Ethernet)
- synthetic MACs, IPv4, TCP with PSH+ACK
- correct IP and TCP checksums
- per-direction sequence and acknowledgement numbers

tcpdump and Wireshark open the files directly. The Python prototype wrote raw
payload concatenation with a .pcap extension; that is fixed here.

Implementation: crates/nettool-capture/src/pcap.rs and tcp.rs
