// src/icmp/mod.rs

use crate::packet::PacketMeta;
use tracing::debug;

/// Generate a minimal ICMP error packet for IPv4 (stub).
pub fn generate_icmp_error(_original: &PacketMeta, error_type: u8, code: u8) -> Vec<u8> {
    debug!("Generating ICMP error type {} code {}", error_type, code);
    let mut payload = vec![0u8; 8];
    payload[0] = error_type;
    payload[1] = code;
    payload
}

/// Generate a minimal ICMPv6 error packet based on the original packet metadata.
/// This is a stub implementation similar to IPv4 version.
pub fn generate_icmpv6_error(_original: &PacketMeta, error_type: u8, code: u8) -> Vec<u8> {
    debug!("Generating ICMPv6 error type {} code {}", error_type, code);
    let mut payload = vec![0u8; 8];
    payload[0] = error_type;
    payload[1] = code;
    payload
}
