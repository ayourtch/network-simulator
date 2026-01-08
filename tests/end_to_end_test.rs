use network_simulator::{
    config::SimulatorConfig,
    packet::parse,
    processor::process_packet,
    routing::{compute_routing, Destination},
    topology::{Fabric, RouterId},
};

#[tokio::test]
async fn test_end_to_end_packet_processing() {
    // Setup configuration with minimal topology
    let mut cfg = SimulatorConfig::default();
    cfg.topology.routers.insert(
        "Rx0y0".to_string(),
        toml::Value::Table(toml::map::Map::new()),
    );
    cfg.topology.routers.insert(
        "Rx0y1".to_string(),
        toml::Value::Table(toml::map::Map::new()),
    );
    cfg.topology.links.insert(
        "Rx0y0_Rx0y1".to_string(),
        network_simulator::topology::link::LinkConfig {
            mtu: None,
            delay_ms: 0,
            jitter_ms: 0,
            loss_percent: 0.0,
            load_balance: false,
        },
    );
    // Build fabric
    let mut fabric = Fabric::new();
    let r0 = network_simulator::topology::router::Router::new(RouterId("Rx0y0".to_string()));
    let r1 = network_simulator::topology::router::Router::new(RouterId("Rx0y1".to_string()));
    fabric.add_router(r0.clone());
    fabric.add_router(r1.clone());
    fabric.add_link(&r0.id, &r1.id, cfg.topology.links["Rx0y0_Rx0y1"].clone());
    // Compute routing tables (ingress routers for tun A/B)
    let tables = compute_routing(
        &fabric,
        RouterId("Rx0y0".to_string()),
        RouterId("Rx0y1".to_string()),
    );
    // Create a simple IPv4 packet (dummy raw bytes with src/dst)
    let raw = vec![0u8; 60]; // enough for IPv4 header
    let raw_clone = raw.clone();
    let packet = parse(&raw).unwrap_or(network_simulator::packet::PacketMeta {
        src_ip: "10.0.0.1".parse().unwrap(),
        dst_ip: "10.0.0.2".parse().unwrap(),
        src_port: 1234,
        dst_port: 80,
        protocol: 6,
        ttl: 64,
        raw: raw_clone,
    });
    // Process packet from tun A (ingress Rx0y0) towards TunB
    let processed = process_packet(
        &mut fabric,
        &tables,
        RouterId("Rx0y0".to_string()),
        packet.clone(),
        Destination::TunB,
    )
    .await;
    // Since the packet reaches its destination router (Rx0y1) after one hop, TTL should be decremented by one.
    // Verify that the TTL byte (offset 8) is reduced and other fields remain unchanged except checksum.
    assert_eq!(processed.raw[8], raw[8].saturating_sub(1));
    // Protocol byte should remain the same.
    assert_eq!(processed.raw[9], raw[9]);
    // Verify statistics: each router should have received and forwarded counts
    let stats = fabric.get_statistics();
    let r0_stats = stats.get(&RouterId("Rx0y0".to_string())).unwrap();
    let r1_stats = stats.get(&RouterId("Rx0y1".to_string())).unwrap();
    assert_eq!(r0_stats.packets_received, 1);
    assert_eq!(r0_stats.packets_forwarded, 1);
    assert_eq!(r1_stats.packets_received, 1);
    // Destination router should not forward further
    assert_eq!(r1_stats.packets_forwarded, 0);
}
