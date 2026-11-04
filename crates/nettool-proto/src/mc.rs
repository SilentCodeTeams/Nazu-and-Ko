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

//! Minecraft packet framing.
//!
//! Wire format (pre-1.8 style / 1.8+ uncompressed):
//!   [VarInt packet_length][VarInt packet_id][payload...]
//!
//! The framer is byte-oriented and tolerates TCP giving us partial or
//! coalesced packets. Compression (1.8+ after login) is not decoded in
//! Phase 0; frames report the raw compressed payload and the caller can
//! detect it by looking at the first VarInt of the payload.

use crate::varint::{decode_varint, encode_varint, VarIntError};

/// Hard cap to avoid a hostile length prefix allocating gigabytes.
pub const MAX_PACKET_SIZE: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// Complete on-wire bytes, including the length prefix.
    pub raw: Vec<u8>,
    /// Packet body, starting at the packet id.
    pub payload: Vec<u8>,
}

/// Incremental framer. Feed it bytes from a TCP stream, get complete frames.
#[derive(Debug, Default)]
pub struct PacketFramer {
    buf: Vec<u8>,
    max_packet: usize,
}

impl PacketFramer {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            max_packet: MAX_PACKET_SIZE,
        }
    }

    pub fn with_max_packet(max_packet: usize) -> Self {
        Self {
            buf: Vec::new(),
            max_packet,
        }
    }

    /// Append bytes and return every complete frame now available.
    pub fn push(&mut self, data: &[u8]) -> Vec<Frame> {
        self.buf.extend_from_slice(data);
        let mut out = Vec::new();
        loop {
            let (len, prefix_len) = match decode_varint(&self.buf, 0) {
                Ok(v) => v,
                Err(VarIntError::Truncated) => break,
                Err(VarIntError::TooLong) => {
                    // Desync: drop buffered garbage and recover on next read.
                    self.buf.clear();
                    break;
                }
            };
            if len < 0 || len as usize > self.max_packet {
                self.buf.clear();
                break;
            }
            let total = prefix_len + len as usize;
            if self.buf.len() < total {
                break;
            }
            let raw = self.buf[..total].to_vec();
            let payload = self.buf[prefix_len..total].to_vec();
            out.push(Frame { raw, payload });
            self.buf.drain(..total);
        }
        out
    }

    /// Bytes currently waiting for the rest of a frame.
    pub fn buffered(&self) -> usize {
        self.buf.len()
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }
}

/// Wrap a packet body (packet id + fields) in its length prefix.
pub fn frame_bytes(payload: &[u8]) -> Vec<u8> {
    let mut out = encode_varint(payload.len() as i32);
    out.extend_from_slice(payload);
    out
}

/// Read the packet id from a frame payload.
pub fn packet_id(payload: &[u8]) -> Option<i32> {
    decode_varint(payload, 0).ok().map(|(v, _)| v)
}

/// Best-effort label for early-phase packets.
///
/// These ids are version-dependent. The table is scoped to the 1.8-era
/// protocol used by the Python prototype; a data-driven version table is
/// planned for a later phase. Callers should always print the raw id too.
pub fn packet_name(id: i32) -> &'static str {
    match id {
        0x00 => "handshake/status",
        0x01 => "status_request/login",
        0x02 => "login_start",
        0x03 => "chat",
        0x04 => "client_settings/disconnect",
        0x05 => "client_settings",
        0x06 => "plugin_message",
        0x1f => "plugin_message_cb",
        0x20 => "keepalive_cb",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(id: u8, body: &[u8]) -> Vec<u8> {
        let mut payload = vec![id];
        payload.extend_from_slice(body);
        frame_bytes(&payload)
    }

    #[test]
    fn single_frame() {
        let mut framer = PacketFramer::new();
        let frames = framer.push(&packet(0x00, b"abc"));
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].payload, vec![0x00, b'a', b'b', b'c']);
        assert_eq!(frames[0].raw, packet(0x00, b"abc"));
    }

    #[test]
    fn split_across_reads() {
        let wire = packet(0x03, b"hello");
        let mut framer = PacketFramer::new();
        assert!(framer.push(&wire[..2]).is_empty());
        assert!(framer.push(&wire[2..4]).is_empty());
        let frames = framer.push(&wire[4..]);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].payload, vec![0x03, b'h', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn coalesced_frames() {
        let mut wire = packet(0x00, b"a");
        wire.extend_from_slice(&packet(0x03, b"bc"));
        let mut framer = PacketFramer::new();
        let frames = framer.push(&wire);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].payload, vec![0x00, b'a']);
        assert_eq!(frames[1].payload, vec![0x03, b'b', b'c']);
        assert_eq!(framer.buffered(), 0);
    }

    #[test]
    fn bad_length_prefix_is_dropped() {
        let mut framer = PacketFramer::with_max_packet(16);
        let huge = encode_varint(1024);
        let frames = framer.push(&huge);
        assert!(frames.is_empty());
        assert_eq!(framer.buffered(), 0);
    }

    #[test]
    fn packet_id_and_name() {
        let payload = vec![0x03, b'h', b'i'];
        assert_eq!(packet_id(&payload), Some(0x03));
        assert_eq!(packet_name(0x03), "chat");
    }
}
