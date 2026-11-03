# Protocol notes

## Built-in dissectors

The registry is confidence-ranked. Each dissector receives the first client
bytes and returns protocol, version, detail and confidence.

    Protocol   Versions detected             Confidence   Parsed
    --------   ---------------------------   ----------   ---------------------
    TLS/SSL    SSL3, TLS 1.0/1.1/1.2/1.3     95 (90 rec)  record, hello, SNI,
                                                            ALPN, cipher suites
    HTTP       0.9, 1.0, 1.1, 2 (preface)    90 (99 h2)   start line, headers
    WebSocket  RFC 6455                       96          HTTP upgrade handshake
    DNS        standard query                 60          question
    Minecraft  handshake protocol version     80          VarInt framing
    QUIC       v1, v2, draft-29               50          long header, version

The highest confidence wins. Unknown input returns Protocol::Unknown.

## TLS / SSL

Parser scope:

- record layer: content type 0x16 (handshake) / 0x17 (application data)
- record version: 0x0300 SSL 3.0, 0x0301 TLS 1.0, 0x0302 TLS 1.1,
  0x0303 TLS 1.2
- ClientHello extensions: SNI (0x0000), ALPN (0x0010), supported_versions
  (0x002b)
- ServerHello: selected cipher suite, selected supported_version, ALPN
- negotiated version: TLS 1.3 when supported_versions contains 0x0304,
  otherwise the legacy version

No cryptography is implemented. The parser only reads public handshake
metadata. TLS MITM and decryption are a later phase and will use rustls or
OpenSSL, never a hand-written record layer.

## HTTP

- HTTP/0.9: method and target only
- HTTP/1.0 and HTTP/1.1: request line, response line, headers
- HTTP/2: connection preface detection (PRI * HTTP/2.0 ...); frame and HPACK
  decoding are later phases
- WebSocket: detected through the HTTP Upgrade handshake
- HTTP/3: only QUIC long-header detection in Phase 1

## DNS

Standard query header and question section: qname, qtype, qclass. Compression
pointers and answers are not decoded yet.

## QUIC

Long-header detection, version and connection ID lengths. Version labels:
v1 (0x00000001), v2 (0x6b3343cf), draft-29 (0xff00001d).

## Minecraft VarInt

Minecraft VarInts are little-endian base-128 with a continuation bit. Maximum
five bytes for a 32-bit value. Negative i32 values use the five-byte two's
complement form.

Implementation: crates/nettool-proto/src/varint.rs

## Minecraft framing

Phase 1 handles the uncompressed wire format:

    [VarInt packet_length][VarInt packet_id][fields...]

packet_length counts the bytes after the length prefix (packet id + fields).

The framer is incremental:

- partial packets stay buffered
- multiple packets in one TCP read are all emitted
- a length above the 2 MiB cap drops the buffer (desync recovery)

Compression (1.8+ after login) uses:

    [VarInt packet_length][VarInt uncompressed_length][zlib data]

It is not decoded yet. Frames expose the raw payload so a later phase can add
a zlib backend and a login-state machine.

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
- no HTTP/2 HPACK or HTTP/3 QUIC stack
- packet-id drop rules need a framed forwarding mode; not in Phase 1

## pcap output

The writer emits classic libpcap:

- magic 0xa1b2c3d4, version 2.4, snaplen 65535, link type 1 (Ethernet)
- synthetic MACs, IPv4, TCP with PSH+ACK
- correct IP and TCP checksums
- per-direction sequence and acknowledgement numbers

tcpdump and Wireshark open the files directly. The Python prototype wrote raw
payload concatenation with a .pcap extension; that is fixed here.

Implementation: crates/nettool-capture/src/pcap.rs and tcp.rs
