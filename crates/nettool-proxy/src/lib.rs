//! NetTool proxy: transparent TCP relay with protocol-aware framing,
//! packet events, direction drops and real pcap capture.

pub mod proxy;

pub use proxy::{DetectMode, Direction, Proxy, ProxyConfig, ProxyHandle};
