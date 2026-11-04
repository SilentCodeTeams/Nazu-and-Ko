// 本文件由 SilentStudio 监管与负责。
// 本文件属于 SilentCodeTeams 产品 Nazu&Ko 的源码部分
// 本项目由 SilentCodeTeams 对法律、版权事宜及相关咨询承担责任。
// 版权所有 © SilentStudio。
// 以 AGPL-3.0-or-later 授权。
// This document is supervised and responsible by SilentStudio.
// This file is part of the source code of the SilentCodeTeams product Nazu&Ko
// SilentCodeTeams is responsible for legal and copyright issues and related consultation in this project.
// Copyright © SilentStudio.
// Licensed as AGPL-3.0-or-later.

//! Protocol registry and first-bytes detection.
//!
//! Each built-in protocol is a Dissector that returns a Detection (protocol,
//! version, detail, confidence). The registry picks the highest-confidence
//! result, so adding a protocol does not mean editing one giant if-chain.
//!
//! Built-ins (Phase 1):
//! - TLS / SSL records and ClientHello / ServerHello metadata
//! - HTTP/0.9, HTTP/1.0, HTTP/1.1, HTTP/2 preface, WebSocket upgrade
//! - DNS questions
//! - Minecraft handshake
//! - QUIC long headers

use std::sync::OnceLock;

use crate::dns;
use crate::http;
use crate::mc::MAX_PACKET_SIZE;
use crate::quic;
use crate::tls;
use crate::varint::decode_varint;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protocol {
    Minecraft,
    Http,
    Tls,
    Dns,
    WebSocket,
    Quic,
    Unknown,
}

impl Protocol {
    pub fn as_str(self) -> &'static str {
        match self {
            Protocol::Minecraft => "minecraft",
            Protocol::Http => "http",
            Protocol::Tls => "tls",
            Protocol::Dns => "dns",
            Protocol::WebSocket => "websocket",
            Protocol::Quic => "quic",
            Protocol::Unknown => "raw",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detection {
    pub protocol: Protocol,
    pub version: Option<String>,
    pub detail: Option<String>,
    pub confidence: u8,
}

impl Detection {
    pub fn unknown() -> Self {
        Self {
            protocol: Protocol::Unknown,
            version: None,
            detail: None,
            confidence: 0,
        }
    }
}

fn detection(
    protocol: Protocol,
    version: Option<&str>,
    detail: Option<String>,
    confidence: u8,
) -> Detection {
    Detection {
        protocol,
        version: version.map(|value| value.to_string()),
        detail,
        confidence,
    }
}

/// A protocol detector. Plugins can implement this trait and register it.
pub trait Dissector: Send + Sync {
    fn name(&self) -> &'static str;
    fn detect(&self, first: &[u8]) -> Option<Detection>;
}

struct TlsDissector;

impl Dissector for TlsDissector {
    fn name(&self) -> &'static str {
        "tls"
    }

    fn detect(&self, first: &[u8]) -> Option<Detection> {
        let info = tls::inspect(first)?;
        let mut detail = Vec::new();
        if let Some(sni) = &info.sni {
            detail.push(format!("sni={sni}"));
        }
        if !info.alpn.is_empty() {
            detail.push(format!("alpn={}", info.alpn.join(",")));
        }
        if !info.cipher_suites.is_empty() {
            detail.push(format!("ciphers={}", info.cipher_suites.len()));
        }
        if !info.is_client_hello && !info.is_server_hello {
            detail.push("record".to_string());
        }
        let confidence = if info.is_client_hello || info.is_server_hello {
            95
        } else {
            90
        };
        Some(detection(
            Protocol::Tls,
            Some(info.version_label()),
            if detail.is_empty() {
                None
            } else {
                Some(detail.join(" "))
            },
            confidence,
        ))
    }
}

struct HttpDissector;

impl Dissector for HttpDissector {
    fn name(&self) -> &'static str {
        "http"
    }

    fn detect(&self, first: &[u8]) -> Option<Detection> {
        let info = http::inspect(first)?;
        if info.version == http::HttpVersion::Http2 {
            return Some(detection(
                Protocol::Http,
                Some("HTTP/2"),
                Some("h2 preface".to_string()),
                99,
            ));
        }
        if let Some(upgrade) = &info.upgraded {
            if upgrade.eq_ignore_ascii_case("websocket") {
                return Some(detection(
                    Protocol::WebSocket,
                    Some("RFC 6455"),
                    Some("upgrade handshake".to_string()),
                    96,
                ));
            }
        }
        let detail = if info.is_request {
            Some(format!(
                "{} {}",
                info.method.as_deref().unwrap_or("-"),
                info.target.as_deref().unwrap_or("-")
            ))
        } else {
            Some(format!("status={}", info.status.unwrap_or(0)))
        };
        Some(detection(
            Protocol::Http,
            Some(info.version.label()),
            detail,
            90,
        ))
    }
}

struct DnsDissector;

impl Dissector for DnsDissector {
    fn name(&self) -> &'static str {
        "dns"
    }

    fn detect(&self, first: &[u8]) -> Option<Detection> {
        let info = dns::inspect(first)?;
        Some(detection(
            Protocol::Dns,
            Some("DNS"),
            Some(format!("{} qtype={}", info.qname, info.qtype)),
            60,
        ))
    }
}

struct MinecraftDissector;

impl Dissector for MinecraftDissector {
    fn name(&self) -> &'static str {
        "minecraft"
    }

    fn detect(&self, first: &[u8]) -> Option<Detection> {
        let (len, prefix_len) = decode_varint(first, 0).ok()?;
        if len < 0 || len as usize > MAX_PACKET_SIZE {
            return None;
        }
        if prefix_len + len as usize > first.len() {
            return None;
        }
        let (packet_id, id_len) = decode_varint(first, prefix_len).ok()?;
        if packet_id != 0 {
            return None;
        }
        let mut pos = prefix_len + id_len;
        let (protocol_version, version_len) = decode_varint(first, pos).ok()?;
        if !(0..=1000).contains(&protocol_version) {
            return None;
        }
        pos += version_len;
        let (address_len, address_len_len) = decode_varint(first, pos).ok()?;
        if address_len < 0 || address_len as usize > 255 {
            return None;
        }
        pos += address_len_len;
        if pos + address_len as usize + 2 > first.len() {
            return None;
        }
        pos += address_len as usize + 2;
        let (next_state, _) = decode_varint(first, pos).ok()?;
        if next_state != 1 && next_state != 2 {
            return None;
        }
        let version = format!("MC {protocol_version}");
        Some(detection(
            Protocol::Minecraft,
            Some(&version),
            Some(format!("handshake len={len} id=0x{packet_id:02x}")),
            80,
        ))
    }
}

struct QuicDissector;

impl Dissector for QuicDissector {
    fn name(&self) -> &'static str {
        "quic"
    }

    fn detect(&self, first: &[u8]) -> Option<Detection> {
        let info = quic::inspect(first)?;
        let version = quic::version_label(info.version);
        Some(detection(
            Protocol::Quic,
            Some(&version),
            Some(format!("dcid={} scid={}", info.dcid_len, info.scid_len)),
            50,
        ))
    }
}

/// Registry of protocol dissectors.
pub struct Registry {
    dissectors: Vec<Box<dyn Dissector>>,
}

impl Registry {
    pub fn builtin() -> Self {
        Self {
            dissectors: vec![
                Box::new(TlsDissector),
                Box::new(HttpDissector),
                Box::new(DnsDissector),
                Box::new(MinecraftDissector),
                Box::new(QuicDissector),
            ],
        }
    }

    pub fn register(&mut self, dissector: Box<dyn Dissector>) {
        self.dissectors.push(dissector);
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.dissectors.iter().map(|item| item.name()).collect()
    }

    pub fn detect(&self, first: &[u8]) -> Detection {
        let mut best: Option<Detection> = None;
        for dissector in &self.dissectors {
            if let Some(candidate) = dissector.detect(first) {
                let better = match &best {
                    Some(current) => candidate.confidence > current.confidence,
                    None => true,
                };
                if better {
                    best = Some(candidate);
                }
            }
        }
        best.unwrap_or_else(Detection::unknown)
    }
}

pub fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(Registry::builtin)
}

/// Inspect the first bytes of a stream and return the best detection.
pub fn inspect(first: &[u8]) -> Detection {
    registry().detect(first)
}

/// Convenience wrapper for callers that only need the protocol label.
pub fn detect(first: &[u8]) -> Protocol {
    inspect(first).protocol
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mc::frame_bytes;
    use crate::varint::encode_varint;

    fn dns_query(name: &str, qtype: u16) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&0x1234u16.to_be_bytes());
        out.extend_from_slice(&0x0100u16.to_be_bytes());
        out.extend_from_slice(&1u16.to_be_bytes());
        out.extend_from_slice(&0u16.to_be_bytes());
        out.extend_from_slice(&0u16.to_be_bytes());
        out.extend_from_slice(&0u16.to_be_bytes());
        for label in name.split('.') {
            out.push(label.len() as u8);
            out.extend_from_slice(label.as_bytes());
        }
        out.push(0);
        out.extend_from_slice(&qtype.to_be_bytes());
        out.extend_from_slice(&1u16.to_be_bytes());
        out
    }

    fn quic_initial() -> Vec<u8> {
        let mut packet = vec![0xc0];
        packet.extend_from_slice(&1u32.to_be_bytes());
        packet.push(8);
        packet.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        packet.push(0);
        packet
    }

    #[test]
    fn detects_tls_record_version() {
        let found = inspect(&[0x16, 0x03, 0x03, 0x00, 0x00]);
        assert_eq!(found.protocol, Protocol::Tls);
        assert_eq!(found.version.as_deref(), Some("TLS 1.2"));
    }

    #[test]
    fn detects_http_versions() {
        assert_eq!(detect(b"GET / HTTP/1.1\r\n"), Protocol::Http);
        let found = inspect(b"GET / HTTP/1.0\r\n\r\n");
        assert_eq!(found.version.as_deref(), Some("HTTP/1.0"));
        let h2 = inspect(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n");
        assert_eq!(h2.version.as_deref(), Some("HTTP/2"));
    }

    #[test]
    fn detects_websocket_upgrade() {
        let found = inspect(b"GET /chat HTTP/1.1\r\nUpgrade: websocket\r\n\r\n");
        assert_eq!(found.protocol, Protocol::WebSocket);
        assert_eq!(found.version.as_deref(), Some("RFC 6455"));
    }

    #[test]
    fn detects_minecraft_handshake() {
        let mut payload = vec![0x00];
        payload.extend_from_slice(&encode_varint(767));
        payload.push(1);
        payload.push(b'a');
        payload.extend_from_slice(&25565u16.to_be_bytes());
        payload.extend_from_slice(&encode_varint(2));
        let wire = frame_bytes(&payload);
        let found = inspect(&wire);
        assert_eq!(found.protocol, Protocol::Minecraft);
        assert_eq!(found.version.as_deref(), Some("MC 767"));
    }

    #[test]
    fn detects_dns() {
        let found = inspect(&dns_query("example.com", 1));
        assert_eq!(found.protocol, Protocol::Dns);
        assert_eq!(found.detail.as_deref(), Some("example.com qtype=1"));
    }

    #[test]
    fn detects_quic() {
        let found = inspect(&quic_initial());
        assert_eq!(found.protocol, Protocol::Quic);
        assert_eq!(found.version.as_deref(), Some("QUIC v1"));
    }

    #[test]
    fn unknown_for_noise() {
        assert_eq!(detect(&[0x01, 0x02, 0x03, 0x04]), Protocol::Unknown);
        assert_eq!(detect(&[]), Protocol::Unknown);
    }

    #[test]
    fn registry_names() {
        let names = registry().names();
        assert!(names.contains(&"tls"));
        assert!(names.contains(&"http"));
        assert!(names.contains(&"dns"));
        assert!(names.contains(&"minecraft"));
        assert!(names.contains(&"quic"));
    }

    #[test]
    fn fuzz_like_input_does_not_panic() {
        let mut state: u32 = 0x1234_5678;
        for len in 0..512 {
            let mut data = vec![0u8; len];
            for byte in &mut data {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                *byte = (state >> 24) as u8;
            }
            let _ = inspect(&data);
        }
    }
}
