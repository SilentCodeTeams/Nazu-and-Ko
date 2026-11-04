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

//! Capture support for NetTool.
//!
//! Phase 0 writes a real classic pcap (Ethernet + IPv4 + TCP) instead of the
//! Python prototype's raw-bytes-with-.pcap-extension. Reads the same format
//! back for offline analysis.

pub mod pcap;
pub mod tcp;

pub use pcap::{read_pcap, CapturedPacket, PcapWriter};
pub use tcp::{build_tcp_frame, checksum, TcpSegment};
