// src/icmp/mod.rs

use crate::packet::PacketMeta;
use tracing::debug;

/// Generate an ICMP error packet based on the original packet metadata.
/// Stub implementation – returns a dummy byte vector.
pub fn generate_icmp_error(_original: &PacketMeta, _error_type: u8, _code: u8) -> Vec<u8> {
    debug!("Generating ICMP error type {} code {}", _error_type, _code);
    vec![] // placeholder
}
