// tests/destination_test.rs

use network_simulator::routing::Destination;

#[test]
fn test_destination_enum_variants() {
    let a = Destination::TunA;
    let b = Destination::TunB;
    // Ensure they are distinct and match pattern
    match a {
        Destination::TunA => {}
        _ => panic!("Expected TunA"),
    }
    match b {
        Destination::TunB => {}
        _ => panic!("Expected TunB"),
    }
    // Ensure Debug output contains variant names
    let dbg_a = format!("{:?}", a);
    let dbg_b = format!("{:?}", b);
    assert!(dbg_a.contains("TunA"));
    assert!(dbg_b.contains("TunB"));
}
