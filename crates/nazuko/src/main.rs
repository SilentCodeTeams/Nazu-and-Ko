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

//! nazuko: Phase 0 CLI.
//!
//!   nazuko proxy --listen 127.0.0.1:25565 --target 127.0.0.1:25566
//!                  [--protocol auto|mc|http|tls|dns|ws|quic|raw]
//!                  [--pcap out.pcap] [--duration 10] [--quiet] [--json]
//!                  [--drop-direction c2s|s2c]
//!
//!   nazuko serve [--bind 127.0.0.1:0] [--state <file>]
//!
//!   nazuko analyze out.pcap [--hex]
//!
//!   nazuko version

mod daemon;
mod json_sink;
mod mcp;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use json_sink::JsonSink;
use nettool_capture::read_pcap;
use nettool_proxy::{
    ConsoleSink, DetectMode, Direction, PcapSink, Proxy, ProxyConfig, RuleSet, SharedSink,
};

type CliResult = Result<(), Box<dyn Error>>;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("proxy") => cmd_proxy(&args[1..]),
        Some("serve") => cmd_serve(&args[1..]),
        Some("analyze") => cmd_analyze(&args[1..]),
        Some("mcp") => mcp::run().map_err(|error| Box::new(error) as Box<dyn Error>),
        Some("version") | Some("--version") | Some("-V") => cmd_version(&args[1..]),
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

fn take_flags(args: &[String], name: &str) -> Vec<String> {
    args.iter()
        .enumerate()
        .filter(|(_, arg)| arg.as_str() == name)
        .filter_map(|(index, _)| args.get(index + 1).cloned())
        .collect()
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
    let listen_value = take_flag(args, "--listen").unwrap_or_else(|| "127.0.0.1:25565".to_string());
    let listen: SocketAddr = listen_value.parse().map_err(|_| {
        invalid(format!(
            "bad --listen address {listen_value}; expected host:port"
        ))
    })?;
    let target = take_flag(args, "--target").unwrap_or_else(|| "127.0.0.1:25566".to_string());
    let (target_host, target_port) = split_host_port(&target)?;
    let protocol_name = take_flag(args, "--protocol").unwrap_or_else(|| "auto".to_string());
    let protocol = DetectMode::parse(&protocol_name)
        .ok_or_else(|| invalid(format!("bad --protocol: {protocol_name}")))?;
    let pcap = take_flag(args, "--pcap").map(PathBuf::from);
    let quiet = has_flag(args, "--quiet");
    let json = has_flag(args, "--json");
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
    let rule_specs = take_flags(args, "--rule");
    if rule_specs.is_empty() && has_flag(args, "--rule") {
        return Err(invalid("--rule requires a value".to_string()));
    }
    let rules =
        RuleSet::parse(&rule_specs).map_err(|error| invalid(format!("bad --rule: {error}")))?;

    let mut sinks: Vec<SharedSink> = Vec::new();
    if let Some(path) = &pcap {
        sinks.push(Arc::new(Mutex::new(PcapSink::create(path)?)));
    }
    if json {
        sinks.push(Arc::new(Mutex::new(JsonSink::new(io::stdout()))));
    } else if !quiet {
        sinks.push(Arc::new(Mutex::new(ConsoleSink::new())));
    }

    let config = ProxyConfig {
        listen,
        target_host,
        target_port,
        protocol,
        sinks,
        verbose: !quiet,
        drop_direction,
        rules,
        duration,
    };
    if !json {
        println!("nazuko proxy");
        println!("  listen: {}", config.listen);
        println!("  target: {}:{}", config.target_host, config.target_port);
        if !config.rules.is_empty() {
            println!("  rules: {}", config.rules.len());
        }
    }
    let proxy = Proxy::bind(config)?;
    if !json {
        println!("  bound: {}", proxy.local_addr()?);
    }
    proxy.run()?;
    Ok(())
}

fn cmd_serve(args: &[String]) -> CliResult {
    let bind: SocketAddr = take_flag(args, "--bind")
        .unwrap_or_else(|| "127.0.0.1:0".to_string())
        .parse()
        .map_err(|_| invalid("bad --bind address".to_string()))?;
    let state_path = take_flag(args, "--state")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join(format!("nazuko-{}.state", std::process::id())));
    let daemon = daemon::Daemon::bind(bind, state_path.clone())?;
    println!("nazuko control server");
    println!("  bind: {}", daemon.local_addr()?);
    println!("  state: {}", state_path.display());
    daemon.run()?;
    Ok(())
}

fn cmd_version(args: &[String]) -> CliResult {
    if has_flag(args, "--json") {
        println!("{}", mcp::version_json().to_json_string());
    } else {
        println!("nazuko {}", env!("CARGO_PKG_VERSION"));
    }
    Ok(())
}

fn positional_path(args: &[String]) -> Option<&str> {
    let mut skip_value = false;
    for arg in args {
        if skip_value {
            skip_value = false;
            continue;
        }
        if arg == "--limit" {
            skip_value = true;
            continue;
        }
        if !arg.starts_with('-') {
            return Some(arg);
        }
    }
    None
}

fn cmd_analyze(args: &[String]) -> CliResult {
    let path = positional_path(args).ok_or_else(|| {
        invalid(
            "usage: nazuko analyze <file.pcap> [--hex] [--dissect] [--json] [--limit N]"
                .to_string(),
        )
    })?;
    let hex = has_flag(args, "--hex");
    let dissect = has_flag(args, "--dissect");
    let json = has_flag(args, "--json");
    let limit = match take_flag(args, "--limit") {
        Some(value) => value
            .parse::<usize>()
            .map_err(|_| invalid(format!("bad --limit: {value}")))?,
        None => 0,
    };
    if json {
        let output = mcp::analyze_pcap_json(path, limit, hex, dissect)?;
        println!("{}", output.to_json_string());
        return Ok(());
    }
    let file = File::open(path)?;
    let packets = read_pcap(file)?;
    println!("{} packets from {}", packets.len(), path);
    let take = if limit == 0 {
        packets.len()
    } else {
        limit.min(packets.len())
    };
    for (index, packet) in packets.iter().take(take).enumerate() {
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
        "nazuko {}\n\
         \n\
         USAGE:\n\
         \x20 nazuko proxy [options]\n\
         \x20 nazuko serve [--bind 127.0.0.1:0] [--state <file>]\n\
         \x20 nazuko analyze <file.pcap> [--hex] [--dissect] [--json] [--limit N]\n\
         \x20 nazuko mcp                 stdio MCP server (read-only tools)\n\
         \x20 nazuko version [--json]\n\
         \n\
         PROXY OPTIONS:\n\
         \x20 --listen <ip:port>       default 127.0.0.1:25565\n\
         \x20 --target <host:port>     default 127.0.0.1:25566\n\
         \x20 --protocol <mode>        auto | mc | http | tls | dns | ws | quic | raw\n\
         \x20 --pcap <file>            write a real pcap\n\
         \x20 --duration <seconds>     stop accepting after N seconds\n\
         \x20 --drop-direction <dir>   drop c2s or s2c (still captured)\n\
         \x20 --rule <expr>            repeatable drop/allow predicate\n\
         \x20 --quiet                  suppress console per-packet output\n\
         \x20 --json                   emit newline-delimited packet JSON\n\
         \n\
         RULE EXAMPLES:\n\
         \x20 --rule 'drop dir=c2s proto=mc contains=hello'\n\
         \x20 --rule 'drop proto=http prefix=GET'\n\
         \x20 --rule 'drop dir=s2c min_len=4096'",
        env!("CARGO_PKG_VERSION")
    );
}
