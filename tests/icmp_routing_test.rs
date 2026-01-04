use network_simulator::{config::SimulatorConfig, topology::{Fabric, RouterId}, routing::{Destination, compute_routing}, processor::process_packet, packet::{self, PacketMeta}};
use std::net::IpAddr;

#[tokio::test]
async fn test_icmp_routing_back_to_source() {
    // Build a simple config with two routers and a link with small MTU to force drop.
    let mut cfg = SimulatorConfig::default();
    cfg.topology.routers.insert("Rx0y0".to_string(), toml::Value::Table(Default::default()));
    cfg.topology.routers.insert("Rx0y1".to_string(), toml::Value::Table(Default::default()));
    let link_cfg = network_simulator::topology::link::LinkConfig { mtu: Some(100), delay_ms: 0, jitter_ms: 0, loss_percent: 0.0, load_balance: false };
    cfg.topology.links.insert("Rx0y0_Rx0y1".to_string(), link_cfg);
    // Build fabric
    let mut fabric = Fabric::new();
    let r0 = network_simulator::topology::router::Router { id: RouterId("Rx0y0".to_string()), routing: Default::default(), stats: Default::default() };
    let r1 = network_simulator::topology::router::Router { id: RouterId("Rx0y1".to_string()), routing: Default::default(), stats: Default::default() };
    fabric.add_router(r0.clone());
    fabric.add_router(r1.clone());
    fabric.add_link(&r0.id, &r1.id, cfg.topology.links["Rx0y0_Rx0y1"].clone());
    // Compute routing tables (both routers are ingress for respective TUNs)
    let tables = compute_routing(&fabric, RouterId("Rx0y0".to_string()), RouterId("Rx0y1".to_string()));
    // Create a large IPv6 packet (200 bytes) to exceed MTU.
    let raw = vec![0u8; 200];
    let packet = packet::parse(&raw).unwrap_or(PacketMeta {
        src_ip: "2001:db8::1".parse().unwrap(),
        dst_ip: "2001:db8::2".parse().unwrap(),
        src_port: 0,
        dst_port: 0,
        protocol: 6,
        ttl: 64,
        raw,
    });
    // Process packet from ingress Rx0y0 towards TunB.
    let result = process_packet(&mut fabric, &tables, RouterId("Rx0y0".to_string()), packet, Destination::TunB).await;
    // The result should be an ICMP packet with src/dst swapped.
    assert_eq!(result.src_ip, "2001:db8::2".parse::<IpAddr>().unwrap());
    assert_eq!(result.dst_ip, "2001:db8::1".parse::<IpAddr>().unwrap());
}
