// tests/tun_mock_test.rs

use network_simulator::config::{SimulatorConfig, TunIngressConfig, TopologyConfig};
use network_simulator::topology::{Fabric, router::Router, router::RouterId};
use network_simulator::topology::link::LinkConfig;
use network_simulator::tun;
use std::collections::HashMap;
use tempfile::NamedTempFile;
use std::io::Write;
use tokio::runtime::Runtime;

#[test]
fn test_tun_mock_packet_processing() {
    // Create a temporary file with a minimal IPv4 packet hex representation.
    let mut tmp = NamedTempFile::new().expect("temp file");
    // Same packet as in packet_test (20 bytes).
    let packet_hex = "450000140000000040060000c0a80101c0a80102"; // version/IHL/TOS/len etc.
    writeln!(tmp, "{}", packet_hex).expect("write");
    let path = tmp.path().to_str().unwrap().to_string();

    // Build a simple config.
    let cfg = SimulatorConfig {
        simulation: Default::default(),
        interfaces: Default::default(),
        tun_ingress: TunIngressConfig { tun_a_ingress: "Rx0y0".to_string(), tun_b_ingress: "Rx0y1".to_string(), tun_a_prefix: "".to_string(), tun_b_prefix: "".to_string() },
        topology: TopologyConfig {
            routers: {
                let mut map = HashMap::new();
                map.insert("Rx0y0".to_string(), toml::Value::Table(toml::map::Map::new()));
                map.insert("Rx0y1".to_string(), toml::Value::Table(toml::map::Map::new()));
                map
            },
            links: {
                let mut map = HashMap::new();
                map.insert(
                    "Rx0y0_Rx0y1".to_string(),
                    LinkConfig { mtu: None, delay_ms: 1, jitter_ms: 0, loss_percent: 0.0, load_balance: false },
                );
                map
            },
        },
        enable_multipath: false,
        packet_file: Some(path.clone()),
        packet_files: None,
        packet_inject_tun: None,
        packet_inject_tuns: None,
    };

    // Build fabric as in lib::run.
    let mut fabric = Fabric::new();
    // Add routers.
    let router_a = Router { id: RouterId("Rx0y0".to_string()), routing: Default::default(), stats: Default::default() };
    let router_b = Router { id: RouterId("Rx0y1".to_string()), routing: Default::default(), stats: Default::default() };
    fabric.add_router(router_a);
    fabric.add_router(router_b);
    // Add link.
    fabric.add_link(&RouterId("Rx0y0".to_string()), &RouterId("Rx0y1".to_string()), LinkConfig { mtu: None, delay_ms: 1, jitter_ms: 0, loss_percent: 0.0, load_balance: false });

    // Run TUN mock processing.
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        tun::start(&cfg, &mut fabric).await.expect("tun start");
    });

    // Verify that the link counter was incremented (packet passed through).
    let edge_idx = fabric.link_index.values().next().expect("link exists");
    let edge = fabric.graph.edge_weight(*edge_idx).expect("edge weight");
    assert!(edge.counter() > 0, "link counter should be incremented");
}
