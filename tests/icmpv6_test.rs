use network_simulator::icmp::generate_icmpv6_error;
use network_simulator::packet::parse;

#[test]
fn test_generate_icmpv6_time_exceeded() {
    // Minimal IPv6 packet (40 bytes header, no payload)
    let raw = vec![
        0x60, 0, 0, 0, // Version, Traffic Class, Flow Label
        0, 0,  // Payload length
        6,  // Next Header (TCP)
        64, // Hop Limit
        // Source address 2001:db8::1
        0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        // Destination address 2001:db8::2
        0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2,
    ];
    let packet = parse(&raw).expect("parse IPv6 packet");
    let icmp = generate_icmpv6_error(&packet, 3, 0); // Time Exceeded
                                                     // Verify IPv6 version
    assert_eq!(icmp[0] >> 4, 6);
    // Next Header should be ICMPv6 (58)
    assert_eq!(icmp[6], 58);
    // ICMPv6 Type and Code at offset 40 (after IPv6 header)
    assert_eq!(icmp[40], 3);
    assert_eq!(icmp[41], 0);
    // Check that checksum is non‑zero (bytes 42‑43)
    let checksum = ((icmp[42] as u16) << 8) | (icmp[43] as u16);
    assert_ne!(checksum, 0);
}
