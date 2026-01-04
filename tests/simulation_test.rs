use network_simulator::topology::{Fabric, Router, RouterId, LinkConfig};
use network_simulator::packet::PacketMeta;
use network_simulator::forwarding::select_egress_link;
use network_simulator::simulation::simulate_link;
use std::net::{IpAddr, Ipv4Addr};

#[tokio::test]
async fn test_link_simulation_and_load_balancing() {
    // Setup fabric with two routers and a link with load_balance enabled
    let mut fabric = Fabric::new();
    let dummy_route = network_simulator::routing::RoutingTable {
        tun_a: network_simulator::routing::RouteEntry { next_hop: RouterId("".to_string()), total_cost: 0 },
        tun_b: network_simulator::routing::RouteEntry { next_hop: RouterId("".to_string()), total_cost: 0 },
    };
    let r1 = Router { id: RouterId("Rx0y0".to_string()), routing: dummy_route.clone(), stats: Default::default() };
    let r2 = Router { id: RouterId("Rx0y1".to_string()), routing: dummy_route, stats: Default::default() };
    fabric.add_router(r1.clone());
    fabric.add_router(r2.clone());
    let link_cfg = LinkConfig { mtu: Some(1500), delay_ms: 0, jitter_ms: 0, loss_percent: 0.0, load_balance: true };
    fabric.add_link(&r1.id, &r2.id, link_cfg.clone());

    // Create a dummy routing table map (stub) with direct next hop
    use std::collections::HashMap;
    let mut tables = HashMap::new();
    tables.insert(r1.id.clone(), network_simulator::routing::RoutingTable { tun_a: network_simulator::routing::RouteEntry { next_hop: r2.id.clone(), total_cost: 0 }, tun_b: network_simulator::routing::RouteEntry { next_hop: r2.id.clone(), total_cost: 0 } });
    tables.insert(r2.id.clone(), network_simulator::routing::RoutingTable { tun_a: network_simulator::routing::RouteEntry { next_hop: r1.id.clone(), total_cost: 0 }, tun_b: network_simulator::routing::RouteEntry { next_hop: r1.id.clone(), total_cost: 0 } });

    // Packet metadata for hashing
    let packet = PacketMeta {
        src_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        dst_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        src_port: 1234,
        dst_port: 80,
        protocol: 6,
        ttl: 64,
        //customer_id: 0,
    };

    // Select egress link from r1
    let links = fabric.incident_links(&r1.id);
    let selected = select_egress_link(&r1.id, &packet, &links, &tables).expect("link selected");
    assert!(selected.cfg.load_balance, "selected link should have load_balance enabled");
    let before = selected.counter();
    // Simulate link processing
    simulate_link(selected, &[]).await.expect("simulation should succeed");
    let after = selected.counter();
    assert_eq!(after, before + 1, "counter should increment");
}
