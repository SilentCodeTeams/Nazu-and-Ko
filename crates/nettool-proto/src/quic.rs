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

//! QUIC long-header detection (Phase 1).
//!
//! This is a version detector, not a QUIC stack. Full HTTP/3 support needs a
//! QUIC implementation (quinn) and TLS 1.3 key schedule, which is planned for
//! a later phase.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuicInfo {
    pub version: u32,
    pub dcid_len: u8,
    pub scid_len: u8,
}

pub fn inspect(first: &[u8]) -> Option<QuicInfo> {
    if first.len() < 7 {
        return None;
    }
    // Long header form (0x80) and fixed bit (0x40).
    if first[0] & 0xc0 != 0xc0 {
        return None;
    }
    let version = u32::from_be_bytes([first[1], first[2], first[3], first[4]]);
    let dcid_len = first[5] as usize;
    if dcid_len > 20 || 6 + dcid_len >= first.len() {
        return None;
    }
    let scid_len = first[6 + dcid_len] as usize;
    if scid_len > 20 {
        return None;
    }
    Some(QuicInfo {
        version,
        dcid_len: dcid_len as u8,
        scid_len: scid_len as u8,
    })
}

pub fn version_label(version: u32) -> String {
    match version {
        0x0000_0001 => "QUIC v1".to_string(),
        0x6b33_43cf => "QUIC v2".to_string(),
        0xff00_001d => "QUIC draft-29".to_string(),
        other => format!("QUIC 0x{other:08x}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quic_v1_initial() {
        let mut packet = vec![0xc0];
        packet.extend_from_slice(&1u32.to_be_bytes());
        packet.push(8);
        packet.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        packet.push(0);
        let info = inspect(&packet).expect("quic");
        assert_eq!(info.version, 1);
        assert_eq!(version_label(info.version), "QUIC v1");
        assert_eq!(info.dcid_len, 8);
        assert_eq!(info.scid_len, 0);
    }

    #[test]
    fn rejects_short_header() {
        assert!(inspect(&[0x40, 0, 0, 0, 1]).is_none());
        assert!(inspect(&[0xc0, 0, 0, 0, 1]).is_none());
    }
}
