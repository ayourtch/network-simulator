use network_simulator::topology::{Fabric, RouterId};

#[test]
fn test_fabric_incident_links_missing_router() {
    let fabric = Fabric::new();
    let router_id = RouterId("Rx0y0".to_string());
    // No router added; incident_links should return empty vector
    let links = fabric.incident_links(&router_id);
    assert!(links.is_empty());
}
