use assert_cmd::Command;
use std::fs;

#[test]
fn test_cli_packet_file_missing() {
    // Provide a non‑existent packet file path
    let missing_path = "nonexistent_packet_file.txt";

    // Minimal config without packet_file (will be set via CLI)
    let cfg_content = r#"
[simulation]
mtu = 1500

[interfaces]
tun_a = \"tunA\"

[tun_ingress]
tun_a_ingress = \"Rx0y0\"

[topology.routers]
Rx0y0 = {}

[topology.links]
Rx0y0_Rx0y0 = { delay_ms = 0 }
"#;
    let cfg_path = "tests/tmp_missing_config.toml";
    fs::write(&cfg_path, cfg_content).expect("write config");

    let mut cmd = Command::cargo_bin("network-simulator").expect("binary exists");
    cmd.arg("--config").arg(&cfg_path)
        .arg("--packet-file").arg(missing_path);
    // Expect the command to fail because the packet file cannot be opened
    cmd.assert().failure();

    let _ = fs::remove_file(cfg_path);
}
