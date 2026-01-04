use assert_cmd::Command;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_cli_packet_file_multipath() {
    // Temporary packet file with a simple IPv4 packet hex string
    let packet_hex = "450000140000000040060000c0a80101c0a80102";
    let mut tmp_packet = NamedTempFile::new().expect("temp packet file");
    writeln!(tmp_packet, "{}", packet_hex).expect("write packet");
    let packet_path = tmp_packet.path().to_str().unwrap().to_string();

    // Minimal config with topology
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
    let cfg_path = "tests/tmp_config_multipath.toml";
    fs::write(&cfg_path, cfg_content).expect("write config");

    let mut cmd = Command::cargo_bin("network-simulator").expect("binary exists");
    cmd.arg("--config").arg(&cfg_path)
        .arg("--packet-file").arg(&packet_path)
        .arg("--multipath");
    cmd.assert().success();

    let _ = fs::remove_file(cfg_path);
}
