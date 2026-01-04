// src/icmp/mod.rs

use crate::packet::PacketMeta;
use tracing::debug;

/// Generate a minimal ICMP error packet based on the original packet metadata.
/// This implementation creates a simple placeholder payload (8 bytes) for the
/// specified error type and code. In a full implementation the packet would
/// contain proper IP/ICMP headers and the original packet data.
pub fn generate_icmp_error(_original: &PacketMeta, error_type: u8, code: u8) -> Vec<u8> {
    debug!("Generating ICMP error type {} code {}", error_type, code);
    // Simple placeholder: type, code, and six zero bytes.
    let mut payload = vec![0u8; 8];
    payload[0] = error_type;
    payload[1] = code;
    payload
}

