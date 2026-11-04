//! Minimal MCP (Model Context Protocol) stdio server.
//!
//! This slice is intentionally read-only: it exposes the CLI version and
//! offline pcap analysis. Write tools will be added only after the control
//! channel has an explicit, auditable operation model.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, Write};

use nettool_capture::read_pcap;
use nettool_control::json::Json;

const LATEST_PROTOCOL: &str = "2025-06-18";
const SUPPORTED_PROTOCOLS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];
const MAX_LINE_BYTES: usize = 2 * 1024 * 1024;

pub(crate) fn run() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if line.len() > MAX_LINE_BYTES {
            let response = error_response(Json::Null, RpcError::new(-32600, "request too large"));
            write_message(&mut stdout, &response)?;
            continue;
        }
        if let Some(response) = handle_line(&line) {
            write_message(&mut stdout, &response)?;
        }
    }
    Ok(())
}

fn write_message(stdout: &mut impl Write, value: &Json) -> io::Result<()> {
    stdout.write_all(value.to_json_string().as_bytes())?;
    stdout.write_all(b"\n")?;
    stdout.flush()
}

fn handle_line(line: &str) -> Option<Json> {
    let request = match Json::parse(line) {
        Ok(value) => value,
        Err(error) => {
            return Some(error_response(
                Json::Null,
                RpcError::new(-32700, format!("parse error: {error}")),
            ))
        }
    };
    handle_request(&request)
}

fn handle_request(request: &Json) -> Option<Json> {
    let method = request.get("method").and_then(Json::as_str)?;
    let id = request.get("id").cloned();
    let result = match method {
        "initialize" => initialize(request),
        "tools/list" => tools_list(),
        "tools/call" => tools_call(request),
        "resources/list" => Ok(obj(vec![("resources", Json::Array(Vec::new()))])),
        "prompts/list" => Ok(obj(vec![("prompts", Json::Array(Vec::new()))])),
        "ping" => Ok(obj(Vec::new())),
        "notifications/initialized" | "notifications/cancelled" => {
            return None;
        }
        other => Err(RpcError::new(-32601, format!("method not found: {other}"))),
    };
    let id = id?;
    Some(match result {
        Ok(value) => obj(vec![("jsonrpc", s("2.0")), ("id", id), ("result", value)]),
        Err(error) => error_response(id, error),
    })
}

fn initialize(request: &Json) -> Result<Json, RpcError> {
    let params = request.get("params").and_then(Json::as_object);
    let requested = params
        .and_then(|params| params.get("protocolVersion"))
        .and_then(Json::as_str);
    let protocol = requested
        .filter(|value| SUPPORTED_PROTOCOLS.contains(value))
        .unwrap_or(LATEST_PROTOCOL);
    Ok(obj(vec![
        ("protocolVersion", s(protocol)),
        (
            "capabilities",
            obj(vec![(
                "tools",
                obj(vec![("listChanged", Json::Bool(false))]),
            )]),
        ),
        ("serverInfo", server_info()),
        (
            "instructions",
            s("Nazu&Ko local analyzer. Tools are read-only. Pass absolute pcap paths."),
        ),
    ]))
}

fn server_info() -> Json {
    obj(vec![
        ("name", s("nazuko")),
        ("version", s(env!("CARGO_PKG_VERSION"))),
    ])
}

pub(crate) fn version_json() -> Json {
    obj(vec![
        ("name", s("nazuko")),
        ("version", s(env!("CARGO_PKG_VERSION"))),
        ("license", s("AGPL-3.0-or-later")),
        (
            "repository",
            s("https://github.com/SilentCodeTeams/Nazu-and-Ko"),
        ),
    ])
}

fn tools_list() -> Result<Json, RpcError> {
    Ok(obj(vec![(
        "tools",
        Json::Array(vec![version_tool(), analyze_tool()]),
    )]))
}

fn version_tool() -> Json {
    tool(
        "nazuko_version",
        "Nazu&Ko version",
        "Return the nazuko version, license, and repository.",
        empty_schema(),
        true,
    )
}

fn analyze_tool() -> Json {
    let schema = obj(vec![
        ("type", s("object")),
        (
            "properties",
            obj(vec![
                (
                    "path",
                    obj(vec![
                        ("type", s("string")),
                        ("description", s("Path to a classic pcap file.")),
                    ]),
                ),
                (
                    "limit",
                    obj(vec![
                        ("type", s("integer")),
                        ("minimum", number(1.0)),
                        ("maximum", number(10000.0)),
                        ("default", number(100.0)),
                        ("description", s("Maximum packets to return.")),
                    ]),
                ),
                (
                    "hex",
                    obj(vec![
                        ("type", s("boolean")),
                        ("default", Json::Bool(false)),
                        ("description", s("Include the payload as lowercase hex.")),
                    ]),
                ),
                (
                    "dissect",
                    obj(vec![
                        ("type", s("boolean")),
                        ("default", Json::Bool(true)),
                        ("description", s("Detect protocol and version.")),
                    ]),
                ),
            ]),
        ),
        ("required", Json::Array(vec![s("path")])),
        ("additionalProperties", Json::Bool(false)),
    ]);
    tool(
        "nazuko_analyze_pcap",
        "Analyze pcap",
        "Read a classic pcap file and return packet summaries. Read-only.",
        schema,
        true,
    )
}

fn tool(name: &str, title: &str, description: &str, input_schema: Json, read_only: bool) -> Json {
    obj(vec![
        ("name", s(name)),
        ("title", s(title)),
        ("description", s(description)),
        ("inputSchema", input_schema),
        (
            "annotations",
            obj(vec![
                ("title", s(title)),
                ("readOnlyHint", Json::Bool(read_only)),
                ("destructiveHint", Json::Bool(false)),
                ("idempotentHint", Json::Bool(true)),
                ("openWorldHint", Json::Bool(false)),
            ]),
        ),
    ])
}

fn empty_schema() -> Json {
    obj(vec![
        ("type", s("object")),
        ("properties", obj(Vec::new())),
        ("additionalProperties", Json::Bool(false)),
    ])
}

fn tools_call(request: &Json) -> Result<Json, RpcError> {
    let params = request
        .get("params")
        .and_then(Json::as_object)
        .ok_or_else(|| RpcError::new(-32602, "missing params"))?;
    let name = params
        .get("name")
        .and_then(Json::as_str)
        .ok_or_else(|| RpcError::new(-32602, "missing tool name"))?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(empty_object);
    match name {
        "nazuko_version" => Ok(tool_success(version_json())),
        "nazuko_analyze_pcap" => {
            let path = arguments
                .get("path")
                .and_then(Json::as_str)
                .ok_or_else(|| RpcError::new(-32602, "missing path"))?;
            let limit = arguments
                .get("limit")
                .and_then(Json::as_u64)
                .unwrap_or(100)
                .clamp(1, 10_000) as usize;
            let hex = arguments
                .get("hex")
                .and_then(Json::as_bool)
                .unwrap_or(false);
            let dissect = arguments
                .get("dissect")
                .and_then(Json::as_bool)
                .unwrap_or(true);
            match analyze_pcap_json(path, limit, hex, dissect) {
                Ok(value) => Ok(tool_success(value)),
                Err(error) => Ok(tool_error(error)),
            }
        }
        other => Err(RpcError::new(-32602, format!("unknown tool: {other}"))),
    }
}

pub(crate) fn analyze_pcap_json(
    path: &str,
    limit: usize,
    include_hex: bool,
    dissect: bool,
) -> Result<Json, String> {
    let file = File::open(path).map_err(|error| format!("open {path}: {error}"))?;
    let packets = read_pcap(file).map_err(|error| format!("read {path}: {error}"))?;
    let total = packets.len();
    let take = if limit == 0 { total } else { limit.min(total) };
    let mut rows = Vec::with_capacity(take);
    for (index, packet) in packets.iter().take(take).enumerate() {
        let mut fields = vec![
            ("index", number((index + 1) as f64)),
            ("timestamp_sec", number(packet.ts_sec as f64)),
            ("timestamp_usec", number(packet.ts_usec as f64)),
            ("src_ip", s(&packet.src_ip.to_string())),
            ("src_port", number(packet.src_port as f64)),
            ("dst_ip", s(&packet.dst_ip.to_string())),
            ("dst_port", number(packet.dst_port as f64)),
            ("seq", number(packet.seq as f64)),
            ("ack", number(packet.ack as f64)),
            ("payload_length", number(packet.payload.len() as f64)),
            ("payload_preview", s(&preview(&packet.payload))),
        ];
        if include_hex {
            fields.push(("payload_hex", s(&hex_string(&packet.payload))));
        }
        if dissect && !packet.payload.is_empty() {
            let found = nettool_proto::inspect(&packet.payload);
            fields.push((
                "detected",
                obj(vec![
                    ("protocol", s(found.protocol.as_str())),
                    ("version", opt_string(found.version.as_deref())),
                    ("detail", opt_string(found.detail.as_deref())),
                ]),
            ));
        }
        rows.push(obj(fields));
    }
    Ok(obj(vec![
        ("path", s(path)),
        ("packet_count", number(total as f64)),
        ("returned", number(take as f64)),
        ("truncated", Json::Bool(take < total)),
        ("packets", Json::Array(rows)),
    ]))
}

fn tool_success(value: Json) -> Json {
    let text = value.to_json_string();
    obj(vec![
        (
            "content",
            Json::Array(vec![obj(vec![("type", s("text")), ("text", s(&text))])]),
        ),
        ("structuredContent", value),
        ("isError", Json::Bool(false)),
    ])
}

fn tool_error(message: String) -> Json {
    obj(vec![
        (
            "content",
            Json::Array(vec![obj(vec![("type", s("text")), ("text", s(&message))])]),
        ),
        ("isError", Json::Bool(true)),
    ])
}

fn error_response(id: Json, error: RpcError) -> Json {
    let mut error_fields = vec![
        ("code", number(error.code as f64)),
        ("message", s(&error.message)),
    ];
    if let Some(data) = error.data {
        error_fields.push(("data", data));
    }
    obj(vec![
        ("jsonrpc", s("2.0")),
        ("id", id),
        ("error", obj(error_fields)),
    ])
}

#[derive(Debug)]
struct RpcError {
    code: i64,
    message: String,
    data: Option<Json>,
}

impl RpcError {
    fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
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

fn opt_string(value: Option<&str>) -> Json {
    value.map(s).unwrap_or(Json::Null)
}

fn empty_object() -> Json {
    obj(Vec::new())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn request(line: &str) -> Json {
        handle_line(line).expect("response")
    }

    #[test]
    fn initialize_negotiates_protocol() {
        let response = request(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#,
        );
        assert_eq!(response.get("jsonrpc").and_then(Json::as_str), Some("2.0"));
        let result = response.get("result").expect("result");
        assert_eq!(
            result.get("protocolVersion").and_then(Json::as_str),
            Some("2025-06-18")
        );
        assert_eq!(
            result
                .get("serverInfo")
                .and_then(|value| value.get("name"))
                .and_then(Json::as_str),
            Some("nazuko")
        );
    }

    #[test]
    fn tools_list_exposes_read_only_tools() {
        let response = request(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#);
        let tools = response
            .get("result")
            .and_then(|value| value.get("tools"))
            .and_then(Json::as_array)
            .expect("tools");
        assert!(tools.iter().any(|tool| {
            tool.get("name").and_then(Json::as_str) == Some("nazuko_analyze_pcap")
        }));
        assert!(tools
            .iter()
            .any(|tool| { tool.get("name").and_then(Json::as_str) == Some("nazuko_version") }));
    }

    #[test]
    fn unknown_method_returns_rpc_error() {
        let response = request(r#"{"jsonrpc":"2.0","id":3,"method":"unknown"}"#);
        let error = response.get("error").expect("error");
        assert!(error.to_json_string().contains("\"code\":-32601"));
    }

    #[test]
    fn tool_error_is_a_tool_result() {
        let response = request(
            r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"nazuko_analyze_pcap","arguments":{}}}"#,
        );
        assert_eq!(
            response
                .get("result")
                .and_then(|value| value.get("isError"))
                .and_then(Json::as_bool),
            None
        );
        assert!(response.get("error").is_some());
    }
}
