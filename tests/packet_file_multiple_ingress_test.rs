use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_cli_packet_file_multiple_ingress() {
    // Create a temporary packet file with two packets: one for each ingress
    let mut tmp_packet = NamedTempFile::new().expect("temp packet file");
    // Packet with source IP 10.0.0.1 (ingress A)
    writeln!(tmp_packet, "450000140000000040060000c0a80101c0a80102").expect("write packet A");
    // Packet with source IP 192.168.0.1 (ingress B)
    writeln!(tmp_packet, "450000140000000040060000c0a80001c0a80102").expect("write packet B");
    let packet_path = tmp_packet.path().to_str().unwrap().to_string();

    // Minimal config with both routers and ingresses
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
    let cfg_path = "tests/tmp_config_multi_ingress.toml";
    fs::write(&cfg_path, cfg_content).expect("write config");

    let mut cmd = cargo_bin_cmd!("network-simulator");
    cmd.arg("--config").arg(&cfg_path)
        .arg("--packet-file").arg(&packet_path);
    cmd.assert().success();

    let _ = fs::remove_file(cfg_path);
}
