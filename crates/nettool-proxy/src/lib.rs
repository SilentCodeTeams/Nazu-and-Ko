//! NetTool proxy: transparent TCP relay with protocol-aware framing,
//! packet events, direction drops and real pcap capture.

pub mod proxy;

pub use nettool_rules::{Action, Rule, RuleContext, RuleError, RuleSet};
pub use proxy::{
    ConsoleSink, DetectMode, Direction, EventSink, PacketEvent, PcapSink, Proxy, ProxyConfig,
    ProxyHandle, SharedSink,
};
