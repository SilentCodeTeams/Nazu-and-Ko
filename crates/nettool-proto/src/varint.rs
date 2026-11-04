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

//! Minecraft VarInt codec (max 5 bytes for i32).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarIntError {
    /// Not enough bytes to finish the VarInt.
    Truncated,
    /// More than 5 continuation bytes.
    TooLong,
}

/// Encode an i32 as a Minecraft VarInt. Negative values use the 5-byte form.
pub fn encode_varint(value: i32) -> Vec<u8> {
    let mut v = value as u32;
    let mut out = Vec::with_capacity(5);
    loop {
        let mut b = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            b |= 0x80;
        }
        out.push(b);
        if v == 0 {
            break;
        }
    }
    out
}

/// Decode a Minecraft VarInt at offset. Returns (value, bytes_consumed).
pub fn decode_varint(data: &[u8], offset: usize) -> Result<(i32, usize), VarIntError> {
    let mut result: u32 = 0;
    for i in 0..5 {
        let b = *data.get(offset + i).ok_or(VarIntError::Truncated)?;
        result |= ((b & 0x7f) as u32) << (7 * i);
        if b & 0x80 == 0 {
            return Ok((result as i32, i + 1));
        }
    }
    Err(VarIntError::TooLong)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_values() {
        for value in [
            0,
            1,
            2,
            127,
            128,
            255,
            300,
            25565,
            2_097_151,
            i32::MAX,
            -1,
            -2_147_483_648,
        ] {
            let encoded = encode_varint(value);
            assert!(
                encoded.len() <= 5,
                "value {value} encoded to {} bytes",
                encoded.len()
            );
            let (decoded, consumed) = decode_varint(&encoded, 0).unwrap();
            assert_eq!(decoded, value, "roundtrip failed for {value}");
            assert_eq!(consumed, encoded.len());
        }
    }

    #[test]
    fn truncated() {
        assert_eq!(decode_varint(&[], 0), Err(VarIntError::Truncated));
        assert_eq!(decode_varint(&[0x80], 0), Err(VarIntError::Truncated));
        assert_eq!(decode_varint(&[0x80, 0x80], 0), Err(VarIntError::Truncated));
    }

    #[test]
    fn too_long() {
        assert_eq!(
            decode_varint(&[0x80, 0x80, 0x80, 0x80, 0x80], 0),
            Err(VarIntError::TooLong)
        );
    }

    #[test]
    fn known_encoding() {
        assert_eq!(encode_varint(300), vec![0xac, 0x02]);
        assert_eq!(encode_varint(25565), vec![0xdd, 0xc7, 0x01]);
    }
}
