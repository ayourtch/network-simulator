// tests/tun_multiple_mock_test.rs

use network_simulator::config::{SimulatorConfig, TunIngressConfig, TopologyConfig};
use network_simulator::topology::{Fabric, router::{Router, RouterId}};
use network_simulator::topology::link::LinkConfig;
use network_simulator::tun;
use std::collections::HashMap;
use tempfile::NamedTempFile;
use std::io::Write;
use tokio::runtime::Runtime;

#[test]
fn test_multiple_mock_packet_files_injection() {
    // Create two temporary packet files.
    let mut tmp1 = NamedTempFile::new().expect("temp file 1");
    let mut tmp2 = NamedTempFile::new().expect("temp file 2");
    // Simple IPv4 packet hex (20 bytes) – same as used elsewhere.
    let packet_hex = "450000140000000040060000c0a80101c0a80102";
    writeln!(tmp1, "{}", packet_hex).expect("write packet 1");
    writeln!(tmp2, "{}", packet_hex).expect("write packet 2");
    let path1 = tmp1.path().to_str().unwrap().to_string();
    let path2 = tmp2.path().to_str().unwrap().to_string();

    // Build config with both files and explicit injection directions.
    let cfg = SimulatorConfig {
        simulation: Default::default(),
        interfaces: Default::default(),
        tun_ingress: TunIngressConfig { tun_a_ingress: "Rx0y0".to_string(), tun_b_ingress: "Rx0y1".to_string() },
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
        packet_file: None,
        packet_files: Some(vec![path1.clone(), path2.clone()]),
        packet_inject_tun: None,
        packet_inject_tuns: Some(vec!["tun_a".to_string(), "tun_b".to_string()]),
    };

    // Build fabric.
    let mut fabric = Fabric::new();
    let router_a = Router { id: RouterId("Rx0y0".to_string()), routing: Default::default(), stats: Default::default() };
    let router_b = Router { id: RouterId("Rx0y1".to_string()), routing: Default::default(), stats: Default::default() };
    fabric.add_router(router_a);
    fabric.add_router(router_b);
    fabric.add_link(&RouterId("Rx0y0".to_string()), &RouterId("Rx0y1".to_string()), LinkConfig { mtu: None, delay_ms: 1, jitter_ms: 0, loss_percent: 0.0, load_balance: false });

    // Run the mock TUN processing.
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        tun::start(&cfg, &mut fabric).await.expect("tun start");
    });

    // Verify that output files were created and contain at least one line.
    let out1 = format!("{}_out.txt", path1);
    let out2 = format!("{}_out.txt", path2);
    let contents1 = std::fs::read_to_string(&out1).expect("read out1");
    let contents2 = std::fs::read_to_string(&out2).expect("read out2");
    assert!(!contents1.trim().is_empty(), "output for first file should not be empty");
    assert!(!contents2.trim().is_empty(), "output for second file should not be empty");

    // Verify that packets passed through the link (counter > 0).
    let edge_idx = fabric.link_index.values().next().expect("link exists");
    let edge = fabric.graph.edge_weight(*edge_idx).expect("edge weight");
    assert!(edge.counter() > 0, "link counter should be incremented");
}
