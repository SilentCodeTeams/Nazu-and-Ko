//! NetTool protocol core.
//!
//! Phase 0 scope:
//! - Minecraft VarInt codec
//! - Minecraft length-prefixed packet framer (split + coalesced reads)
//! - Minimal protocol detection (TLS / HTTP / Minecraft)
//!
//! Deliberately dependency-free so it builds on macOS, Windows x64/ARM64
//! and Linux without network access.

pub mod detect;
pub mod mc;
pub mod varint;

pub use detect::{detect, Protocol};
pub use mc::{packet_id, packet_name, Frame, PacketFramer, MAX_PACKET_SIZE};
pub use varint::{decode_varint, encode_varint, VarIntError};
