use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_config_packet_file() {
    // Create a temporary packet file with a simple IPv4 packet hex string
    let packet_hex = "450000140000000040060000c0a80101c0a80102"; // minimal packet
    let mut tmp_packet = NamedTempFile::new().expect("temp packet file");
    writeln!(tmp_packet, "{}", packet_hex).expect("write packet");
    let packet_path = tmp_packet.path().to_str().unwrap().to_string();

    // Minimal config that includes packet_file entry
    let cfg_content = format!(r#"
[simulation]
mtu = 1500

[interfaces]
tun_a = "tunA"

tun_b = "tunB"

# Optional mock packet file (overridden by CLI)
packet_file = "{}"

[interfaces.real_tun]
name = "tun0"
address = "10.0.0.1"
netmask = "255.255.255.0"

[tun_ingress]
tun_a_ingress = "Rx0y0"
tun_b_ingress = "Rx5y5"

[topology.routers]
Rx0y0 = {{}}
Rx5y5 = {{}}

[topology.links]
Rx0y0_Rx5y5 = {{ delay_ms = 0 }}
"#, packet_path);
    let cfg_path = "tests/tmp_config_packet.toml";
    fs::write(cfg_path, cfg_content).expect("write config");

    let mut cmd = cargo_bin_cmd!("network-simulator");
    cmd.arg("--config").arg(cfg_path);
    cmd.assert().success();

    let _ = fs::remove_file(cfg_path);
}
