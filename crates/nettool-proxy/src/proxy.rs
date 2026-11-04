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

//! Proxy: transparent TCP relay with protocol detection and an event bus.
//!
//! Design notes:
//! - The client speaks first (Minecraft/HTTP/TLS). We read the first chunk,
//!   detect the protocol label, then connect to the configured target. We do
//!   NOT silently switch the target port; that was the source of the Python
//!   prototype's protocol-detection race.
//! - Raw bytes are forwarded immediately. Framing is used for display/pcap
//!   metadata and never delays forwarding.
//! - Every captured chunk becomes a PacketEvent. Pcap, console, the future
//!   daemon and the GUI all consume the same EventSink interface.
//! - Each connection has its own stop flag; both relay directions exit when
//!   either side closes.

use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{IpAddr, Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use nettool_capture::{PcapWriter, TcpSegment};
use nettool_proto::{inspect, packet_id, Detection, Frame, PacketFramer, Protocol};
use nettool_rules::{Action, Direction as RuleDirection, RuleContext, RuleSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    ClientToServer,
    ServerToClient,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::ClientToServer => "C->S",
            Direction::ServerToClient => "S->C",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "c2s" | "c->s" | "client" | "serverbound" => Some(Direction::ClientToServer),
            "s2c" | "s->c" | "server" | "clientbound" => Some(Direction::ServerToClient),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectMode {
    Auto,
    Minecraft,
    Http,
    Tls,
    Dns,
    WebSocket,
    Quic,
    Raw,
}

impl DetectMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(DetectMode::Auto),
            "mc" | "minecraft" => Some(DetectMode::Minecraft),
            "http" => Some(DetectMode::Http),
            "tls" | "https" | "ssl" => Some(DetectMode::Tls),
            "dns" => Some(DetectMode::Dns),
            "ws" | "websocket" => Some(DetectMode::WebSocket),
            "quic" | "http3" | "h3" => Some(DetectMode::Quic),
            "raw" | "tcp" => Some(DetectMode::Raw),
            _ => None,
        }
    }

    fn resolve(self, first: &[u8]) -> Detection {
        match self {
            DetectMode::Auto => inspect(first),
            DetectMode::Minecraft => forced(Protocol::Minecraft, "minecraft"),
            DetectMode::Http => forced(Protocol::Http, "HTTP"),
            DetectMode::Tls => forced(Protocol::Tls, "TLS"),
            DetectMode::Dns => forced(Protocol::Dns, "DNS"),
            DetectMode::WebSocket => forced(Protocol::WebSocket, "RFC 6455"),
            DetectMode::Quic => forced(Protocol::Quic, "QUIC"),
            DetectMode::Raw => Detection::unknown(),
        }
    }
}

fn forced(protocol: Protocol, label: &str) -> Detection {
    Detection {
        protocol,
        version: Some(label.to_string()),
        detail: Some("forced".to_string()),
        confidence: 100,
    }
}

/// One captured chunk, as seen by every event consumer.
#[derive(Debug, Clone)]
pub struct PacketEvent {
    pub id: u64,
    pub timestamp: SystemTime,
    pub direction: Direction,
    pub protocol: Protocol,
    pub src: SocketAddr,
    pub dst: SocketAddr,
    pub seq: u32,
    pub ack: u32,
    pub data: Vec<u8>,
    pub frames: Vec<Frame>,
    pub dropped: bool,
    pub matched_rule: Option<String>,
}

/// A consumer of PacketEvents. Implementations must be Send because they are
/// shared by the two relay threads.
pub trait EventSink: Send {
    fn on_event(&mut self, event: &PacketEvent) -> io::Result<()>;
}

/// Shared handle to an event consumer. The proxy fans every event out to all
/// registered sinks; pcap, console, daemon and tests use the same type.
pub type SharedSink = Arc<Mutex<dyn EventSink>>;

/// Writes every event to a classic pcap file.
pub struct PcapSink {
    writer: PcapWriter<File>,
}

impl PcapSink {
    pub fn create(path: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self {
            writer: PcapWriter::new(File::create(path)?)?,
        })
    }
}

impl EventSink for PcapSink {
    fn on_event(&mut self, event: &PacketEvent) -> io::Result<()> {
        if let (IpAddr::V4(src_ip), IpAddr::V4(dst_ip)) = (event.src.ip(), event.dst.ip()) {
            let segment = TcpSegment {
                src_ip,
                dst_ip,
                src_port: event.src.port(),
                dst_port: event.dst.port(),
                seq: event.seq,
                ack: event.ack,
                payload: &event.data,
            };
            self.writer.write_tcp(event.timestamp, &segment)?;
        }
        Ok(())
    }
}

/// Prints one line per captured packet or Minecraft frame.
#[derive(Debug, Default)]
pub struct ConsoleSink;

impl ConsoleSink {
    pub fn new() -> Self {
        Self
    }
}

impl EventSink for ConsoleSink {
    fn on_event(&mut self, event: &PacketEvent) -> io::Result<()> {
        let mut suffix = String::new();
        if event.dropped {
            suffix.push_str(" dropped");
        }
        if let Some(rule) = &event.matched_rule {
            suffix.push_str(" rule=");
            suffix.push_str(rule);
        }
        if event.protocol == Protocol::Minecraft && !event.frames.is_empty() {
            for frame in &event.frames {
                let id = packet_id(&frame.payload)
                    .map(|value| format!("0x{value:02x}"))
                    .unwrap_or_else(|| "unknown".to_string());
                println!(
                    "#{} {} {}B mc id={}{}",
                    event.id,
                    event.direction.as_str(),
                    frame.payload.len(),
                    id,
                    suffix
                );
            }
        } else {
            println!(
                "#{} {} {}B {}{}",
                event.id,
                event.direction.as_str(),
                event.data.len(),
                event.protocol.as_str(),
                suffix
            );
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct ProxyConfig {
    pub listen: SocketAddr,
    pub target_host: String,
    pub target_port: u16,
    pub protocol: DetectMode,
    pub sinks: Vec<SharedSink>,
    pub verbose: bool,
    pub drop_direction: Option<Direction>,
    pub rules: RuleSet,
    pub duration: Option<Duration>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen: "127.0.0.1:25565".parse().expect("valid default listen"),
            target_host: "127.0.0.1".to_string(),
            target_port: 25566,
            protocol: DetectMode::Auto,
            sinks: Vec::new(),
            verbose: true,
            drop_direction: None,
            rules: RuleSet::default(),
            duration: None,
        }
    }
}

/// A bound proxy. Bind first (so tests can learn the port), then run.
pub struct Proxy {
    listener: TcpListener,
    config: ProxyConfig,
    running: Arc<AtomicBool>,
}

/// A handle that can stop a running proxy from another thread.
#[derive(Clone)]
pub struct ProxyHandle {
    running: Arc<AtomicBool>,
}

impl ProxyHandle {
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Proxy {
    pub fn bind(config: ProxyConfig) -> io::Result<Self> {
        let listener = TcpListener::bind(config.listen)?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener,
            config,
            running: Arc::new(AtomicBool::new(true)),
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn handle(&self) -> ProxyHandle {
        ProxyHandle {
            running: self.running.clone(),
        }
    }

    pub fn run(&self) -> io::Result<()> {
        let start = Instant::now();
        while self.running.load(Ordering::SeqCst) {
            if let Some(limit) = self.config.duration {
                if start.elapsed() >= limit {
                    break;
                }
            }
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    let config = self.config.clone();
                    let conn_running = Arc::new(AtomicBool::new(true));
                    thread::spawn(move || {
                        if let Err(e) = handle_connection(stream, addr, config, conn_running) {
                            eprintln!("[conn {}] {}", addr, e);
                        }
                    });
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(25));
                }
                Err(e) => {
                    eprintln!("[proxy] accept error: {}", e);
                    break;
                }
            }
        }
        Ok(())
    }
}

fn read_with_deadline(
    stream: &mut TcpStream,
    buf: &mut [u8],
    deadline: Duration,
) -> io::Result<usize> {
    let start = Instant::now();
    loop {
        match stream.read(buf) {
            Ok(n) => return Ok(n),
            Err(ref e)
                if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut =>
            {
                if start.elapsed() >= deadline {
                    return Ok(0);
                }
            }
            Err(e) => return Err(e),
        }
    }
}

fn handle_connection(
    mut client: TcpStream,
    client_addr: SocketAddr,
    config: ProxyConfig,
    conn_running: Arc<AtomicBool>,
) -> io::Result<()> {
    client.set_read_timeout(Some(Duration::from_secs(1)))?;
    let mut first = vec![0u8; 8192];
    let n = read_with_deadline(&mut client, &mut first, Duration::from_secs(5))?;
    if n == 0 {
        return Ok(());
    }
    first.truncate(n);
    let detection = config.protocol.resolve(&first);
    let protocol = detection.protocol;
    if config.verbose && protocol != Protocol::Unknown {
        eprintln!(
            "[conn {}] {} {} {}",
            client_addr,
            protocol.as_str(),
            detection.version.as_deref().unwrap_or("-"),
            detection.detail.as_deref().unwrap_or("")
        );
    }

    let target_addr = (config.target_host.as_str(), config.target_port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "target did not resolve"))?;
    let server = TcpStream::connect_timeout(&target_addr, Duration::from_secs(10))?;
    server.set_read_timeout(Some(Duration::from_secs(1)))?;
    client.set_read_timeout(Some(Duration::from_secs(1)))?;

    let dispatcher = Arc::new(Mutex::new(Dispatcher::new(
        &config,
        client_addr,
        target_addr,
    )));

    let client_read = client.try_clone()?;
    let server_write = server.try_clone()?;
    let server_read = server.try_clone()?;
    let client_write = client.try_clone()?;

    let s2c_running = conn_running.clone();
    let s2c_sink = dispatcher.clone();
    let s2c = thread::spawn(move || {
        relay(
            server_read,
            client_write,
            Direction::ServerToClient,
            s2c_sink,
            s2c_running,
            protocol,
            None,
            config.drop_direction,
        );
    });

    let c2s_running = conn_running.clone();
    relay(
        client_read,
        server_write,
        Direction::ClientToServer,
        dispatcher,
        c2s_running,
        protocol,
        Some(first),
        config.drop_direction,
    );

    let _ = s2c.join();
    let _ = client.shutdown(Shutdown::Both);
    let _ = server.shutdown(Shutdown::Both);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn relay(
    mut from: TcpStream,
    mut to: TcpStream,
    direction: Direction,
    dispatcher: Arc<Mutex<Dispatcher>>,
    running: Arc<AtomicBool>,
    protocol: Protocol,
    initial: Option<Vec<u8>>,
    drop_direction: Option<Direction>,
) {
    let mut framer = if protocol == Protocol::Minecraft {
        Some(PacketFramer::new())
    } else {
        None
    };
    let mut initial = initial;
    let drop_this_direction = drop_direction == Some(direction);
    let mut buf = vec![0u8; 8192];

    while running.load(Ordering::SeqCst) {
        let data = if let Some(d) = initial.take() {
            d
        } else {
            match from.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => buf[..n].to_vec(),
                Err(ref e)
                    if e.kind() == io::ErrorKind::WouldBlock
                        || e.kind() == io::ErrorKind::TimedOut =>
                {
                    continue;
                }
                Err(_) => break,
            }
        };

        let frames = match framer.as_mut() {
            Some(f) => f.push(&data),
            None => Vec::new(),
        };

        let dropped = {
            let mut dispatcher = dispatcher
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            dispatcher.record(direction, &data, &frames, protocol, drop_this_direction)
        };

        if !dropped && to.write_all(&data).is_err() {
            break;
        }
    }

    running.store(false, Ordering::SeqCst);
    let _ = to.shutdown(Shutdown::Both);
    let _ = from.shutdown(Shutdown::Both);
}

struct Dispatcher {
    sinks: Vec<SharedSink>,
    rules: RuleSet,
    client: SocketAddr,
    target: SocketAddr,
    c2s_seq: u32,
    s2c_seq: u32,
    next_id: u64,
}

impl Dispatcher {
    fn new(config: &ProxyConfig, client: SocketAddr, target: SocketAddr) -> Self {
        Self {
            sinks: config.sinks.clone(),
            rules: config.rules.clone(),
            client,
            target,
            c2s_seq: 0,
            s2c_seq: 0,
            next_id: 0,
        }
    }

    fn record(
        &mut self,
        direction: Direction,
        data: &[u8],
        frames: &[Frame],
        protocol: Protocol,
        drop_this_direction: bool,
    ) -> bool {
        let (src, dst, seq, ack) = match direction {
            Direction::ClientToServer => (self.client, self.target, self.c2s_seq, self.s2c_seq),
            Direction::ServerToClient => (self.target, self.client, self.s2c_seq, self.c2s_seq),
        };

        let rule_direction = match direction {
            Direction::ClientToServer => RuleDirection::ClientToServer,
            Direction::ServerToClient => RuleDirection::ServerToClient,
        };
        let matched = self.rules.evaluate(&RuleContext {
            direction: rule_direction,
            protocol,
            src_port: src.port(),
            dst_port: dst.port(),
            data,
        });
        let rule_drop = matched.is_some_and(|rule| rule.action() == Action::Drop);
        let matched_rule = matched.map(|rule| rule.name().to_string());
        let dropped = drop_this_direction || rule_drop;

        self.next_id += 1;
        let event = PacketEvent {
            id: self.next_id,
            timestamp: SystemTime::now(),
            direction,
            protocol,
            src,
            dst,
            seq,
            ack,
            data: data.to_vec(),
            frames: frames.to_vec(),
            dropped,
            matched_rule,
        };

        for sink in &self.sinks {
            let mut sink = sink.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Err(e) = sink.on_event(&event) {
                eprintln!("[sink] {}", e);
            }
        }

        match direction {
            Direction::ClientToServer => {
                self.c2s_seq = self.c2s_seq.wrapping_add(data.len() as u32)
            }
            Direction::ServerToClient => {
                self.s2c_seq = self.s2c_seq.wrapping_add(data.len() as u32)
            }
        }

        dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    struct RecordingSink {
        events: Arc<Mutex<Vec<PacketEvent>>>,
    }

    impl EventSink for RecordingSink {
        fn on_event(&mut self, event: &PacketEvent) -> io::Result<()> {
            self.events
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(event.clone());
            Ok(())
        }
    }

    #[test]
    fn relay_echo_and_events() {
        let echo = TcpListener::bind("127.0.0.1:0").unwrap();
        let echo_port = echo.local_addr().unwrap().port();
        thread::spawn(move || {
            if let Ok((mut stream, _)) = echo.accept() {
                let mut buf = [0u8; 1024];
                if let Ok(n) = stream.read(&mut buf) {
                    let _ = stream.write_all(&buf[..n]);
                }
            }
        });

        let events = Arc::new(Mutex::new(Vec::new()));
        let recording = Arc::new(Mutex::new(RecordingSink {
            events: events.clone(),
        }));
        let config = ProxyConfig {
            listen: "127.0.0.1:0".parse().unwrap(),
            target_host: "127.0.0.1".to_string(),
            target_port: echo_port,
            protocol: DetectMode::Raw,
            sinks: vec![recording],
            verbose: false,
            drop_direction: None,
            rules: RuleSet::default(),
            duration: Some(Duration::from_millis(500)),
        };
        let proxy = Arc::new(Proxy::bind(config).unwrap());
        let addr = proxy.local_addr().unwrap();
        let running_proxy = proxy.clone();
        thread::spawn(move || {
            let _ = running_proxy.run();
        });

        let mut client = TcpStream::connect(addr).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        client.write_all(b"hello").unwrap();
        let mut buf = [0u8; 16];
        let n = client.read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"hello");

        let captured = events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(captured.len() >= 2, "expected at least 2 events");
        assert_eq!(captured[0].direction, Direction::ClientToServer);
        assert_eq!(captured[0].data, b"hello");
        assert_eq!(captured[0].protocol, Protocol::Unknown);
        assert_eq!(captured[1].direction, Direction::ServerToClient);
        assert_eq!(captured[1].data, b"hello");
        assert_eq!(captured[1].seq, 0);
        assert_eq!(captured[1].ack, 5);
    }

    #[test]
    fn rule_drop_is_reported_and_blocks_forwarding() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let recording = Arc::new(Mutex::new(RecordingSink {
            events: events.clone(),
        }));
        let rules =
            RuleSet::parse(&["drop dir=c2s contains=secret".to_string()]).expect("valid rule");
        let config = ProxyConfig {
            listen: "127.0.0.1:0".parse().unwrap(),
            target_host: "127.0.0.1".to_string(),
            target_port: 1,
            protocol: DetectMode::Raw,
            sinks: vec![recording],
            verbose: false,
            drop_direction: None,
            rules,
            duration: None,
        };
        let mut dispatcher = Dispatcher::new(
            &config,
            "127.0.0.1:5000".parse().unwrap(),
            "127.0.0.1:5001".parse().unwrap(),
        );

        let dropped = dispatcher.record(
            Direction::ClientToServer,
            b"secret",
            &[],
            Protocol::Unknown,
            false,
        );
        assert!(dropped);

        let captured = events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert_eq!(captured.len(), 1);
        assert!(captured[0].dropped);
        assert_eq!(captured[0].matched_rule.as_deref(), Some("rule-1"));
    }
}
