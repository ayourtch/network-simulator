// tests/fabric_get_link_test.rs

use network_simulator::topology::{Fabric, Router, RouterId, LinkConfig};

#[test]
fn test_fabric_get_link() {
    let mut fabric = Fabric::new();
    let a_id = RouterId("Rx0y0".to_string());
    let b_id = RouterId("Rx1y1".to_string());
    let router_a = Router { id: a_id.clone(), routing: Default::default(), stats: Default::default() };
    let router_b = Router { id: b_id.clone(), routing: Default::default(), stats: Default::default() };
    fabric.add_router(router_a);
    fabric.add_router(router_b);
    let cfg = LinkConfig { mtu: None, delay_ms: 5, jitter_ms: 0, loss_percent: 0.0, load_balance: false };
    fabric.add_link(&a_id, &b_id, cfg);
    let link_opt = fabric.get_link(&a_id, &b_id);
    assert!(link_opt.is_some());
    let link = link_opt.unwrap();
    // Ensure link id components match the routers (order may be normalized)
    assert!( (link.id.a == a_id && link.id.b == b_id) || (link.id.a == b_id && link.id.b == a_id) );
}
