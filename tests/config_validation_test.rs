use network_simulator::config::SimulatorConfig;

#[test]
fn test_mutually_exclusive_packet_files() {
    let mut cfg = SimulatorConfig::default();
    cfg.packet_file = Some("single.txt".to_string());
    cfg.packet_files = Some(vec!["multi1.txt".to_string(), "multi2.txt".to_string()]);
    let result = cfg.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Both 'packet_file' and 'packet_files' are set; only one may be specified"
    );
}

#[test]
fn test_mismatched_injection_counts() {
    let mut cfg = SimulatorConfig::default();
    cfg.packet_files = Some(vec!["file1.txt".to_string(), "file2.txt".to_string()]);
    cfg.packet_inject_tuns = Some(vec!["tun_a".to_string()]); // mismatched length
    let result = cfg.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Number of packet files (2) does not match number of injection directions (1)"));
}

#[test]
fn test_inject_without_file() {
    let mut cfg = SimulatorConfig::default();
    cfg.packet_inject_tun = Some("tun_a".to_string());
    // No packet_file set
    let result = cfg.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "'packet_inject_tun' specified without a 'packet_file'"
    );
}

#[test]
fn test_injects_without_files() {
    let mut cfg = SimulatorConfig::default();
    cfg.packet_inject_tuns = Some(vec!["tun_a".to_string()]);
    // No packet_files set
    let result = cfg.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "'packet_inject_tuns' specified without 'packet_files'"
    );
}
