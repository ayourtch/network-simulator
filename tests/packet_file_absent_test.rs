use network_simulator::config::SimulatorConfig;
use std::fs;

#[test]
fn test_config_without_packet_file() {
    // Minimal config without packet_file entry
    let cfg_content = r#"
[simulation]
mtu = 1500

[interfaces]
tun_a = "tunA"

[tun_ingress]
tun_a_ingress = "Rx0y0"

[topology.routers]
Rx0y0 = {}

[topology.links]
Rx0y0_Rx0y0 = { delay_ms = 0 }
"#;
    let cfg_path = "tests/tmp_config_no_packet.toml";
    fs::write(&cfg_path, cfg_content).expect("write config");

    let cfg_str = fs::read_to_string(cfg_path).expect("read config");
    let cfg: SimulatorConfig = toml::from_str(&cfg_str).expect("parse config");
    assert!(cfg.packet_file.is_none(), "packet_file should be None when not present");

    let _ = fs::remove_file(cfg_path);
}
