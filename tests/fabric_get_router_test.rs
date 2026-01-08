use network_simulator::topology::{Fabric, Router, RouterId};

#[test]
fn test_fabric_get_router() {
    // Create fabric and add a router
    let mut fabric = Fabric::new();
    let router_id = RouterId("Rx0y0".to_string());
    let router = Router::new(router_id.clone());
    fabric.add_router(router);
    // Retrieve the router
    let retrieved = fabric.get_router(&router_id);
    assert!(retrieved.is_some());
    let r = retrieved.unwrap();
    assert_eq!(r.id, router_id);
}
