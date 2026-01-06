use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_cli_packet_file_with_comments() {
    // Create a temporary packet file with comment lines and a valid packet
    let mut tmp_packet = NamedTempFile::new().expect("temp packet file");
    writeln!(tmp_packet, "# This is a comment line").expect("write comment");
    writeln!(tmp_packet, "").expect("write empty line");
    let valid_hex = "450000140000000040060000c0a80101c0a80102";
    writeln!(tmp_packet, "{}", valid_hex).expect("write valid packet");
    let packet_path = tmp_packet.path().to_str().unwrap().to_string();

    // Minimal config with topology and both ingresses
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
    let cfg_path = "tests/tmp_config_comments.toml";
    fs::write(&cfg_path, cfg_content).expect("write config");

    let mut cmd = cargo_bin_cmd!("network-simulator");
    cmd.arg("--config")
        .arg(&cfg_path)
        .arg("--packet-file")
        .arg(&packet_path);
    cmd.assert().success();

    let _ = fs::remove_file(cfg_path);
}
