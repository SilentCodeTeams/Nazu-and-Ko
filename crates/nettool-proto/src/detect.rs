//! Minimal first-bytes protocol detection.
//!
//! This is a heuristic for labeling traffic, not a parser. It never changes
//! the configured target port by itself.

use crate::mc::MAX_PACKET_SIZE;
use crate::varint::decode_varint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Minecraft,
    Http,
    Tls,
    Unknown,
}

impl Protocol {
    pub fn as_str(self) -> &'static str {
        match self {
            Protocol::Minecraft => "minecraft",
            Protocol::Http => "http",
            Protocol::Tls => "tls",
            Protocol::Unknown => "raw",
        }
    }
}

/// Detect from the first client bytes.
pub fn detect(first: &[u8]) -> Protocol {
    if is_tls(first) {
        return Protocol::Tls;
    }
    if is_http(first) {
        return Protocol::Http;
    }
    if is_minecraft_handshake(first) {
        return Protocol::Minecraft;
    }
    Protocol::Unknown
}

fn is_tls(d: &[u8]) -> bool {
    d.len() >= 3 && d[0] == 0x16 && d[1] == 0x03
}

fn is_http(d: &[u8]) -> bool {
    const METHODS: &[&[u8; 4]] = &[
        b"GET ", b"POST", b"PUT ", b"HEAD", b"DELE", b"OPTI", b"CONN", b"TRAC", b"PATC",
    ];
    d.len() >= 4 && METHODS.iter().any(|m| &d[..4] == *m)
}

fn is_minecraft_handshake(d: &[u8]) -> bool {
    let (len, prefix_len) = match decode_varint(d, 0) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if len < 0 || len as usize > MAX_PACKET_SIZE {
        return false;
    }
    let (id, id_len) = match decode_varint(d, prefix_len) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if id != 0 {
        return false;
    }
    let (protocol_version, _) = match decode_varint(d, prefix_len + id_len) {
        Ok(v) => v,
        Err(_) => return false,
    };
    (0..=1000).contains(&protocol_version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mc::frame_bytes;

    #[test]
    fn detects_tls() {
        assert_eq!(detect(&[0x16, 0x03, 0x01, 0x00]), Protocol::Tls);
    }

    #[test]
    fn detects_http() {
        assert_eq!(detect(b"GET / HTTP/1.1\r\n"), Protocol::Http);
        assert_eq!(detect(b"POST /a HTTP/1.1\r\n"), Protocol::Http);
    }

    #[test]
    fn detects_minecraft_handshake() {
        let mut payload = vec![0x00];
        payload.extend_from_slice(&crate::varint::encode_varint(767));
        payload.push(1);
        payload.push(b'a');
        payload.extend_from_slice(&25565u16.to_be_bytes());
        payload.extend_from_slice(&crate::varint::encode_varint(2));
        let wire = frame_bytes(&payload);
        assert_eq!(detect(&wire), Protocol::Minecraft);
    }

    #[test]
    fn unknown_for_noise() {
        assert_eq!(detect(&[0x01, 0x02, 0x03, 0x04]), Protocol::Unknown);
        assert_eq!(detect(&[]), Protocol::Unknown);
    }
}
