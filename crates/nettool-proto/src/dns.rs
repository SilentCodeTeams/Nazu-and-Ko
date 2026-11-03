//! DNS query parser (Phase 1).
//!
//! Parses the question section for a standard query. Compression pointers
//! and answer sections are not decoded yet.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsInfo {
    pub id: u16,
    pub is_response: bool,
    pub qname: String,
    pub qtype: u16,
    pub qclass: u16,
}

pub fn inspect(first: &[u8]) -> Option<DnsInfo> {
    if first.len() < 12 {
        return None;
    }
    let id = u16::from_be_bytes([first[0], first[1]]);
    let flags = u16::from_be_bytes([first[2], first[3]]);
    let is_response = flags & 0x8000 != 0;
    let questions = u16::from_be_bytes([first[4], first[5]]);
    if questions == 0 {
        return None;
    }

    let mut pos = 12;
    let mut labels = Vec::new();
    for _ in 0..128 {
        let len = *first.get(pos)? as usize;
        pos += 1;
        if len == 0 {
            break;
        }
        if len > 63 || pos + len > first.len() {
            return None;
        }
        let label = &first[pos..pos + len];
        pos += len;
        if !label
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-' || *byte == b'_')
        {
            return None;
        }
        labels.push(std::str::from_utf8(label).ok()?.to_string());
    }
    if labels.is_empty() {
        return None;
    }
    if pos + 4 > first.len() {
        return None;
    }
    let qtype = u16::from_be_bytes([first[pos], first[pos + 1]]);
    let qclass = u16::from_be_bytes([first[pos + 2], first[pos + 3]]);
    Some(DnsInfo {
        id,
        is_response,
        qname: labels.join("."),
        qtype,
        qclass,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(name: &str, qtype: u16) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&0x1234u16.to_be_bytes());
        out.extend_from_slice(&0x0100u16.to_be_bytes());
        out.extend_from_slice(&1u16.to_be_bytes());
        out.extend_from_slice(&0u16.to_be_bytes());
        out.extend_from_slice(&0u16.to_be_bytes());
        out.extend_from_slice(&0u16.to_be_bytes());
        for label in name.split('.') {
            out.push(label.len() as u8);
            out.extend_from_slice(label.as_bytes());
        }
        out.push(0);
        out.extend_from_slice(&qtype.to_be_bytes());
        out.extend_from_slice(&1u16.to_be_bytes());
        out
    }

    #[test]
    fn parses_query() {
        let info = inspect(&query("example.com", 1)).expect("dns");
        assert_eq!(info.qname, "example.com");
        assert_eq!(info.qtype, 1);
        assert_eq!(info.qclass, 1);
        assert!(!info.is_response);
    }

    #[test]
    fn rejects_junk() {
        assert!(inspect(&[0u8; 11]).is_none());
        assert!(inspect(&[0u8; 12]).is_none());
        assert!(inspect(b"GET / HTTP/1.1\r\n").is_none());
    }
}
