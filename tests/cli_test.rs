use assert_cmd::cargo_bin_cmd;
use predicates::str::contains;
use std::fs;
use tempfile;


#[test]
fn test_cli_multipath_enabled_verbose() {
    // Run the binary with multipath flag and verbose level 2
    let mut cmd = cargo_bin_cmd!("network-simulator");
    cmd.arg("--config")
        .arg("config.toml")
        .arg("--multipath")
        .arg("-vv");

    cmd.assert()
        .success()
        .stdout(contains("Multipath routing enabled"));
}

#[test]
fn test_cli_multipath_disabled() {
    // Create a temporary config without multipath enabled
    let tmp_dir = tempfile::tempdir().expect("tempdir");
    let cfg_path = tmp_dir.path().join("config_no_mp.toml");
    let cfg_contents = r#"[simulation]
mtu = 1500
seed = 42

[interfaces]
tun_a = "tunA"
tun_b = "tunB"

[tun_ingress]
tun_a_ingress = "Rx0y0"
tun_b_ingress = "Rx0y1"

[topology.routers]
Rx0y0 = {}
Rx0y1 = {}

[topology.links]
Rx0y0_Rx0y1 = { delay_ms = 10, load_balance = true }
"#;
    fs::write(&cfg_path, cfg_contents).expect("write config");

    let mut cmd = cargo_bin_cmd!("network-simulator");
    cmd.arg("--config")
        .arg(cfg_path)
        .arg("-v"); // no --multipath flag

    cmd.assert()
        .success()
        .stdout(contains("Multipath routing disabled"));
}
