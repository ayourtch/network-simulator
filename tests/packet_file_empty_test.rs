use assert_cmd::Command;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_cli_empty_packet_file() {
    // Create an empty temporary packet file (kept alive)
    let tmp_packet = NamedTempFile::new().expect("temp file");
    let empty_path = tmp_packet.path().to_str().unwrap().to_string();

    // Minimal config with topology including both routers and ingresses
    let cfg_content = r#"
[simulation]
mtu = 1500

[interfaces]
tun_a = "tunA"

[tun_ingress]
tun_a_ingress = "Rx0y0"

tun_b_ingress = "Rx5y5"

[topology.routers]
Rx0y0 = {}
Rx5y5 = {}

[topology.links]
Rx0y0_Rx5y5 = { delay_ms = 0 }
"#;
    let cfg_path = "tests/tmp_config_empty_packet.toml";
    fs::write(&cfg_path, cfg_content).expect("write config");

    let mut cmd = Command::cargo_bin("network-simulator").expect("binary exists");
    cmd.arg("--config").arg(&cfg_path)
        .arg("--packet-file").arg(&empty_path);
    // Should succeed even if the packet file is empty
    cmd.assert().success();

    let _ = fs::remove_file(cfg_path);
}
