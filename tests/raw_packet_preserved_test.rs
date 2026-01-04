use network_simulator::packet::{parse, PacketMeta};

#[test]
fn test_raw_packet_ttl_decrement_preserves_raw() {
    // Minimal IPv4 packet (20 bytes) with TTL=64 at offset 8.
    // Header fields: version/IHL=0x45, DSCP=0, total length=20, identification=0, flags/frag=0,
    // TTL=64, protocol=6 (TCP), header checksum=0, src=10.0.0.1, dst=10.0.0.2
    let raw: Vec<u8> = vec![
        0x45, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x00,
        0x40, 0x06, 0x00, 0x00,
        10, 0, 0, 1,
        10, 0, 0, 2,
    ];
    let packet = parse(&raw).expect("parse failed");
    assert_eq!(packet.ttl, 64);
    assert_eq!(packet.raw[8], 64);
    // Decrement TTL using method
    let mut pkt = packet.clone();
    pkt.decrement_ttl().expect("decrement failed");
    assert_eq!(pkt.ttl, 63);
    assert_eq!(pkt.raw[8], 63);
    // Ensure other bytes unchanged
    for i in 0..8 {
        assert_eq!(pkt.raw[i], raw[i]);
    }
    for i in 9..raw.len() {
        assert_eq!(pkt.raw[i], raw[i]);
    }
}
