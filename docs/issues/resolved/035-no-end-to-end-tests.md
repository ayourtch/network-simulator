# Issue 035: No Tests for End-to-End Packet Delivery

## Summary
While there are many unit tests for individual components, there are no integration tests that verify end-to-end packet delivery from ingress to egress through the complete fabric. This is critical for verifying the system works as a whole.

## Location
- File: `tests/` directory
- Missing: `tests/end_to_end_test.rs`

## Current Test Coverage
The existing tests cover:
- Configuration parsing and validation
- Packet parsing (IPv4, IPv6)
- Router ID validation
- Routing table computation
- Link simulation
- Mock TUN packet processing

But there's no test that verifies:
- A packet entering at TUN A exits at TUN B
- Packets traverse the expected path through the fabric
- TTL is correctly decremented at each hop
- Statistics are correctly updated at each router

## Expected Behavior (from Plan 9)
Plan 9 describes integration tests that should:
1. Create a fabric with known topology
2. Inject packets at ingress
3. Verify packets arrive at egress with expected modifications
4. Measure delay matches configuration
5. Verify packet loss matches configuration

## Recommended Solution

1. Create end-to-end integration test:
```rust
// tests/end_to_end_test.rs

use network_simulator::config::SimulatorConfig;
use network_simulator::topology::{Fabric, Router, RouterId};
use network_simulator::routing::{compute_routing, Destination};
use network_simulator::processor::process_packet;
use network_simulator::packet::parse;

fn create_test_fabric() -> (Fabric, SimulatorConfig) {
    // Create a simple linear topology: A -> B -> C
    // Rx0y0 -> Rx0y1 -> Rx0y2
    let toml = r#"
        [simulation]
        mtu = 1500
        
        [tun_ingress]
        tun_a_ingress = "Rx0y0"
        tun_b_ingress = "Rx0y2"
        
        [topology.routers]
        Rx0y0 = {}
        Rx0y1 = {}
        Rx0y2 = {}
        
        [topology.links.Rx0y0_Rx0y1]
        delay_ms = 10
        
        [topology.links.Rx0y1_Rx0y2]
        delay_ms = 10
    "#;
    
    let cfg: SimulatorConfig = toml::from_str(toml).unwrap();
    // Build fabric from config
    // ...
    (fabric, cfg)
}

#[tokio::test]
async fn test_packet_traverses_fabric_and_reaches_destination() {
    let (mut fabric, cfg) = create_test_fabric();
    let ingress_a = RouterId(cfg.tun_ingress.tun_a_ingress.clone());
    let ingress_b = RouterId(cfg.tun_ingress.tun_b_ingress.clone());
    let tables = compute_routing(&fabric, ingress_a.clone(), ingress_b.clone());
    
    // Create a test packet with TTL=64
    let packet_bytes = vec![
        0x45, 0x00, 0x00, 0x28,  // IPv4, length=40
        0x00, 0x00, 0x00, 0x00,
        0x40, 0x06, 0x00, 0x00,  // TTL=64, TCP
        0x0a, 0x00, 0x00, 0x01,  // src=10.0.0.1
        0x0a, 0x00, 0x00, 0x02,  // dst=10.0.0.2
        // ... TCP header ...
    ];
    
    let packet = parse(&packet_bytes).unwrap();
    let initial_ttl = packet.ttl;
    
    // Process packet from ingress_a toward ingress_b
    let result = process_packet(
        &mut fabric, &tables, ingress_a.clone(), packet, Destination::TunB
    ).await;
    
    // Verify TTL was decremented (2 hops = 2 decrements)
    assert_eq!(result.ttl, initial_ttl - 2);
    
    // Verify packet is parseable
    assert!(result.raw.len() >= 20);
}

#[tokio::test]
async fn test_ttl_expiration_generates_icmp() {
    let (mut fabric, cfg) = create_test_fabric();
    let tables = compute_routing(&fabric, ...);
    
    // Create packet with TTL=1 (will expire at first hop)
    let packet_bytes = vec![
        0x45, 0x00, 0x00, 0x28,
        0x00, 0x00, 0x00, 0x00,
        0x01, 0x06, 0x00, 0x00,  // TTL=1
        // ...
    ];
    
    let packet = parse(&packet_bytes).unwrap();
    let result = process_packet(&mut fabric, &tables, ...).await;
    
    // Verify ICMP Time Exceeded was generated
    // (depends on Issue 024 being fixed)
    assert_eq!(result.protocol, 1);  // ICMP
    // Verify ICMP type is 11 (Time Exceeded)
    // ...
}

#[tokio::test]
async fn test_statistics_updated_at_each_hop() {
    let (mut fabric, cfg) = create_test_fabric();
    // ...
    
    // Get initial statistics
    let initial_stats = fabric.get_statistics();
    
    // Process packet
    process_packet(&mut fabric, &tables, ...).await;
    
    // Verify statistics were updated at each router in path
    let final_stats = fabric.get_statistics();
    assert!(final_stats.get(&RouterId("Rx0y0".into())).unwrap().packets_received > 
            initial_stats.get(&RouterId("Rx0y0".into())).unwrap().packets_received);
}
```

2. Add test for delay measurement:
```rust
#[tokio::test]
async fn test_delay_matches_configuration() {
    // Configure 50ms delay on each link
    // Process packet
    // Verify elapsed time is ~100ms (2 hops * 50ms)
}
```

3. Add test for packet loss:
```rust
#[tokio::test]
async fn test_packet_loss_rate_matches_configuration() {
    // Configure 10% loss on each link
    // Send 1000 packets
    // Verify ~19% are lost (1 - 0.9^2)
}
```

## Files to Modify
- `tests/end_to_end_test.rs` (create new file)

## Effort Estimate
Medium (3-4 hours)

## Dependencies
- Issues 024, 026, 027 (TTL handling, statistics, destination detection)

## Related Plans
- Plan 9: Integration and End-to-End Testing
