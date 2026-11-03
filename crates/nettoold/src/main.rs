//! nettoold: Phase 0 CLI.
//!
//!   nettoold proxy --listen 127.0.0.1:25565 --target 127.0.0.1:25566
//!                  [--protocol auto|mc|http|tls|raw]
//!                  [--pcap out.pcap] [--duration 10] [--quiet]
//!                  [--drop-direction c2s|s2c]
//!
//!   nettoold analyze out.pcap [--hex]
//!
//!   nettoold version

use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use nettool_capture::read_pcap;
use nettool_proxy::{DetectMode, Direction, Proxy, ProxyConfig};

type CliResult = Result<(), Box<dyn Error>>;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("proxy") => cmd_proxy(&args[1..]),
        Some("analyze") => cmd_analyze(&args[1..]),
        Some("version") | Some("--version") | Some("-V") => {
            println!("nettoold {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("help") | Some("--help") | Some("-h") | None => {
            usage();
            Ok(())
        }
        Some(other) => {
            usage();
            Err(invalid(format!("unknown command: {other}")))
        }
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn invalid(message: String) -> Box<dyn Error> {
    Box::new(io::Error::new(io::ErrorKind::InvalidInput, message))
}

fn take_flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|index| args.get(index + 1))
        .cloned()
}

fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn split_host_port(value: &str) -> Result<(String, u16), Box<dyn Error>> {
    let (host, port) = value
        .rsplit_once(':')
        .ok_or_else(|| invalid(format!("expected host:port, got {value}")))?;
    if host.is_empty() {
        return Err(invalid(format!("empty host in {value}")));
    }
    let port = port
        .parse::<u16>()
        .map_err(|_| invalid(format!("bad port in {value}")))?;
    Ok((host.to_string(), port))
}

fn cmd_proxy(args: &[String]) -> CliResult {
    let listen: SocketAddr = take_flag(args, "--listen")
        .unwrap_or_else(|| "127.0.0.1:25565".to_string())
        .parse()
        .map_err(|_| invalid("bad --listen address".to_string()))?;
    let target = take_flag(args, "--target").unwrap_or_else(|| "127.0.0.1:25566".to_string());
    let (target_host, target_port) = split_host_port(&target)?;
    let protocol_name = take_flag(args, "--protocol").unwrap_or_else(|| "auto".to_string());
    let protocol = DetectMode::parse(&protocol_name)
        .ok_or_else(|| invalid(format!("bad --protocol: {protocol_name}")))?;
    let pcap = take_flag(args, "--pcap").map(PathBuf::from);
    let quiet = has_flag(args, "--quiet");
    let drop_direction = match take_flag(args, "--drop-direction") {
        Some(value) => Some(
            Direction::parse(&value)
                .ok_or_else(|| invalid(format!("bad --drop-direction: {value}")))?,
        ),
        None => None,
    };
    let duration = match take_flag(args, "--duration") {
        Some(value) => Some(Duration::from_secs(
            value
                .parse::<u64>()
                .map_err(|_| invalid(format!("bad --duration: {value}")))?,
        )),
        None => None,
    };

    let config = ProxyConfig {
        listen,
        target_host,
        target_port,
        protocol,
        pcap,
        quiet,
        drop_direction,
        duration,
    };
    println!("nettoold proxy");
    println!("  listen: {}", config.listen);
    println!("  target: {}:{}", config.target_host, config.target_port);
    let proxy = Proxy::bind(config)?;
    println!("  bound: {}", proxy.local_addr()?);
    proxy.run()?;
    Ok(())
}

fn cmd_analyze(args: &[String]) -> CliResult {
    let path = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .ok_or_else(|| invalid("usage: nettoold analyze <file.pcap> [--hex]".to_string()))?;
    let hex = has_flag(args, "--hex");
    let dissect = has_flag(args, "--dissect");
    let file = File::open(path)?;
    let packets = read_pcap(file)?;
    println!("{} packets from {}", packets.len(), path);
    for (index, packet) in packets.iter().enumerate() {
        println!(
            "#{} ts={}.{:06} {}:{} -> {}:{} seq={} ack={} len={}",
            index + 1,
            packet.ts_sec,
            packet.ts_usec,
            packet.src_ip,
            packet.src_port,
            packet.dst_ip,
            packet.dst_port,
            packet.seq,
            packet.ack,
            packet.payload.len()
        );
        if dissect && !packet.payload.is_empty() {
            let found = nettool_proto::inspect(&packet.payload);
            println!(
                "  detect: {} {} {}",
                found.protocol.as_str(),
                found.version.as_deref().unwrap_or("-"),
                found.detail.as_deref().unwrap_or("")
            );
        }
        if packet.payload.is_empty() {
            continue;
        }
        if hex {
            println!("  {}", hex_string(&packet.payload));
        } else {
            println!("  {}", preview(&packet.payload));
        }
    }
    Ok(())
}

fn hex_string(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 2);
    for byte in data {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

fn preview(data: &[u8]) -> String {
    data.iter()
        .map(|byte| {
            if byte.is_ascii_graphic() || *byte == b' ' {
                *byte as char
            } else {
                '.'
            }
        })
        .take(160)
        .collect()
}

fn usage() {
    println!(
        "nettoold {}\n\
         \n\
         USAGE:\n\
         \x20 nettoold proxy [options]\n\
         \x20 nettoold analyze <file.pcap> [--hex] [--dissect]\n\
         \x20 nettoold version\n\
         \n\
         PROXY OPTIONS:\n\
         \x20 --listen <addr>          default 127.0.0.1:25565\n\
         \x20 --target <host:port>     default 127.0.0.1:25566\n\
         \x20 --protocol <mode>        auto | mc | http | tls | dns | ws | quic | raw\n\
         \x20 --pcap <file>            write a real pcap\n\
         \x20 --duration <seconds>     stop accepting after N seconds\n\
         \x20 --drop-direction <dir>   drop c2s or s2c (still captured)\n\
         \x20 --quiet                  suppress per-packet output",
        env!("CARGO_PKG_VERSION")
    );
}
