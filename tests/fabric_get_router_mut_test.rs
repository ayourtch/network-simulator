use network_simulator::topology::{Fabric, Router, RouterId};

#[test]
fn test_fabric_get_router_mut() {
    // Setup fabric with a router
    let mut fabric = Fabric::new();
    let router_id = RouterId("Rx0y0".to_string());
    let router = Router {
        id: router_id.clone(),
        routing: Default::default(),
        stats: Default::default(),
    };
    fabric.add_router(router);
    // Retrieve mutable reference and modify stats
    {
        let r_mut = fabric.get_router_mut(&router_id).expect("router exists");
        r_mut.increment_received();
        r_mut.increment_forwarded();
        r_mut.increment_icmp();
    }
    // Verify modifications via immutable getter
    let r = fabric.get_router(&router_id).expect("router exists");
    assert_eq!(r.stats.packets_received, 1);
    assert_eq!(r.stats.packets_forwarded, 1);
    assert_eq!(r.stats.icmp_generated, 1);
}
