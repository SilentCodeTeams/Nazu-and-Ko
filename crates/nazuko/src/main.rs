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
mod i18n;
mod json_sink;
mod language;
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
    let mut args: Vec<String> = env::args().skip(1).collect();
    let cli_lang_file = take_option(&mut args, "--lang-file");
    let cli_lang = take_option(&mut args, "--lang");
    let command = args.first().map(String::as_str);

    if command == Some("mcp")
        || command == Some("version")
        || command == Some("--version")
        || command == Some("-V")
    {
        i18n::init_code("en");
    } else {
        match language::resolve(cli_lang_file, cli_lang) {
            Ok(language::Selection::Builtin(code)) => i18n::init_code(&code),
            Ok(language::Selection::Custom(path)) => {
                if let Err(error) = i18n::init_custom(&path) {
                    eprintln!("Error: {error}");
                    std::process::exit(2);
                }
            }
            Err(error) => {
                eprintln!("Error: {error}");
                std::process::exit(2);
            }
        }
    }

    let result = match command {
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
            Err(invalid(i18n::tr_args(
                "cli.error.unknownCommand",
                &[("command", other)],
            )))
        }
    };
    if let Err(e) = result {
        eprintln!("{}{e}", i18n::t("cli.error.prefix"));
        std::process::exit(1);
    }
}

fn take_option(args: &mut Vec<String>, name: &str) -> Option<String> {
    let mut index = 0;
    while index < args.len() {
        if args[index] == name {
            if index + 1 < args.len() {
                let value = args[index + 1].clone();
                args.drain(index..=index + 1);
                return Some(value);
            }
            args.remove(index);
            return None;
        }
        if let Some(value) = args[index].strip_prefix(&format!("{name}=")) {
            let value = value.to_string();
            args.remove(index);
            return Some(value);
        }
        index += 1;
    }
    None
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

fn split_host_port(value: &str, flag: &str) -> Result<(String, u16), Box<dyn Error>> {
    let (host, port) = value.rsplit_once(':').ok_or_else(|| {
        invalid(i18n::tr_args(
            "cli.error.hostPort",
            &[("flag", flag), ("value", value)],
        ))
    })?;
    if host.is_empty() {
        return Err(invalid(i18n::tr_args(
            "cli.error.hostEmpty",
            &[("flag", flag), ("value", value)],
        )));
    }
    let port = port.parse::<u16>().map_err(|_| {
        invalid(i18n::tr_args(
            "cli.error.port",
            &[("flag", flag), ("value", port)],
        ))
    })?;
    Ok((host.to_string(), port))
}

fn cmd_proxy(args: &[String]) -> CliResult {
    let listen_value = take_flag(args, "--listen").unwrap_or_else(|| "127.0.0.1:25565".to_string());
    let listen: SocketAddr = listen_value.parse().map_err(|_| {
        invalid(i18n::tr_args(
            "cli.error.listen",
            &[("flag", "--listen"), ("value", &listen_value)],
        ))
    })?;
    let target = take_flag(args, "--target").unwrap_or_else(|| "127.0.0.1:25566".to_string());
    let (target_host, target_port) = split_host_port(&target, "--target")?;
    let protocol_name = take_flag(args, "--protocol").unwrap_or_else(|| "auto".to_string());
    let protocol = DetectMode::parse(&protocol_name).ok_or_else(|| {
        invalid(i18n::tr_args(
            "cli.error.protocol",
            &[("value", &protocol_name)],
        ))
    })?;
    let pcap = take_flag(args, "--pcap").map(PathBuf::from);
    let quiet = has_flag(args, "--quiet");
    let json = has_flag(args, "--json");
    let drop_direction = match take_flag(args, "--drop-direction") {
        Some(value) => Some(Direction::parse(&value).ok_or_else(|| {
            invalid(i18n::tr_args(
                "cli.error.dropDirection",
                &[("value", &value)],
            ))
        })?),
        None => None,
    };
    let duration = match take_flag(args, "--duration") {
        Some(value) => Some(Duration::from_secs(value.parse::<u64>().map_err(|_| {
            invalid(i18n::tr_args("cli.error.duration", &[("value", &value)]))
        })?)),
        None => None,
    };
    let rule_specs = take_flags(args, "--rule");
    if rule_specs.is_empty() && has_flag(args, "--rule") {
        return Err(invalid(i18n::t("cli.error.ruleMissing")));
    }
    let rules = RuleSet::parse(&rule_specs).map_err(|error| {
        invalid(i18n::tr_args(
            "cli.error.rule",
            &[("error", &error.to_string())],
        ))
    })?;

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
        println!("{}", i18n::t("cli.proxy.title"));
        println!(
            "{}",
            i18n::tr_args("cli.proxy.listen", &[("value", &config.listen.to_string())])
        );
        println!(
            "{}",
            i18n::tr_args(
                "cli.proxy.target",
                &[(
                    "value",
                    &format!("{}:{}", config.target_host, config.target_port)
                )]
            )
        );
        if !config.rules.is_empty() {
            println!(
                "{}",
                i18n::tr_args(
                    "cli.proxy.rules",
                    &[("value", &config.rules.len().to_string())]
                )
            );
        }
    }
    let proxy = Proxy::bind(config).map_err(|error| {
        invalid(i18n::tr_args(
            "cli.error.proxyBind",
            &[
                ("address", &listen.to_string()),
                ("error", &error.to_string()),
            ],
        ))
    })?;
    if !json {
        println!(
            "{}",
            i18n::tr_args(
                "cli.proxy.bound",
                &[("value", &proxy.local_addr()?.to_string())]
            )
        );
    }
    proxy.run()?;
    Ok(())
}

fn cmd_serve(args: &[String]) -> CliResult {
    let bind_value = take_flag(args, "--bind").unwrap_or_else(|| "127.0.0.1:0".to_string());
    let bind: SocketAddr = bind_value.parse().map_err(|_| {
        invalid(i18n::tr_args(
            "cli.error.bind",
            &[("flag", "--bind"), ("value", &bind_value)],
        ))
    })?;
    let state_path = take_flag(args, "--state")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join(format!("nazuko-{}.state", std::process::id())));
    let daemon = daemon::Daemon::bind(bind, state_path.clone()).map_err(|error| {
        invalid(i18n::tr_args(
            "cli.error.serveBind",
            &[
                ("address", &bind.to_string()),
                ("error", &error.to_string()),
            ],
        ))
    })?;
    println!("{}", i18n::t("cli.serve.title"));
    println!(
        "{}",
        i18n::tr_args(
            "cli.serve.bind",
            &[("value", &daemon.local_addr()?.to_string())]
        )
    );
    println!(
        "{}",
        i18n::tr_args(
            "cli.serve.state",
            &[("value", &state_path.display().to_string())]
        )
    );
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
    let path = positional_path(args).ok_or_else(|| invalid(i18n::t("cli.error.analyzeUsage")))?;
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
    let file = File::open(path).map_err(|error| {
        invalid(i18n::tr_args(
            "cli.error.analyzeOpen",
            &[("path", path), ("error", &error.to_string())],
        ))
    })?;
    let packets = read_pcap(file).map_err(|error| {
        invalid(i18n::tr_args(
            "cli.error.analyzeParse",
            &[("path", path), ("error", &error.to_string())],
        ))
    })?;
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
        "{}",
        i18n::tr_args("cli.help", &[("version", env!("CARGO_PKG_VERSION"))])
    );
}
