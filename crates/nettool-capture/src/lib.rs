//! Capture support for NetTool.
//!
//! Phase 0 writes a real classic pcap (Ethernet + IPv4 + TCP) instead of the
//! Python prototype's raw-bytes-with-.pcap-extension. Reads the same format
//! back for offline analysis.

pub mod pcap;
pub mod tcp;

pub use pcap::{read_pcap, CapturedPacket, PcapWriter};
pub use tcp::{build_tcp_frame, checksum, TcpSegment};
