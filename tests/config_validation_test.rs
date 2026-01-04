use network_simulator::config::SimulatorConfig;

#[test]
fn test_invalid_link_reference() {
    let toml = r#"
        [topology.routers]
        Rx0y0 = {}
        Rx5y5 = {}
        
        [topology.links]
        Rx0y0_Rx9y9 = { delay_ms = 10, jitter_ms = 0, loss_percent = 0, mtu = 1500 }
    "#;
    let cfg: SimulatorConfig = toml::from_str(toml).expect("parse config");
    let res = cfg.validate();
    assert!(res.is_err());
    if let Err(msg) = res {
        assert!(msg.contains("unknown router"));
    }
}
