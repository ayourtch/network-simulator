use network_simulator::routing::{compute_multi_path_routing, MultiPathTable};
use network_simulator::topology::{Fabric, LinkConfig, Router, RouterId};
use std::collections::HashMap;

#[test]
fn test_multipath_routing_two_equal_paths() {
    // Build a simple fabric: routers R0, R1, R2; ingress at R0; destination at R2.
    let mut fabric = Fabric::new();
    // Add routers with valid IDs (RxXyY pattern)
    let r0 = Router {
        id: RouterId("Rx0y0".to_string()),
        routing: Default::default(),
        stats: Default::default(),
    };
    let r1 = Router {
        id: RouterId("Rx0y1".to_string()),
        routing: Default::default(),
        stats: Default::default(),
    };
    let r2 = Router {
        id: RouterId("Rx0y2".to_string()),
        routing: Default::default(),
        stats: Default::default(),
    };
    fabric.add_router(r0);
    fabric.add_router(r1);
    fabric.add_router(r2);
    // Links: R0-R1 and R0-R2 with same delay, and R1-R2.
    let cfg_direct = LinkConfig {
        mtu: None,
        delay_ms: 10,
        jitter_ms: 0,
        loss_percent: 0.0,
        load_balance: false,
    };
    let cfg_via = LinkConfig {
        mtu: None,
        delay_ms: 5,
        jitter_ms: 0,
        loss_percent: 0.0,
        load_balance: false,
    };
    fabric.add_link(
        &RouterId("Rx0y0".to_string()),
        &RouterId("Rx0y1".to_string()),
        cfg_via.clone(),
    );
    fabric.add_link(
        &RouterId("Rx0y0".to_string()),
        &RouterId("Rx0y2".to_string()),
        cfg_direct.clone(),
    );
    fabric.add_link(
        &RouterId("Rx0y1".to_string()),
        &RouterId("Rx0y2".to_string()),
        cfg_via.clone(),
    );

    let ingress_a = RouterId("Rx0y0".to_string());
    let ingress_b = RouterId("Rx0y2".to_string()); // treat R2 as other ingress for completeness
    let tables: HashMap<RouterId, MultiPathTable> =
        compute_multi_path_routing(&fabric, ingress_a.clone(), ingress_b.clone());
    // For router R0, there should be two equal-cost next hops to reach R2 (direct and via R1)
    let r0_table = tables.get(&ingress_a).expect("R0 table missing");
    // tun_a entries (from ingress A to destination B) should contain both R1 and R2 as next hops
    let mut next_hops: Vec<String> = r0_table
        .tun_a
        .iter()
        .map(|e| e.next_hop.0.clone())
        .collect();
    next_hops.sort();
    assert_eq!(next_hops, vec!["Rx0y1".to_string(), "Rx0y2".to_string()]);
}
