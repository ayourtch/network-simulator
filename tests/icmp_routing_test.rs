// tests/icmp_routing_test.rs

use network_simulator::config::{SimulatorConfig, TopologyConfig, TunIngressConfig};
use network_simulator::packet::PacketMeta;
use network_simulator::processor::process_packet;
use network_simulator::routing::{Destination, RoutingTable};
use network_simulator::topology::{router::Router, router::RouterId, Fabric};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use tokio::runtime::Runtime;

#[test]
fn test_icmp_destination_unreachable_generated() {
    // Simple config with no routing entries.
    let _cfg = SimulatorConfig {
        simulation: Default::default(),
        interfaces: Default::default(),
        tun_ingress: TunIngressConfig::default(),
        topology: TopologyConfig::default(),
        enable_multipath: false,
        packet_file: None,
        packet_files: None,
        packet_inject_tun: None,
        packet_inject_tuns: None,
        virtual_customer: None,
    };

    // Build minimal fabric with one router having a valid ID.
    let mut fabric = Fabric::new();
    let router = Router {
        id: RouterId("Rx0y0".to_string()),
        routing: Default::default(),
        stats: Default::default(),
    };
    fabric.add_router(router);

    // Empty routing table (no entry for destination).
    let tables: HashMap<RouterId, RoutingTable> = HashMap::new();

    // Create a simple IPv4 packet.
    let packet = PacketMeta {
        src_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        dst_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        src_port: 0,
        dst_port: 0,
        protocol: 6, // TCP
        ttl: 64,
        raw: vec![0u8; 20],
    };

    let rt = Runtime::new().unwrap();
    let processed = rt.block_on(async {
        process_packet(
            &mut fabric,
            &tables,
            RouterId("Rx0y0".to_string()),
            packet,
            Destination::TunA,
        )
        .await
    });

    // The processed packet should be an ICMP (protocol 1) indicating Destination Unreachable.
    assert_eq!(
        processed.protocol, 1,
        "Expected ICMP protocol after routing failure"
    );
    // Router stats should show an ICMP generated.
    let router_stats = fabric
        .get_router(&RouterId("Rx0y0".to_string()))
        .unwrap()
        .stats
        .clone();
    assert!(
        router_stats.icmp_generated > 0,
        "ICMP counter should be incremented"
    );
}
