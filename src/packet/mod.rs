// src/packet/mod.rs

use std::net::{IpAddr, Ipv4Addr};

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
pub fn parse(data: &[u8]) -> Result<PacketMeta, &'static str> {
    // Minimal IPv4 header parsing (no options).
    if data.len() < 20 {
        return Err("packet too short for IPv4 header");
    }
    let version_ihl = data[0];
    let version = version_ihl >> 4;
    if version != 4 {
        return Err("only IPv4 parsing supported in stub");
    }
    let ihl = version_ihl & 0x0F;
    if ihl < 5 {
        return Err("invalid IHL");
    }
    let total_len = u16::from_be_bytes([data[2], data[3]]) as usize;
    if data.len() < total_len {
        return Err("packet length less than total_len");
    }
    let ttl = data[8];
    let protocol = data[9];
    let src_ip = Ipv4Addr::new(data[12], data[13], data[14], data[15]);
    let dst_ip = Ipv4Addr::new(data[16], data[17], data[18], data[19]);
    // Ports are not present in raw IP header; set to 0 for stub.
    Ok(PacketMeta {
        src_ip: IpAddr::V4(src_ip),
        dst_ip: IpAddr::V4(dst_ip),
        src_port: 0,
        dst_port: 0,
        protocol,
        ttl,
        customer_id: 0,
    })
}

