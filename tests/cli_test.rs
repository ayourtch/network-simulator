use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

#[test]
fn test_cli_overrides_real_tun() {
    // Write a minimal config file with proper topology sections
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
    let cfg_path = "tests/tmp_config.toml";
    fs::write(cfg_path, cfg_content).expect("write config");

    let mut cmd = cargo_bin_cmd!("network-simulator");
    cmd.arg("--config").arg(cfg_path)
        .arg("--tun-name").arg("tun_test0")
        .arg("--tun-address").arg("10.1.0.1")
        .arg("--tun-netmask").arg("255.255.255.0");
    cmd.assert().success();
    let _ = fs::remove_file(cfg_path);
}
