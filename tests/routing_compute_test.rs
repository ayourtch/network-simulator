// tests/routing_compute_test.rs

use network_simulator::config::{SimulatorConfig, TunIngressConfig, TopologyConfig};
use network_simulator::topology::router::RouterId;
use std::collections::HashMap;
use toml::Value;



#[test]
fn test_compute_routing_tables_counts() {
    // Minimal configuration with three routers and two links.
    let cfg = SimulatorConfig {
        simulation: Default::default(),
        interfaces: Default::default(),
        tun_ingress: TunIngressConfig {
            tun_a_ingress: "Rx0y0".to_string(),
            tun_b_ingress: "Rx0y1".to_string(),
            tun_a_prefix: "".to_string(),
            tun_b_prefix: "".to_string(),
            tun_a_ipv6_prefix: "".to_string(),
            tun_b_ipv6_prefix: "".to_string(),
        },
        topology: TopologyConfig {
            routers: {
                let mut map = HashMap::new();
                map.insert("Rx0y0".to_string(), Value::Table(toml::map::Map::new()));
                map.insert("Rx0y1".to_string(), Value::Table(toml::map::Map::new()));
                map.insert("Rx0y2".to_string(), Value::Table(toml::map::Map::new()));
                map
            },
            links: {
                let mut map = HashMap::new();
                map.insert(
                    "Rx0y0_Rx0y1".to_string(),
                    network_simulator::topology::link::LinkConfig { mtu: None, delay_ms: 1, jitter_ms: 0, loss_percent: 0.0, load_balance: true },
                );
                map.insert(
                    "Rx0y0_Rx0y2".to_string(),
                    network_simulator::topology::link::LinkConfig { mtu: None, delay_ms: 1, jitter_ms: 0, loss_percent: 0.0, load_balance: true },
                );
                map
            },
        },
        enable_multipath: false,
        packet_file: None,
        packet_files: None,
        packet_inject_tun: None,
        packet_inject_tuns: None,
        virtual_customer: None,
    };

    let tables = network_simulator::compute_routing_tables(&cfg);
    // Should have entries for each router.
    assert_eq!(tables.len(), 3);
    // Verify a known router exists.
    assert!(tables.contains_key(&RouterId("Rx0y0".to_string())));
}

#[test]
fn test_compute_multipath_tables_when_enabled() {
    // Same minimal configuration but enable multipath.
    let cfg = SimulatorConfig {
        simulation: Default::default(),
        interfaces: Default::default(),
        tun_ingress: TunIngressConfig {
            tun_a_ingress: "Rx0y0".to_string(),
            tun_b_ingress: "Rx0y1".to_string(),
            tun_a_prefix: "".to_string(),
            tun_b_prefix: "".to_string(),
            tun_a_ipv6_prefix: "".to_string(),
            tun_b_ipv6_prefix: "".to_string(),
        },
        topology: TopologyConfig {
            routers: {
                let mut map = HashMap::new();
                map.insert("Rx0y0".to_string(), Value::Table(toml::map::Map::new()));
                map.insert("Rx0y1".to_string(), Value::Table(toml::map::Map::new()));
                map.insert("Rx0y2".to_string(), Value::Table(toml::map::Map::new()));
                map
            },
            links: {
                let mut map = HashMap::new();
                map.insert(
                    "Rx0y0_Rx0y1".to_string(),
                    network_simulator::topology::link::LinkConfig { mtu: None, delay_ms: 1, jitter_ms: 0, loss_percent: 0.0, load_balance: true },
                );
                map.insert(
                    "Rx0y0_Rx0y2".to_string(),
                    network_simulator::topology::link::LinkConfig { mtu: None, delay_ms: 1, jitter_ms: 0, loss_percent: 0.0, load_balance: true },
                );
                map
            },
        },
        enable_multipath: true,
        packet_file: None,
        packet_files: None,
        packet_inject_tun: None,
        packet_inject_tuns: None,
        virtual_customer: None,
    };
    let tables = network_simulator::compute_multipath_tables(&cfg);
    // Should have entries for each router.
    assert_eq!(tables.len(), 3);
    // Verify a router has non‑empty tun_a entries.
    let entry = tables.get(&RouterId("Rx0y0".to_string())).expect("missing entry");
    assert!(!entry.tun_a.is_empty());
}

#[test]
fn test_compute_multipath_tables_when_disabled() {
    let cfg = SimulatorConfig {
        simulation: Default::default(),
        interfaces: Default::default(),
        tun_ingress: Default::default(),
        topology: TopologyConfig { routers: HashMap::new(), links: HashMap::new() },
        enable_multipath: false,
        packet_file: None,
        packet_files: None,
        packet_inject_tun: None,
        packet_inject_tuns: None,
        virtual_customer: None,
    };
    let tables = network_simulator::compute_multipath_tables(&cfg);
    assert!(tables.is_empty());
}
