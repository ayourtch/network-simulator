// src/packet/mod.rs

use crate::topology::RouterId;
use std::net::IpAddr;

/// Minimal packet metadata used by the simulator.
#[derive(Debug, Clone)]
pub struct PacketMeta {
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8, // TCP=6, UDP=17, ICMP=1, ICMPv6=58
    pub ttl: u8,
    pub customer_id: u32,
}

/// Stub parser – in the full version this would decode raw bytes using the `pnet` crate.
pub fn parse(_data: &[u8]) -> Result<PacketMeta, &'static str> {
    Err("packet parsing not implemented yet")
}
