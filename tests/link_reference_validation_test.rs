use network_simulator::config::SimulatorConfig;
use std::collections::HashMap;
use network_simulator::topology::link::LinkConfig;

#[test]
fn test_invalid_link_reference() {
    // Config with routers A and B, but link references unknown router C
    let mut topology = network_simulator::config::TopologyConfig {
        routers: HashMap::new(),
        links: HashMap::new(),
    };
    topology.routers.insert("Rx0y0".to_string(), toml::Value::String("".to_string()));
    topology.routers.insert("Rx0y1".to_string(), toml::Value::String("".to_string()));
    // link to unknown Rx0y2
    topology.links.insert("Rx0y0_Rx0y2".to_string(), LinkConfig { mtu: None, delay_ms: 0, jitter_ms: 0, loss_percent: 0.0, load_balance: false });
    let cfg = SimulatorConfig {
        simulation: Default::default(),
        interfaces: Default::default(),
        tun_ingress: network_simulator::config::TunIngressConfig { tun_a_ingress: "Rx0y0".to_string(), tun_b_ingress: "Rx0y1".to_string() },
        topology,
        enable_multipath: false,
        packet_file: None,
    };
    assert!(cfg.validate().is_err());
}
