use network_simulator::config::SimulatorConfig;

#[test]
fn test_invalid_packet_inject_tun_value() {
    let mut cfg = SimulatorConfig::default();
    cfg.packet_file = Some("dummy.txt".to_string());
    cfg.packet_inject_tun = Some("invalid_tun".to_string());
    let result = cfg.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Invalid packet_inject_tun value 'invalid_tun', expected 'tun_a' or 'tun_b'");
}

#[test]
fn test_invalid_packet_inject_tuns_values() {
    let mut cfg = SimulatorConfig::default();
    cfg.packet_files = Some(vec!["file1.txt".to_string()]);
    cfg.packet_inject_tuns = Some(vec!["tun_a".to_string(), "bad".to_string()]);
    let result = cfg.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Invalid packet_inject_tuns value 'bad', expected 'tun_a' or 'tun_b'");
}
