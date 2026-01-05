# Issue 031: ICMP Packet Routing After Generation Is Incorrect

## Summary
When an ICMP error packet is generated (e.g., for MTU exceeded), the code attempts to route it back to the source by parsing the ICMP bytes and continuing the forwarding loop. However, this approach has several problems that prevent correct ICMP delivery.

## Location
- File: `src/processor.rs`
- Function: `process_packet()` (lines 61-82)

## Current Behavior
```rust
if e == "mtu_exceeded" {
    let icmp_bytes = if is_ipv6(&packet) {
        icmp::generate_icmpv6_error(&packet, 2, 0)
    } else {
        icmp::generate_icmp_error(&packet, 3, 4)
    };
    // ...
    if let Ok(icmp_packet) = packet::parse(&icmp_bytes) {
        packet = icmp_packet;
        destination = opposite_destination(destination);
        continue; // forward the ICMP reply
    }
}
```

Issues:
1. **IPv4 ICMP parsing fails**: The stub `generate_icmp_error()` returns 8 bytes, which fails parsing (needs 20 bytes minimum for IP header)
2. **Wrong direction logic**: `opposite_destination()` flips the destination, but the ICMP should be routed toward the source, not just the opposite TUN
3. **No TTL reset**: The ICMP packet should have a fresh TTL (e.g., 64), not the decremented TTL from the original packet
4. **Ingress router not updated**: After generating ICMP at router X, the next iteration still uses router X as ingress, but the next-hop lookup uses the flipped destination, which may not be correct

## Expected Behavior (from Plan 7 and Plan 11)
1. Generate a valid ICMP error packet with:
   - Correct IP headers
   - Fresh TTL
   - Router's IP as source
   - Original sender's IP as destination
2. Look up the route to the original sender
3. Forward the ICMP packet using proper routing tables
4. The ICMP should eventually exit through the appropriate TUN

## Recommended Solution

1. Create a dedicated ICMP routing function:
```rust
async fn route_icmp_to_source(
    fabric: &mut Fabric,
    tables: &HashMap<RouterId, RoutingTable>,
    current_router: RouterId,
    icmp_bytes: Vec<u8>,
    original_destination: Destination,
) -> Option<PacketMeta> {
    // Parse the generated ICMP packet
    let icmp_packet = match packet::parse(&icmp_bytes) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to parse generated ICMP packet: {}", e);
            return None;
        }
    };
    
    // ICMP goes back toward the original source
    // If original was heading to TunB, ICMP goes toward TunA (and vice versa)
    let icmp_destination = opposite_destination(original_destination);
    
    // Route the ICMP packet from the current router
    let result = process_packet(
        fabric,
        tables,
        current_router,
        icmp_packet,
        icmp_destination,
    ).await;
    
    Some(result)
}
```

2. Use a Box or separate task for ICMP routing to avoid infinite recursion:
```rust
// In the main processing loop
if e == "mtu_exceeded" {
    let icmp_bytes = generate_proper_icmp(&packet, mtu);  // Issue 023
    
    // Don't use continue - this would cause issues
    // Instead, spawn a separate task or return the ICMP for routing
    if let Ok(icmp_packet) = packet::parse(&icmp_bytes) {
        // Return early, letting caller handle ICMP routing
        return (icmp_packet, ForwardingResult::IcmpGenerated(original_destination));
    }
    break;
}
```

3. Handle ICMP routing in the TUN handler:
```rust
match result {
    ForwardingResult::IcmpGenerated(original_dest) => {
        // Route ICMP back toward source
        let icmp_dest = opposite_destination(original_dest);
        let icmp_result = process_packet(
            fabric, tables, current_router, icmp_packet, icmp_dest
        ).await;
        // Write to appropriate TUN
    }
}
```

4. Add tests for ICMP routing:
```rust
#[tokio::test]
async fn test_icmp_mtu_exceeded_reaches_source() {
    // Create topology
    // Send oversized packet
    // Verify ICMP reaches the source TUN
}
```

## Files to Modify
- `src/processor.rs` (fix ICMP routing logic)
- `src/tun/mod.rs` (handle ICMP routing in TUN handler)
- `tests/` (add ICMP routing tests)

## Effort Estimate
Medium (3-4 hours)

## Dependencies
- Issue 023: IPv4 ICMP generation (ICMP must be valid to be parsed)
- Issue 027: Destination detection (need proper forwarding result)

## Related Plans
- Plan 7: ICMP Error Generation
- Plan 11 (resolved): ICMP Routing
