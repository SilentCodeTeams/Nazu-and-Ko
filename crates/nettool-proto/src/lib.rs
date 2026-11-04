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

//! NetTool protocol core.
//!
//! Phase 1 scope:
//! - Minecraft VarInt codec and length-prefixed framer
//! - Protocol registry with version-aware dissectors
//! - TLS / SSL record, ClientHello and ServerHello metadata
//! - HTTP/0.9 through HTTP/2 preface, WebSocket upgrade
//! - DNS questions and QUIC long-header detection
//!
//! Deliberately dependency-free so it builds offline on macOS, Windows
//! x64/ARM64 and Linux.

pub mod detect;
pub mod dns;
pub mod http;
pub mod mc;
pub mod quic;
pub mod tls;
pub mod varint;

pub use detect::{detect, inspect, Detection, Dissector, Protocol, Registry};
pub use dns::DnsInfo;
pub use http::{HttpInfo, HttpVersion};
pub use mc::{packet_id, packet_name, Frame, PacketFramer, MAX_PACKET_SIZE};
pub use quic::QuicInfo;
pub use tls::{version_label, TlsInfo};
pub use varint::{decode_varint, encode_varint, VarIntError};
