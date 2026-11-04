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

//! Newline-delimited JSON packet events for machine consumers.

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::time::UNIX_EPOCH;

use nettool_control::json::Json;
use nettool_proxy::{EventSink, PacketEvent};

pub struct JsonSink<W: Write> {
    out: W,
}

impl<W: Write> JsonSink<W> {
    pub fn new(out: W) -> Self {
        Self { out }
    }
}

impl<W: Write + Send> EventSink for JsonSink<W> {
    fn on_event(&mut self, event: &PacketEvent) -> io::Result<()> {
        let duration = event
            .timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let mut fields = vec![
            ("event", s("packet")),
            ("id", number(event.id as f64)),
            ("timestamp_sec", number(duration.as_secs() as f64)),
            ("timestamp_usec", number(duration.subsec_micros() as f64)),
            ("direction", s(event.direction.as_str())),
            ("protocol", s(event.protocol.as_str())),
            ("src", s(&event.src.to_string())),
            ("dst", s(&event.dst.to_string())),
            ("seq", number(event.seq as f64)),
            ("ack", number(event.ack as f64)),
            ("length", number(event.data.len() as f64)),
            ("dropped", Json::Bool(event.dropped)),
            ("frame_count", number(event.frames.len() as f64)),
            ("payload_preview", s(&preview(&event.data))),
        ];
        if let Some(rule) = &event.matched_rule {
            fields.push(("matched_rule", s(rule)));
        }
        self.out
            .write_all(obj(fields).to_json_string().as_bytes())?;
        self.out.write_all(b"\n")?;
        self.out.flush()
    }
}

fn obj(items: Vec<(&str, Json)>) -> Json {
    let mut map = BTreeMap::new();
    for (key, value) in items {
        map.insert(key.to_string(), value);
    }
    Json::Object(map)
}

fn s(value: &str) -> Json {
    Json::String(value.to_string())
}

fn number(value: f64) -> Json {
    Json::Number(value)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::{Duration, UNIX_EPOCH};

    use nettool_proto::Protocol;
    use nettool_proxy::Direction;

    fn packet() -> PacketEvent {
        PacketEvent {
            id: 7,
            timestamp: UNIX_EPOCH + Duration::from_micros(1_500_000),
            direction: Direction::ClientToServer,
            protocol: Protocol::Http,
            src: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000),
            dst: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            seq: 1,
            ack: 2,
            data: b"GET / HTTP/1.1".to_vec(),
            frames: Vec::new(),
            dropped: false,
            matched_rule: None,
        }
    }

    #[test]
    fn writes_one_json_line_per_event() {
        let mut sink = JsonSink::new(Vec::new());
        sink.on_event(&packet()).expect("event");
        let output = String::from_utf8(sink.out).expect("utf8");
        assert_eq!(output.lines().count(), 1);
        assert!(output.contains("\"event\":\"packet\""));
        assert!(output.contains("\"protocol\":\"http\""));
    }
}
