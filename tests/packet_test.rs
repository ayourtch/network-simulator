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

#[test]
fn test_ipv6_parse_minimal() {
    // Minimal IPv6 header (40 bytes) with Hop Limit=64, Next Header=17 (UDP)
    let data = vec![
        0x60, 0x00, 0x00, 0x00, // Version=6, Traffic Class & Flow Label
        0x00, 0x08, 0x11, 0x40, // Payload Length=8, Next Header=17 (UDP), Hop Limit=64
        // Source IPv6 address: 2001:db8::1
        0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        // Destination IPv6 address: 2001:db8::2
        0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
    ];
    let meta = parse(&data).expect("IPv6 parse should succeed");
    assert_eq!(meta.ttl, 64); // hop limit
    assert_eq!(meta.protocol, 17); // next header (UDP)
    assert_eq!(meta.src_ip.to_string(), "2001:db8::1");
    assert_eq!(meta.dst_ip.to_string(), "2001:db8::2");
}
