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

//! Classic pcap (libpcap) writer and reader.

use std::io::{self, Read, Write};
use std::net::Ipv4Addr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::tcp::{build_tcp_frame, TcpSegment};

const PCAP_MAGIC: u32 = 0xa1b2_c3d4;
const LINKTYPE_ETHERNET: u32 = 1;
const SNAPLEN: u32 = 65535;
const MAX_RECORD: usize = 4 * 1024 * 1024;

/// Writes a real classic pcap file. Wireshark/tcpdump can open the result.
pub struct PcapWriter<W: Write> {
    w: W,
    ident: u16,
}

impl<W: Write> PcapWriter<W> {
    pub fn new(mut w: W) -> io::Result<Self> {
        let mut header = [0u8; 24];
        header[0..4].copy_from_slice(&PCAP_MAGIC.to_le_bytes());
        header[4..6].copy_from_slice(&2u16.to_le_bytes()); // major
        header[6..8].copy_from_slice(&4u16.to_le_bytes()); // minor
        header[8..12].copy_from_slice(&0i32.to_le_bytes()); // thiszone
        header[12..16].copy_from_slice(&0u32.to_le_bytes()); // sigfigs
        header[16..20].copy_from_slice(&SNAPLEN.to_le_bytes());
        header[20..24].copy_from_slice(&LINKTYPE_ETHERNET.to_le_bytes());
        w.write_all(&header)?;
        Ok(Self { w, ident: 1 })
    }

    /// Append one TCP segment directed at the given endpoints.
    pub fn write_tcp(&mut self, ts: SystemTime, segment: &TcpSegment) -> io::Result<()> {
        let frame = build_tcp_frame(segment, self.ident);
        self.ident = self.ident.wrapping_add(1);
        self.write_record(ts, &frame)
    }

    fn write_record(&mut self, ts: SystemTime, frame: &[u8]) -> io::Result<()> {
        let dur = ts.duration_since(UNIX_EPOCH).unwrap_or_default();
        let mut header = [0u8; 16];
        header[0..4].copy_from_slice(&(dur.as_secs() as u32).to_le_bytes());
        header[4..8].copy_from_slice(&dur.subsec_micros().to_le_bytes());
        header[8..12].copy_from_slice(&(frame.len() as u32).to_le_bytes());
        header[12..16].copy_from_slice(&(frame.len() as u32).to_le_bytes());
        self.w.write_all(&header)?;
        self.w.write_all(frame)?;
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.w.flush()
    }

    pub fn into_inner(self) -> W {
        self.w
    }
}

/// A TCP segment read back from a pcap file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedPacket {
    pub ts_sec: u32,
    pub ts_usec: u32,
    pub src_ip: Ipv4Addr,
    pub dst_ip: Ipv4Addr,
    pub src_port: u16,
    pub dst_port: u16,
    pub seq: u32,
    pub ack: u32,
    pub payload: Vec<u8>,
}

/// Read a classic pcap file produced by this crate (little- or big-endian).
pub fn read_pcap<R: Read>(mut r: R) -> io::Result<Vec<CapturedPacket>> {
    let mut global = [0u8; 24];
    r.read_exact(&mut global)?;
    let magic_le = u32::from_le_bytes([global[0], global[1], global[2], global[3]]);
    let swapped = match magic_le {
        PCAP_MAGIC => false,
        m if m.swap_bytes() == PCAP_MAGIC => true,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "not a pcap file (bad magic)",
            ))
        }
    };
    let u32_at = |b: &[u8]| {
        let arr = [b[0], b[1], b[2], b[3]];
        if swapped {
            u32::from_be_bytes(arr)
        } else {
            u32::from_le_bytes(arr)
        }
    };

    let mut out = Vec::new();
    let mut header = [0u8; 16];
    loop {
        match r.read(&mut header) {
            Ok(0) => break,
            Ok(n) if n < 16 => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "truncated pcap record header",
                ))
            }
            Ok(_) => {}
            Err(e) => return Err(e),
        }
        let ts_sec = u32_at(&header[0..4]);
        let ts_usec = u32_at(&header[4..8]);
        let incl_len = u32_at(&header[8..12]) as usize;
        if incl_len > MAX_RECORD {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "pcap record too large",
            ));
        }
        let mut frame = vec![0u8; incl_len];
        r.read_exact(&mut frame)?;
        if let Some(parsed) = parse_ethernet_tcp(&frame) {
            out.push(CapturedPacket {
                ts_sec,
                ts_usec,
                src_ip: parsed.src_ip,
                dst_ip: parsed.dst_ip,
                src_port: parsed.src_port,
                dst_port: parsed.dst_port,
                seq: parsed.seq,
                ack: parsed.ack,
                payload: parsed.payload,
            });
        }
    }
    Ok(out)
}

struct ParsedTcp {
    src_ip: Ipv4Addr,
    dst_ip: Ipv4Addr,
    src_port: u16,
    dst_port: u16,
    seq: u32,
    ack: u32,
    payload: Vec<u8>,
}

fn parse_ethernet_tcp(frame: &[u8]) -> Option<ParsedTcp> {
    if frame.len() < 14 {
        return None;
    }
    if u16::from_be_bytes([frame[12], frame[13]]) != 0x0800 {
        return None;
    }
    let ip = &frame[14..];
    if ip.len() < 20 || ip[0] >> 4 != 4 {
        return None;
    }
    let ihl = ((ip[0] & 0x0f) as usize) * 4;
    if ihl < 20 || ip.len() < ihl + 20 || ip[9] != 6 {
        return None;
    }
    let src_ip = Ipv4Addr::new(ip[12], ip[13], ip[14], ip[15]);
    let dst_ip = Ipv4Addr::new(ip[16], ip[17], ip[18], ip[19]);
    let tcp = &ip[ihl..];
    let data_off = ((tcp[12] >> 4) as usize) * 4;
    if data_off < 20 || tcp.len() < data_off {
        return None;
    }
    Some(ParsedTcp {
        src_ip,
        dst_ip,
        src_port: u16::from_be_bytes([tcp[0], tcp[1]]),
        dst_port: u16::from_be_bytes([tcp[2], tcp[3]]),
        seq: u32::from_be_bytes([tcp[4], tcp[5], tcp[6], tcp[7]]),
        ack: u32::from_be_bytes([tcp[8], tcp[9], tcp[10], tcp[11]]),
        payload: tcp[data_off..].to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(
        src: Ipv4Addr,
        dst: Ipv4Addr,
        sport: u16,
        dport: u16,
        seq: u32,
        ack: u32,
        payload: &[u8],
    ) -> TcpSegment<'_> {
        TcpSegment {
            src_ip: src,
            dst_ip: dst,
            src_port: sport,
            dst_port: dport,
            seq,
            ack,
            payload,
        }
    }

    #[test]
    fn write_and_read_roundtrip() {
        let mut writer = PcapWriter::new(Vec::new()).unwrap();
        writer
            .write_tcp(
                SystemTime::now(),
                &segment(
                    Ipv4Addr::new(10, 0, 0, 1),
                    Ipv4Addr::new(10, 0, 0, 2),
                    40000,
                    25565,
                    100,
                    200,
                    b"hello",
                ),
            )
            .unwrap();
        writer
            .write_tcp(
                SystemTime::now(),
                &segment(
                    Ipv4Addr::new(10, 0, 0, 2),
                    Ipv4Addr::new(10, 0, 0, 1),
                    25565,
                    40000,
                    200,
                    105,
                    b"world",
                ),
            )
            .unwrap();
        let bytes = writer.into_inner();
        let packets = read_pcap(&bytes[..]).unwrap();
        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0].payload, b"hello");
        assert_eq!(packets[1].payload, b"world");
        assert_eq!(packets[0].src_port, 40000);
        assert_eq!(packets[0].dst_port, 25565);
        assert_eq!(packets[0].seq, 100);
        assert_eq!(packets[1].ack, 105);
    }
}
