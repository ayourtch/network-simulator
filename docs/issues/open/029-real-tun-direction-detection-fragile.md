# Issue 029: Real TUN Mode Direction Detection Based on IP Prefix

## Summary
When running in real TUN mode, the packet direction (which ingress router and destination to use) is determined by checking if the source IP starts with "10.". This is a fragile heuristic that will break for many real-world use cases.

## Location
- File: `src/tun/mod.rs`
- Function: `start()` (lines 199-203)

## Current Behavior
```rust
let (ingress, destination) = if packet.src_ip.to_string().starts_with("10.") {
    (ingress_a.clone(), Destination::TunB)
} else {
    (ingress_b.clone(), Destination::TunA)
};
```

Issues:
1. Assumes 10.0.0.0/8 addresses always come from TUN A
2. Breaks if user configures different IP ranges
3. Doesn't work correctly for IPv6 addresses
4. No configuration option to customize the detection logic
5. When dual-TUN is implemented (Issue 028), direction should be determined by which TUN the packet came from, not by IP

## Expected Behavior
Direction should be determined by:
1. **Which TUN device the packet arrived from** (preferred, after Issue 028 is fixed)
2. **Configurable IP prefix matching** (as fallback for single-TUN mode)
3. **Support for IPv6 prefixes**

## Recommended Solution

1. For dual-TUN mode (after Issue 028), direction is automatic:
```rust
// In the dual-TUN select! loop:
select! {
    read_res = async_dev_a.read(&mut buf_a) => {
        // Came from TUN A -> destination is TUN B
        let packet = parse(&buf_a[..n])?;
        let processed = process_packet(
            fabric, &routing_tables, ingress_a.clone(),
            packet, Destination::TunB  // Always TunB for packets from TunA
        ).await;
        async_dev_b.write_all(&processed.raw).await?;
    }
    
    read_res = async_dev_b.read(&mut buf_b) => {
        // Came from TUN B -> destination is TUN A
        let packet = parse(&buf_b[..n])?;
        let processed = process_packet(
            fabric, &routing_tables, ingress_b.clone(),
            packet, Destination::TunA  // Always TunA for packets from TunB
        ).await;
        async_dev_a.write_all(&processed.raw).await?;
    }
}
```

2. For single-TUN mode, add configurable prefix matching:
```toml
# In config.toml
[tun_ingress]
tun_a_ingress = "Rx0y0"
tun_b_ingress = "Rx5y5"
# New: prefix-based direction detection
tun_a_prefixes = ["10.0.0.0/8", "fd00::/8"]  # Packets from these -> TunB
tun_b_prefixes = ["192.168.0.0/16", "2001:db8::/32"]  # Packets from these -> TunA
```

3. Add prefix matching logic:
```rust
use std::net::IpAddr;

fn determine_direction(
    src_ip: &IpAddr,
    tun_a_prefixes: &[IpNetwork],
    tun_b_prefixes: &[IpNetwork],
    ingress_a: &RouterId,
    ingress_b: &RouterId,
) -> (RouterId, Destination) {
    for prefix in tun_a_prefixes {
        if prefix.contains(*src_ip) {
            return (ingress_a.clone(), Destination::TunB);
        }
    }
    for prefix in tun_b_prefixes {
        if prefix.contains(*src_ip) {
            return (ingress_b.clone(), Destination::TunA);
        }
    }
    // Default fallback
    (ingress_a.clone(), Destination::TunB)
}
```

4. Add tests for prefix matching:
```rust
#[test]
fn test_direction_detection_ipv4() {
    // Test various IPv4 prefixes
}

#[test]
fn test_direction_detection_ipv6() {
    // Test IPv6 prefix matching
}
```

## Files to Modify
- `src/config.rs` (add prefix configuration)
- `src/tun/mod.rs` (implement prefix matching)
- `tests/` (add direction detection tests)

## Effort Estimate
Small (1-2 hours)

## Dependencies
- Issue 028: Dual TUN support (direction is automatic with dual TUN)

## Related Plans
- Plan 3: TUN Interface Management
- Plan 4: Packet Processing and Forwarding Engine
