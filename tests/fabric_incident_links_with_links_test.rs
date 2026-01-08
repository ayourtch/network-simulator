use network_simulator::topology::{Fabric, LinkConfig, Router, RouterId};

#[test]
fn test_fabric_incident_links_with_links() {
    // Setup fabric with two routers and a link between them
    let mut fabric = Fabric::new();
    let router_a_id = RouterId("Rx0y0".to_string());
    let router_b_id = RouterId("Rx5y5".to_string());
    let router_a = Router::new(router_a_id.clone());
    let router_b = Router::new(router_b_id.clone());
    fabric.add_router(router_a);
    fabric.add_router(router_b);
    // Add a link between the routers
    let link_cfg = LinkConfig {
        mtu: None,
        delay_ms: 1,
        jitter_ms: 0,
        loss_percent: 0.0,
        load_balance: false,
    };
    fabric.add_link(&router_a_id, &router_b_id, link_cfg);
    // Verify incident_links returns the link for each router
    let links_a = fabric.incident_links(&router_a_id);
    let links_b = fabric.incident_links(&router_b_id);
    assert_eq!(links_a.len(), 1);
    assert_eq!(links_b.len(), 1);
    // Ensure the link IDs are consistent
    let link_a = links_a[0];
    let link_b = links_b[0];
    assert_eq!(link_a.id, link_b.id);
}
