use network_simulator::config::SimulatorConfig;
use toml;

#[test]
fn test_link_reference_validation_fails_on_unknown_router() {
    let toml_str = r#"
        [topology]
        routers = { Rx0y0 = {} }
        links = { "Rx0y0_Rx1y1" = { delay_ms = 1 } }
    "#;
    let cfg: SimulatorConfig = toml::from_str(toml_str).expect("parse config");
    let result = cfg.validate();
    assert!(
        result.is_err(),
        "Validation should fail for unknown router in link"
    );
    let err_msg = result.err().unwrap();
    assert!(
        err_msg.contains("unknown router"),
        "Error should mention unknown router"
    );
}
