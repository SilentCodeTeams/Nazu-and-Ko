//! HTTP version-aware first-message parser.
//!
//! Phase 1 scope:
//! - HTTP/0.9, HTTP/1.0, HTTP/1.1 request and response start lines
//! - HTTP/2 connection preface detection
//! - Header parsing and WebSocket upgrade detection
//!
//! HTTP/2 frame/HPACK decoding and HTTP/3 (QUIC) decoding are later phases.
//! The detector only needs the first bytes; the parser tolerates an
//! incomplete header block and returns what it has.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpVersion {
    Http09,
    Http10,
    Http11,
    Http2,
}

impl HttpVersion {
    pub fn label(self) -> &'static str {
        match self {
            HttpVersion::Http09 => "HTTP/0.9",
            HttpVersion::Http10 => "HTTP/1.0",
            HttpVersion::Http11 => "HTTP/1.1",
            HttpVersion::Http2 => "HTTP/2",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpInfo {
    pub version: HttpVersion,
    pub is_request: bool,
    pub method: Option<String>,
    pub target: Option<String>,
    pub status: Option<u16>,
    pub headers: Vec<(String, String)>,
    pub upgraded: Option<String>,
}

const H2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
const MAX_PARSE: usize = 64 * 1024;
const MAX_HEADERS: usize = 200;

pub fn inspect(first: &[u8]) -> Option<HttpInfo> {
    if first.starts_with(H2_PREFACE) {
        return Some(HttpInfo {
            version: HttpVersion::Http2,
            is_request: true,
            method: Some("PRI".to_string()),
            target: Some("*".to_string()),
            status: None,
            headers: Vec::new(),
            upgraded: None,
        });
    }
    if first.is_empty() {
        return None;
    }
    let text = String::from_utf8_lossy(&first[..first.len().min(MAX_PARSE)]);
    let line_end = text.find("\r\n").or_else(|| text.find('\n'))?;
    let line = &text[..line_end];
    if line.is_empty() {
        return None;
    }

    if line.starts_with("HTTP/") {
        let mut parts = line.split(' ');
        let version = parse_version(parts.next()?)?;
        if version == HttpVersion::Http2 {
            return None;
        }
        let status = parts.next()?.parse::<u16>().ok();
        let headers = parse_headers(&text, line_end);
        return Some(HttpInfo {
            version,
            is_request: false,
            method: None,
            target: None,
            status,
            headers,
            upgraded: None,
        });
    }

    let mut parts = line.split(' ');
    let method = parts.next()?;
    if !is_method(method) {
        return None;
    }
    let target = parts.next()?;
    if target.is_empty() {
        return None;
    }
    let version = match parts.next() {
        Some(raw) => parse_version(raw)?,
        None => HttpVersion::Http09,
    };
    let headers = parse_headers(&text, line_end);
    let upgraded = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("upgrade"))
        .map(|(_, value)| value.clone());
    Some(HttpInfo {
        version,
        is_request: true,
        method: Some(method.to_string()),
        target: Some(target.to_string()),
        status: None,
        headers,
        upgraded,
    })
}

fn parse_version(raw: &str) -> Option<HttpVersion> {
    match raw {
        "HTTP/0.9" => Some(HttpVersion::Http09),
        "HTTP/1.0" => Some(HttpVersion::Http10),
        "HTTP/1.1" => Some(HttpVersion::Http11),
        "HTTP/2.0" | "HTTP/2" => Some(HttpVersion::Http2),
        _ => None,
    }
}

fn is_method(method: &str) -> bool {
    !method.is_empty()
        && method.len() <= 16
        && method
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

fn parse_headers(text: &str, line_end: usize) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    let block = match text[line_end..].find("\r\n\r\n") {
        Some(0) => "",
        Some(index) => &text[line_end + 2..line_end + index],
        None => match text[line_end..].find("\n\n") {
            Some(0) => "",
            Some(index) => &text[line_end + 1..line_end + index],
            None => return headers,
        },
    };
    for line in block.split("\r\n").flat_map(|part| part.split('\n')) {
        if headers.len() >= MAX_HEADERS {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            headers.push((name.to_string(), value.trim().to_string()));
        }
    }
    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http11_request() {
        let raw = b"GET /api?x=1 HTTP/1.1\r\nHost: example.com\r\nAccept: */*\r\n\r\n";
        let info = inspect(raw).expect("http");
        assert_eq!(info.version, HttpVersion::Http11);
        assert!(info.is_request);
        assert_eq!(info.method.as_deref(), Some("GET"));
        assert_eq!(info.target.as_deref(), Some("/api?x=1"));
        assert_eq!(info.headers.len(), 2);
        assert_eq!(info.upgraded, None);
    }

    #[test]
    fn http10_response() {
        let raw = b"HTTP/1.0 200 OK\r\nContent-Length: 0\r\n\r\n";
        let info = inspect(raw).expect("http");
        assert_eq!(info.version, HttpVersion::Http10);
        assert!(!info.is_request);
        assert_eq!(info.status, Some(200));
    }

    #[test]
    fn http09_request() {
        let raw = b"GET /index.html\r\n";
        let info = inspect(raw).expect("http");
        assert_eq!(info.version, HttpVersion::Http09);
        assert_eq!(info.method.as_deref(), Some("GET"));
    }

    #[test]
    fn http2_preface() {
        let info = inspect(H2_PREFACE).expect("h2");
        assert_eq!(info.version, HttpVersion::Http2);
    }

    #[test]
    fn websocket_upgrade() {
        let raw = b"GET /chat HTTP/1.1\r\nHost: example.com\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\r\n";
        let info = inspect(raw).expect("ws");
        assert_eq!(info.version, HttpVersion::Http11);
        assert_eq!(info.upgraded.as_deref(), Some("websocket"));
    }

    #[test]
    fn incomplete_start_line_is_none() {
        assert!(inspect(b"GET /").is_none());
        assert!(inspect(b"not http").is_none());
    }
}
