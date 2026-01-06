use network_simulator::config::SimulatorConfig;

#[test]
fn test_empty_topology_routers() {
    let cfg = SimulatorConfig::default();
    // No routers added to topology
    let result = cfg.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Topology must define at least one router"
    );
}

#[test]
fn test_duplicate_bidirectional_link() {
    let mut cfg = SimulatorConfig::default();
    // Add two valid routers matching default ingress IDs
    cfg.topology.routers.insert(
        "Rx0y0".to_string(),
        toml::Value::Table(toml::map::Map::new()),
    );
    cfg.topology.routers.insert(
        "Rx5y5".to_string(),
        toml::Value::Table(toml::map::Map::new()),
    );
    // Add link Rx0y0_Rx5y5 and duplicate Rx5y5_Rx0y0
    cfg.topology.links.insert(
        "Rx0y0_Rx5y5".to_string(),
        network_simulator::topology::link::LinkConfig {
            mtu: None,
            delay_ms: 1,
            jitter_ms: 0,
            loss_percent: 0.0,
            load_balance: false,
        },
    );
    cfg.topology.links.insert(
        "Rx5y5_Rx0y0".to_string(),
        network_simulator::topology::link::LinkConfig {
            mtu: None,
            delay_ms: 1,
            jitter_ms: 0,
            loss_percent: 0.0,
            load_balance: false,
        },
    );
    let result = cfg.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Duplicate bidirectional link detected"));
}
