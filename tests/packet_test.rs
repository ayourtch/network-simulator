use network_simulator::packet::parse;

#[test]
fn test_ipv4_parse_minimal() {
    // Minimal IPv4 header (20 bytes) with TTL=64, protocol=6 (TCP)
    let data = vec![
        0x45, 0x00, 0x00, 0x14, // version/IHL, TOS, total length 20
        0x00, 0x00, 0x00, 0x00, // ID, flags/frag
        0x40, 0x06, 0x00, 0x00, // TTL=64, protocol=6, checksum
        192, 168, 1, 1, // src IP 192.168.1.1
        192, 168, 1, 2, // dst IP 192.168.1.2
    ];
    let meta = parse(&data).expect("parse should succeed");
    assert_eq!(meta.ttl, 64);
    assert_eq!(meta.protocol, 6);
    assert_eq!(meta.src_ip.to_string(), "192.168.1.1");
    assert_eq!(meta.dst_ip.to_string(), "192.168.1.2");
}
