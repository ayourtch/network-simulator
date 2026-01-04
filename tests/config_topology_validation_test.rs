use network_simulator::config::SimulatorConfig;

#[test]
fn test_empty_topology_routers() {
    let mut cfg = SimulatorConfig::default();
    // No routers added to topology
    let result = cfg.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Topology must define at least one router");
}

#[test]
fn test_duplicate_bidirectional_link() {
    let mut cfg = SimulatorConfig::default();
    // Add two routers
    cfg.topology.routers.insert("R1".to_string(), toml::Value::Table(toml::map::Map::new()));
    cfg.topology.routers.insert("R2".to_string(), toml::Value::Table(toml::map::Map::new()));
    // Add link R1_R2 and duplicate R2_R1
    cfg.topology.links.insert(
        "R1_R2".to_string(),
        network_simulator::topology::link::LinkConfig { mtu: None, delay_ms: 1, jitter_ms: 0, loss_percent: 0.0, load_balance: false },
    );
    cfg.topology.links.insert(
        "R2_R1".to_string(),
        network_simulator::topology::link::LinkConfig { mtu: None, delay_ms: 1, jitter_ms: 0, loss_percent: 0.0, load_balance: false },
    );
    let result = cfg.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Duplicate bidirectional link detected"));
}
