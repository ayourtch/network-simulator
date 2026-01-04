use network_simulator::topology::router::RouterId;

#[test]
fn test_valid_router_ids() {
    assert!(RouterId("Rx0y0".to_string()).validate().is_ok());
    assert!(RouterId("Rx5y5".to_string()).validate().is_ok());
    assert!(RouterId("Rx3y2".to_string()).validate().is_ok());
}

#[test]
fn test_invalid_router_ids() {
    assert!(RouterId("Rx6y0".to_string()).validate().is_err());
    assert!(RouterId("Rx0y6".to_string()).validate().is_err());
    assert!(RouterId("Rx10y5".to_string()).validate().is_err());
    assert!(RouterId("Rxa0yb0".to_string()).validate().is_err());
}
