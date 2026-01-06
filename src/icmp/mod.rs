// src/icmp/mod.rs

use crate::packet::PacketMeta;
use std::net::Ipv6Addr;
use tracing::debug;

/// Compute ICMPv6 checksum with pseudo‑header.
fn icmpv6_checksum(src: Ipv6Addr, dst: Ipv6Addr, icmp: &[u8]) -> u16 {
    // Pseudo‑header: src (16), dst (16), payload length (4), zeros (3), next header (1)
    let mut sum: u32 = 0;
    for chunk in src.octets().chunks(2) {
        let word = u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
        sum += word;
    }
    for chunk in dst.octets().chunks(2) {
        let word = u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
        sum += word;
    }
    // payload length
    let len = icmp.len() as u32;
    sum += (len >> 16) & 0xFFFF; // high 16 bits (should be zero)
    sum += len & 0xFFFF;
    // three zero bytes + next header (58 for ICMPv6)
    sum += 58;
    // ICMP payload
    let mut i = 0;
    while i + 1 < icmp.len() {
        let word = u16::from_be_bytes([icmp[i], icmp[i + 1]]) as u32;
        sum += word;
        i += 2;
    }
    if i < icmp.len() {
        // odd length, pad with zero
        let word = (icmp[i] as u32) << 8;
        sum += word;
    }
    // add carries
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

/// Generate a minimal ICMPv6 error packet.
/// `error_type` and `code` follow the ICMPv6 specification.
/// Returns a full IPv6 packet containing the ICMPv6 message and as much of the original
/// packet as fits (up to the IPv6 minimum MTU of 1280 bytes).
pub fn generate_icmpv6_error(packet: &PacketMeta, error_type: u8, code: u8) -> Vec<u8> {
    debug!("Generating ICMPv6 error type {} code {}", error_type, code);
    let mut buf: Vec<u8> = Vec::new();
    // IPv6 header (40 bytes)
    // Version (6), Traffic Class & Flow Label = 0
    buf.extend_from_slice(&[0x60, 0, 0, 0]);
    // Payload length placeholder (will be set later)
    let payload_len_pos = buf.len();
    buf.extend_from_slice(&[0, 0]);
    // Next Header = 58 (ICMPv6)
    buf.push(58);
    // Hop Limit = 64 (arbitrary)
    buf.push(64);
    // Source address = destination of the original packet (router)
    let src = match packet.dst_ip {
        std::net::IpAddr::V6(a) => a,
        _ => Ipv6Addr::UNSPECIFIED,
    };
    // Destination address = source of the original packet
    let dst = match packet.src_ip {
        std::net::IpAddr::V6(a) => a,
        _ => Ipv6Addr::UNSPECIFIED,
    };
    buf.extend_from_slice(&src.octets());
    buf.extend_from_slice(&dst.octets());
    // ICMPv6 header
    buf.push(error_type);
    buf.push(code);
    // checksum placeholder
    buf.extend_from_slice(&[0, 0]);
    // For Time Exceeded (type 3) include 4‑byte unused field
    if error_type == 3 {
        buf.extend_from_slice(&[0, 0, 0, 0]);
    }
    // Append as much of the original packet as will fit within the IPv6 minimum MTU (1280)
    let max_payload = 1280 - buf.len();
    let copy_len = std::cmp::min(max_payload, packet.raw.len());
    buf.extend_from_slice(&packet.raw[..copy_len]);
    // Set payload length (everything after the IPv6 header)
    let payload_len = (buf.len() - 40) as u16;
    buf[payload_len_pos] = (payload_len >> 8) as u8;
    buf[payload_len_pos + 1] = (payload_len & 0xFF) as u8;
    // Compute ICMPv6 checksum over the ICMPv6 message (starting after the IPv6 header)
    let icmp_start = 40;
    let checksum = icmpv6_checksum(src, dst, &buf[icmp_start..]);
    buf[icmp_start + 2] = (checksum >> 8) as u8;
    buf[icmp_start + 3] = (checksum & 0xFF) as u8;
    buf
}

/// Generate a minimal ICMP error packet for IPv4 (stub).
// Generate a proper ICMP error packet for IPv4.
// `error_type` and `code` follow the ICMP specification.
pub fn generate_icmp_error(original: &PacketMeta, error_type: u8, code: u8) -> Vec<u8> {
    debug!("Generating ICMP error type {} code {}", error_type, code);
    const IPV4_HEADER_LEN: usize = 20;
    const ICMP_HEADER_LEN: usize = 8;
    const ORIGINAL_INCLUDE_LEN: usize = 28; // IP header (20) + 8 bytes of payload
    let mut packet = Vec::with_capacity(IPV4_HEADER_LEN + ICMP_HEADER_LEN + ORIGINAL_INCLUDE_LEN);
    // IPv4 header
    packet.push(0x45); // Version 4, IHL 5
    packet.push(0x00); // DSCP/ECN
                       // Total length placeholder
    packet.extend_from_slice(&[0, 0]);
    packet.extend_from_slice(&[0, 0]); // Identification
    packet.extend_from_slice(&[0, 0]); // Flags+Fragment Offset
    packet.push(64); // TTL
    packet.push(1); // Protocol = ICMP
    packet.extend_from_slice(&[0, 0]); // Header checksum placeholder
                                       // Source = original destination (router)
    let src_ip = match original.dst_ip {
        std::net::IpAddr::V4(a) => a.octets(),
        _ => [0, 0, 0, 0],
    };
    packet.extend_from_slice(&src_ip);
    // Destination = original source
    let dst_ip = match original.src_ip {
        std::net::IpAddr::V4(a) => a.octets(),
        _ => [0, 0, 0, 0],
    };
    packet.extend_from_slice(&dst_ip);
    // ICMP header
    packet.push(error_type);
    packet.push(code);
    packet.extend_from_slice(&[0, 0]); // Checksum placeholder
    packet.extend_from_slice(&[0, 0, 0, 0]); // Unused/MTU field
                                             // Include original IP header + first 8 bytes of payload
    let copy_len = std::cmp::min(ORIGINAL_INCLUDE_LEN, original.raw.len());
    packet.extend_from_slice(&original.raw[..copy_len]);
    // Set total length
    let total_len = packet.len() as u16;
    packet[2] = (total_len >> 8) as u8;
    packet[3] = (total_len & 0xFF) as u8;
    // Compute IPv4 header checksum
    crate::packet::update_ipv4_checksum(&mut packet[..IPV4_HEADER_LEN]);
    // Compute ICMP checksum
    let icmp_start = IPV4_HEADER_LEN;
    let icmp_checksum = calculate_icmp_checksum(&packet[icmp_start..]);
    packet[icmp_start + 2] = (icmp_checksum >> 8) as u8;
    packet[icmp_start + 3] = (icmp_checksum & 0xFF) as u8;
    packet
}

/// Calculate ICMP checksum (RFC 792) over the provided slice.
fn calculate_icmp_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        let word = u16::from_be_bytes([data[i], data[i + 1]]) as u32;
        sum += word;
        i += 2;
    }
    if i < data.len() {
        let word = (data[i] as u32) << 8;
        sum += word;
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}
