//! Synthetic Ethernet/IPv4/TCP frame construction.
//!
//! The proxy knows the client and target endpoints and the TCP payload, but
//! not the original L2 headers. We build a plausible frame so Wireshark can
//! reassemble the stream: fixed synthetic MACs, IPv4, TCP with PSH+ACK.

use std::net::Ipv4Addr;

/// Internet checksum (RFC 1071).
pub fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut chunks = data.chunks_exact(2);
    for chunk in &mut chunks {
        sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
    }
    if let Some(&last) = chunks.remainder().first() {
        sum += (last as u32) << 8;
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

/// Build a complete Ethernet frame carrying one TCP segment.
pub fn build_tcp_frame(
    src_ip: Ipv4Addr,
    dst_ip: Ipv4Addr,
    src_port: u16,
    dst_port: u16,
    seq: u32,
    ack: u32,
    ident: u16,
    payload: &[u8],
) -> Vec<u8> {
    let mut tcp = Vec::with_capacity(20 + payload.len());
    tcp.extend_from_slice(&src_port.to_be_bytes());
    tcp.extend_from_slice(&dst_port.to_be_bytes());
    tcp.extend_from_slice(&seq.to_be_bytes());
    tcp.extend_from_slice(&ack.to_be_bytes());
    tcp.push(5 << 4); // data offset: 5 * 4 = 20 bytes
    tcp.push(0x18); // PSH | ACK
    tcp.extend_from_slice(&65535u16.to_be_bytes());
    tcp.extend_from_slice(&[0, 0]); // checksum placeholder
    tcp.extend_from_slice(&[0, 0]); // urgent pointer
    tcp.extend_from_slice(payload);

    let mut pseudo = Vec::with_capacity(12 + tcp.len());
    pseudo.extend_from_slice(&src_ip.octets());
    pseudo.extend_from_slice(&dst_ip.octets());
    pseudo.push(0);
    pseudo.push(6);
    pseudo.extend_from_slice(&(tcp.len() as u16).to_be_bytes());
    pseudo.extend_from_slice(&tcp);
    let tcp_csum = checksum(&pseudo);
    tcp[16..18].copy_from_slice(&tcp_csum.to_be_bytes());

    let total_len = 20 + tcp.len();
    let mut ip = Vec::with_capacity(20);
    ip.push(0x45); // IPv4, IHL 5
    ip.push(0); // DSCP/ECN
    ip.extend_from_slice(&(total_len as u16).to_be_bytes());
    ip.extend_from_slice(&ident.to_be_bytes());
    ip.extend_from_slice(&0x4000u16.to_be_bytes()); // Don't Fragment
    ip.push(64); // TTL
    ip.push(6); // TCP
    ip.extend_from_slice(&[0, 0]); // checksum placeholder
    ip.extend_from_slice(&src_ip.octets());
    ip.extend_from_slice(&dst_ip.octets());
    let ip_csum = checksum(&ip);
    ip[10..12].copy_from_slice(&ip_csum.to_be_bytes());

    let mut frame = Vec::with_capacity(14 + ip.len() + tcp.len());
    frame.extend_from_slice(&[0x02, 0x00, 0x00, 0x00, 0x00, 0x01]); // dst MAC
    frame.extend_from_slice(&[0x02, 0x00, 0x00, 0x00, 0x00, 0x02]); // src MAC
    frame.extend_from_slice(&0x0800u16.to_be_bytes()); // IPv4
    frame.extend_from_slice(&ip);
    frame.extend_from_slice(&tcp);
    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_empty() {
        assert_eq!(checksum(&[]), 0xffff);
    }

    #[test]
    fn frame_layout() {
        let src = Ipv4Addr::new(10, 0, 0, 1);
        let dst = Ipv4Addr::new(10, 0, 0, 2);
        let frame = build_tcp_frame(src, dst, 1234, 80, 7, 9, 1, b"hello");
        assert_eq!(&frame[12..14], &[0x08, 0x00]);
        assert_eq!(frame[14] >> 4, 4);
        assert_eq!(frame[14] & 0x0f, 5);
        assert_eq!(&frame[26..30], &src.octets());
        assert_eq!(&frame[30..34], &dst.octets());
        let tcp_off = 34;
        assert_eq!(
            u16::from_be_bytes([frame[tcp_off], frame[tcp_off + 1]]),
            1234
        );
        assert_eq!(
            u16::from_be_bytes([frame[tcp_off + 2], frame[tcp_off + 3]]),
            80
        );
        assert_eq!(&frame[tcp_off + 20..], b"hello");
    }
}
