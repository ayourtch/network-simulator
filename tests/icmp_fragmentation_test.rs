// tests/icmp_fragmentation_test.rs

use network_simulator::icmp;
use network_simulator::packet::PacketMeta;
use std::net::Ipv4Addr;

#[test]
fn test_generate_fragmentation_needed() {
    // Minimal IPv4 packet (20-byte header) with dummy addresses.
    let src_ip = Ipv4Addr::new(192, 168, 0, 1);
    let dst_ip = Ipv4Addr::new(192, 168, 0, 2);
    let mut raw = vec![0u8; 20];
    raw[0] = 0x45; // Version/IHL
    raw[9] = 6; // TCP protocol placeholder
    raw[12..16].copy_from_slice(&src_ip.octets());
    raw[16..20].copy_from_slice(&dst_ip.octets());
    let packet = PacketMeta {
        src_ip: std::net::IpAddr::V4(src_ip),
        dst_ip: std::net::IpAddr::V4(dst_ip),
        src_port: 0,
        dst_port: 0,
        protocol: 6,
        ttl: 64,
        raw,
    };
    let mtu = 1500u32;
    let icmp_pkt = icmp::generate_fragmentation_needed(&packet, mtu);
    // Verify IPv4 header length (should be >= 28 bytes)
    assert!(icmp_pkt.len() >= 28);
    // Type should be 3, code 4
    assert_eq!(icmp_pkt[20], 3);
    assert_eq!(icmp_pkt[21], 4);
    // MTU field is at offset 24 (after type,code,checksum,unused)
    let mtu_bytes = &icmp_pkt[24..26];
    assert_eq!(u16::from_be_bytes([mtu_bytes[0], mtu_bytes[1]]), mtu as u16);
}
