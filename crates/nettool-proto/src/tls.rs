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

//! TLS / SSL record and handshake parsing.
//!
//! Phase 1 scope: detect every SSL/TLS record version the peers use and parse
//! ClientHello / ServerHello metadata:
//!
//! - SSL 3.0 (0x0300), TLS 1.0 (0x0301), TLS 1.1 (0x0302), TLS 1.2 (0x0303)
//! - TLS 1.3 negotiated through the supported_versions extension (0x0304)
//! - SNI, ALPN, cipher suites
//!
//! No cryptography and no decryption. MITM will use a vetted TLS library
//! (rustls / OpenSSL) in a later phase, not a hand-written record layer.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsInfo {
    pub record_version: u16,
    pub legacy_version: u16,
    pub negotiated_version: u16,
    pub handshake_type: u8,
    pub sni: Option<String>,
    pub alpn: Vec<String>,
    pub cipher_suites: Vec<u16>,
    pub is_client_hello: bool,
    pub is_server_hello: bool,
}

impl TlsInfo {
    pub fn version_label(&self) -> &'static str {
        version_label(self.negotiated_version)
    }
}

pub fn version_label(version: u16) -> &'static str {
    match version {
        0x0300 => "SSL 3.0",
        0x0301 => "TLS 1.0",
        0x0302 => "TLS 1.1",
        0x0303 => "TLS 1.2",
        0x0304 => "TLS 1.3",
        _ => "TLS (unknown)",
    }
}

fn record_only(version: u16) -> TlsInfo {
    TlsInfo {
        record_version: version,
        legacy_version: version,
        negotiated_version: version,
        handshake_type: 0,
        sni: None,
        alpn: Vec::new(),
        cipher_suites: Vec::new(),
        is_client_hello: false,
        is_server_hello: false,
    }
}

/// Inspect the first bytes of a TLS stream. Returns None when this is not a
/// TLS record prefix.
pub fn inspect(first: &[u8]) -> Option<TlsInfo> {
    if first.len() < 3 {
        return None;
    }
    let content_type = first[0];
    if content_type != 0x16 && content_type != 0x17 {
        return None;
    }
    let version = u16::from_be_bytes([first[1], first[2]]);
    if version >> 8 != 0x03 {
        return None;
    }
    if first.len() < 5 {
        return Some(record_only(version));
    }
    if content_type == 0x17 {
        return Some(record_only(version));
    }

    let mut record_version = version;
    let mut handshake = Vec::new();
    let mut pos = 0;
    while pos + 5 <= first.len() {
        let content = first[pos];
        let record_ver = u16::from_be_bytes([first[pos + 1], first[pos + 2]]);
        let len = u16::from_be_bytes([first[pos + 3], first[pos + 4]]) as usize;
        if content != 0x16 || pos + 5 + len > first.len() {
            break;
        }
        record_version = record_ver;
        handshake.extend_from_slice(&first[pos + 5..pos + 5 + len]);
        pos += 5 + len;
        if handshake.len() >= 4 {
            let need = 4 + handshake_len(&handshake);
            if handshake.len() >= need {
                break;
            }
        }
    }

    if handshake.len() < 4 {
        return Some(record_only(record_version));
    }
    let handshake_type = handshake[0];
    let body_len = handshake_len(&handshake);
    if handshake.len() < 4 + body_len {
        return Some(record_only(record_version));
    }
    let body = &handshake[4..4 + body_len];
    match handshake_type {
        1 => parse_client_hello(record_version, body),
        2 => parse_server_hello(record_version, body),
        _ => Some(TlsInfo {
            record_version,
            legacy_version: record_version,
            negotiated_version: record_version,
            handshake_type,
            sni: None,
            alpn: Vec::new(),
            cipher_suites: Vec::new(),
            is_client_hello: false,
            is_server_hello: false,
        }),
    }
}

fn handshake_len(handshake: &[u8]) -> usize {
    ((handshake[1] as usize) << 16) | ((handshake[2] as usize) << 8) | handshake[3] as usize
}

fn parse_client_hello(record_version: u16, body: &[u8]) -> Option<TlsInfo> {
    let mut reader = Reader::new(body);
    let legacy_version = reader.u16()?;
    reader.skip(32)?;
    let session_len = reader.u8()? as usize;
    reader.skip(session_len)?;
    let suites_len = reader.u16()? as usize;
    let suites = reader.take(suites_len)?;
    let (suite_chunks, _) = suites.as_chunks::<2>();
    let cipher_suites = suite_chunks
        .iter()
        .map(|chunk| u16::from_be_bytes(*chunk))
        .collect::<Vec<_>>();
    let compression_len = reader.u8()? as usize;
    reader.skip(compression_len)?;

    let mut sni = None;
    let mut alpn = Vec::new();
    let mut supported = Vec::new();
    if reader.remaining() >= 2 {
        let extensions_len = reader.u16()? as usize;
        if let Some(extensions) = reader.take(extensions_len) {
            let parsed = parse_extensions(extensions, false);
            sni = parsed.sni;
            alpn = parsed.alpn;
            supported = parsed.supported_versions;
        }
    }
    let negotiated = supported.iter().copied().max().unwrap_or(legacy_version);
    Some(TlsInfo {
        record_version,
        legacy_version,
        negotiated_version: negotiated,
        handshake_type: 1,
        sni,
        alpn,
        cipher_suites,
        is_client_hello: true,
        is_server_hello: false,
    })
}

fn parse_server_hello(record_version: u16, body: &[u8]) -> Option<TlsInfo> {
    let mut reader = Reader::new(body);
    let legacy_version = reader.u16()?;
    reader.skip(32)?;
    let session_len = reader.u8()? as usize;
    reader.skip(session_len)?;
    let cipher_suite = reader.u16()?;
    reader.u8()?; // compression method

    let mut sni = None;
    let mut alpn = Vec::new();
    let mut selected = None;
    if reader.remaining() >= 2 {
        let extensions_len = reader.u16()? as usize;
        if let Some(extensions) = reader.take(extensions_len) {
            let parsed = parse_extensions(extensions, true);
            sni = parsed.sni;
            alpn = parsed.alpn;
            selected = parsed.server_selected_version;
        }
    }
    let negotiated = selected.unwrap_or(legacy_version);
    Some(TlsInfo {
        record_version,
        legacy_version,
        negotiated_version: negotiated,
        handshake_type: 2,
        sni,
        alpn,
        cipher_suites: vec![cipher_suite],
        is_client_hello: false,
        is_server_hello: true,
    })
}

#[derive(Default)]
struct ParsedExtensions {
    sni: Option<String>,
    alpn: Vec<String>,
    supported_versions: Vec<u16>,
    server_selected_version: Option<u16>,
}

fn parse_extensions(data: &[u8], server: bool) -> ParsedExtensions {
    let mut out = ParsedExtensions::default();
    let mut reader = Reader::new(data);
    while reader.remaining() >= 4 {
        let ext_type = match reader.u16() {
            Some(v) => v,
            None => break,
        };
        let ext_len = match reader.u16() {
            Some(v) => v as usize,
            None => break,
        };
        let ext = match reader.take(ext_len) {
            Some(v) => v,
            None => break,
        };
        match ext_type {
            0x0000 => out.sni = parse_sni(ext),
            0x0010 => out.alpn = parse_alpn(ext),
            0x002b => {
                if server {
                    out.server_selected_version = parse_server_version(ext);
                } else {
                    out.supported_versions = parse_supported_versions(ext);
                }
            }
            _ => {}
        }
    }
    out
}

fn parse_sni(data: &[u8]) -> Option<String> {
    let mut reader = Reader::new(data);
    let _list_len = reader.u16()?;
    let name_type = reader.u8()?;
    if name_type != 0 {
        return None;
    }
    let name_len = reader.u16()? as usize;
    let name = reader.take(name_len)?;
    std::str::from_utf8(name).ok().map(|s| s.to_string())
}

fn parse_alpn(data: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    let mut reader = Reader::new(data);
    let list_len = match reader.u16() {
        Some(v) => v as usize,
        None => return out,
    };
    let list = match reader.take(list_len) {
        Some(v) => v,
        None => return out,
    };
    let mut list_reader = Reader::new(list);
    while list_reader.remaining() >= 1 {
        let len = match list_reader.u8() {
            Some(v) => v as usize,
            None => break,
        };
        let proto = match list_reader.take(len) {
            Some(v) => v,
            None => break,
        };
        if let Ok(text) = std::str::from_utf8(proto) {
            out.push(text.to_string());
        }
    }
    out
}

fn parse_supported_versions(data: &[u8]) -> Vec<u16> {
    let mut reader = Reader::new(data);
    let len = match reader.u8() {
        Some(v) => v as usize,
        None => return Vec::new(),
    };
    let list = match reader.take(len) {
        Some(v) => v,
        None => return Vec::new(),
    };
    let (version_chunks, _) = list.as_chunks::<2>();
    version_chunks
        .iter()
        .map(|chunk| u16::from_be_bytes(*chunk))
        .collect()
}

fn parse_server_version(data: &[u8]) -> Option<u16> {
    if data.len() < 2 {
        return None;
    }
    Some(u16::from_be_bytes([data[0], data[1]]))
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn u8(&mut self) -> Option<u8> {
        let value = *self.data.get(self.pos)?;
        self.pos += 1;
        Some(value)
    }

    fn u16(&mut self) -> Option<u16> {
        if self.remaining() < 2 {
            return None;
        }
        let value = u16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Some(value)
    }

    fn take(&mut self, len: usize) -> Option<&'a [u8]> {
        if self.remaining() < len {
            return None;
        }
        let out = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Some(out)
    }

    fn skip(&mut self, len: usize) -> Option<()> {
        self.take(len).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extension(ext_type: u16, data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&ext_type.to_be_bytes());
        out.extend_from_slice(&(data.len() as u16).to_be_bytes());
        out.extend_from_slice(data);
        out
    }

    fn client_hello(legacy: u16, versions: &[u16], sni: &str, alpn: &[&str]) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(&legacy.to_be_bytes());
        body.extend_from_slice(&[0u8; 32]);
        body.push(0); // session id length
        body.extend_from_slice(&2u16.to_be_bytes());
        body.extend_from_slice(&0x1301u16.to_be_bytes());
        body.push(1);
        body.push(0);

        let mut extensions = Vec::new();
        let name = sni.as_bytes();
        let mut sni_data = Vec::new();
        sni_data.extend_from_slice(&((name.len() + 3) as u16).to_be_bytes());
        sni_data.push(0);
        sni_data.extend_from_slice(&(name.len() as u16).to_be_bytes());
        sni_data.extend_from_slice(name);
        extensions.extend(extension(0x0000, &sni_data));

        let mut alpn_list = Vec::new();
        for protocol in alpn {
            alpn_list.push(protocol.len() as u8);
            alpn_list.extend_from_slice(protocol.as_bytes());
        }
        let mut alpn_data = Vec::new();
        alpn_data.extend_from_slice(&(alpn_list.len() as u16).to_be_bytes());
        alpn_data.extend_from_slice(&alpn_list);
        extensions.extend(extension(0x0010, &alpn_data));

        if !versions.is_empty() {
            let mut version_data = vec![(versions.len() * 2) as u8];
            for version in versions {
                version_data.extend_from_slice(&version.to_be_bytes());
            }
            extensions.extend(extension(0x002b, &version_data));
        }

        body.extend_from_slice(&(extensions.len() as u16).to_be_bytes());
        body.extend_from_slice(&extensions);

        let mut handshake = vec![1u8];
        handshake.extend_from_slice(&(body.len() as u32).to_be_bytes()[1..]);
        handshake.extend_from_slice(&body);

        let mut record = vec![0x16, 0x03, 0x03];
        record.extend_from_slice(&(handshake.len() as u16).to_be_bytes());
        record.extend_from_slice(&handshake);
        record
    }

    #[test]
    fn tls10_client_hello() {
        let wire = client_hello(0x0301, &[], "example.com", &["http/1.1"]);
        let info = inspect(&wire).expect("tls");
        assert!(info.is_client_hello);
        assert_eq!(info.legacy_version, 0x0301);
        assert_eq!(info.negotiated_version, 0x0301);
        assert_eq!(info.version_label(), "TLS 1.0");
        assert_eq!(info.sni.as_deref(), Some("example.com"));
        assert_eq!(info.alpn, vec!["http/1.1"]);
    }

    #[test]
    fn tls12_client_hello() {
        let wire = client_hello(0x0303, &[], "mc.example", &["h2"]);
        let info = inspect(&wire).expect("tls");
        assert_eq!(info.version_label(), "TLS 1.2");
        assert_eq!(info.sni.as_deref(), Some("mc.example"));
    }

    #[test]
    fn tls13_client_hello() {
        let wire = client_hello(
            0x0303,
            &[0x0304, 0x0303],
            "api.example",
            &["h2", "http/1.1"],
        );
        let info = inspect(&wire).expect("tls");
        assert_eq!(info.legacy_version, 0x0303);
        assert_eq!(info.negotiated_version, 0x0304);
        assert_eq!(info.version_label(), "TLS 1.3");
        assert_eq!(info.sni.as_deref(), Some("api.example"));
        assert_eq!(info.alpn, vec!["h2", "http/1.1"]);
    }

    #[test]
    fn ssl3_short_prefix() {
        let info = inspect(&[0x16, 0x03, 0x00, 0x00]).expect("ssl3");
        assert_eq!(info.version_label(), "SSL 3.0");
    }

    #[test]
    fn application_data_is_not_handshake() {
        let info = inspect(&[0x17, 0x03, 0x03, 0x00, 0x05, 1, 2, 3, 4, 5]).expect("record");
        assert!(!info.is_client_hello);
        assert_eq!(info.version_label(), "TLS 1.2");
    }

    #[test]
    fn junk_is_none() {
        assert!(inspect(b"GET / HTTP/1.1\r\n").is_none());
        assert!(inspect(&[0x16, 0x02, 0x00]).is_none());
    }
}
