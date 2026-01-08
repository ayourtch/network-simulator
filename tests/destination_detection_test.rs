use network_simulator::{
    config::SimulatorConfig,
    packet::{self, PacketMeta},
    processor::process_packet,
    routing::{compute_routing, Destination},
    topology::{Fabric, RouterId},
};
use std::net::IpAddr;

#[tokio::test]
async fn test_destination_detection_stops_forwarding() {
    // Build a simple topology with two routers and a link between them.
    let mut cfg = SimulatorConfig::default();
    cfg.topology
        .routers
        .insert("Rx0y0".to_string(), toml::Value::Table(Default::default()));
    cfg.topology
        .routers
        .insert("Rx0y1".to_string(), toml::Value::Table(Default::default()));
    let link_cfg = network_simulator::topology::link::LinkConfig {
        mtu: Some(1500),
        delay_ms: 0,
        jitter_ms: 0,
        loss_percent: 0.0,
        load_balance: false,
    };
    cfg.topology
        .links
        .insert("Rx0y0_Rx0y1".to_string(), link_cfg);
    // Build fabric
    let mut fabric = Fabric::new();
    let r0 = network_simulator::topology::router::Router::new(RouterId("Rx0y0".to_string()));
    let r1 = network_simulator::topology::router::Router::new(RouterId("Rx0y1".to_string()));
    fabric.add_router(r0.clone());
    fabric.add_router(r1.clone());
    fabric.add_link(&r0.id, &r1.id, cfg.topology.links["Rx0y0_Rx0y1"].clone());
    // Compute routing tables (ingress A is Rx0y0, ingress B is Rx0y1)
    let tables = compute_routing(
        &fabric,
        RouterId("Rx0y0".to_string()),
        RouterId("Rx0y1".to_string()),
    );
    // Create a packet with TTL 64.
    let raw = vec![0u8; 20]; // minimal IPv4 header (enough for parsing)
    let packet = packet::parse(&raw).unwrap_or(PacketMeta {
        src_ip: "10.0.0.1".parse().unwrap(),
        dst_ip: "10.0.1.1".parse().unwrap(),
        src_port: 0,
        dst_port: 0,
        protocol: 6,
        ttl: 64,
        raw,
    });
    // Process packet from ingress Rx0y0 towards TunB (destination router is Rx0y1).
    let result = process_packet(
        &mut fabric,
        &tables,
        RouterId("Rx0y0".to_string()),
        packet,
        Destination::TunB,
    )
    .await;
    // The packet should have been forwarded once (TTL decremented) and then stopped at Rx0y1.
    assert_eq!(result.ttl, 63, "TTL should be decremented by one hop");
    // Ensure the packet's IPs are unchanged (no ICMP generation).
    assert_eq!(result.src_ip, "10.0.0.1".parse::<IpAddr>().unwrap());
    assert_eq!(result.dst_ip, "10.0.1.1".parse::<IpAddr>().unwrap());
}
