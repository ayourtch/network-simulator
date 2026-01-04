use network_simulator::config::SimulatorConfig;

#[test]
fn test_invalid_link_name_format() {
    let mut cfg = SimulatorConfig::default();
    // Add a router to avoid empty topology error
    cfg.topology.routers.insert("R1".to_string(), toml::Value::Table(toml::map::Map::new()));
    cfg.topology.routers.insert("R2".to_string(), toml::Value::Table(toml::map::Map::new()));
    // Add a link with invalid name (no underscore)
    cfg.topology.links.insert(
        "R1R2".to_string(),
        network_simulator::topology::link::LinkConfig { mtu: None, delay_ms: 1, jitter_ms: 0, loss_percent: 0.0, load_balance: false },
    );
    let result = cfg.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid link name 'R1R2'"));
}
