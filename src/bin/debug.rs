fn main() {
    let raw = vec![0u8; 200];
    let packet = network_simulator::packet::parse(&raw).unwrap_or(network_simulator::packet::PacketMeta {
        src_ip: "2001:db8::1".parse().unwrap(),
        dst_ip: "2001:db8::2".parse().unwrap(),
        src_port: 0,
        dst_port: 0,
        protocol: 6,
        ttl: 64,
        raw,
    });
    let icmp_bytes = network_simulator::icmp::generate_icmpv6_error(&packet, 2, 0);
    println!("icmp len {}", icmp_bytes.len());
    let icmp_packet = network_simulator::packet::parse(&icmp_bytes).unwrap();
    println!("src: {} dst: {}", icmp_packet.src_ip, icmp_packet.dst_ip);
}
