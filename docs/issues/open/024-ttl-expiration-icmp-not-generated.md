# Issue 024: TTL Expiration Does Not Generate ICMP Time Exceeded

## Summary
When a packet's TTL reaches 1 and is decremented to 0, the processor should generate an ICMP Time Exceeded message and send it back to the source. Currently, the code just decrements TTL and logs an error, but no ICMP is generated for TTL expiration.

## Location
- File: `src/processor.rs`
- Function: `process_packet()` (lines 38-44)

## Current Behavior
```rust
loop {
    // Decrement TTL / Hop Limit.
    if let Err(e) = packet.decrement_ttl() {
        error!("Failed to decrement TTL: {}", e);
        break;  // Just breaks out of loop - no ICMP generated
    }
    // ... rest of processing
}
```

The `decrement_ttl()` function returns `Err("TTL already zero")` when TTL is 0, but:
1. It doesn't return an error when TTL is 1 (which would become 0 after decrement)
2. No ICMP Time Exceeded is generated when TTL expires
3. The packet is silently dropped

## Expected Behavior (from Plan 4 and Plan 7)
1. Check TTL **before** decrementing
2. If TTL <= 1, generate ICMP Time Exceeded (Type 11, Code 0)
3. Route the ICMP packet back toward the source
4. Drop the original packet

## Recommended Solution

1. Add TTL expiration check before decrement in `process_packet()`:
```rust
loop {
    // Check if TTL is about to expire (will be 0 or 1 after this hop)
    if packet.ttl <= 1 {
        debug!("TTL expired at router {}", ingress.0);
        // Generate ICMP Time Exceeded
        let icmp_bytes = if is_ipv6(&packet) {
            icmp::generate_icmpv6_error(&packet, 3, 0)  // Type 3 = Time Exceeded for ICMPv6
        } else {
            icmp::generate_icmp_error(&packet, 11, 0)  // Type 11 = Time Exceeded
        };
        
        // Increment ICMP counter for this router
        if let Some(node_idx) = fabric.router_index.get(&ingress) {
            if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                router.increment_icmp();
            }
        }
        
        // Parse and route ICMP packet back to source
        if let Ok(icmp_packet) = packet::parse(&icmp_bytes) {
            packet = icmp_packet;
            destination = opposite_destination(destination);
            continue;  // Forward the ICMP reply
        }
        break;
    }
    
    // Now safe to decrement TTL
    if let Err(e) = packet.decrement_ttl() {
        error!("Failed to decrement TTL: {}", e);
        break;
    }
    // ... rest of processing
}
```

2. Update `PacketMeta::decrement_ttl()` to check for TTL=1:
```rust
pub fn decrement_ttl(&mut self) -> Result<(), &'static str> {
    if self.ttl <= 1 {
        return Err("TTL would expire");
    }
    // ... rest of implementation
}
```

3. Add unit test for TTL expiration handling:
```rust
#[tokio::test]
async fn test_ttl_expiration_generates_icmp() {
    // Create packet with TTL=1
    // Process through fabric
    // Verify ICMP Time Exceeded is returned
}
```

## Files to Modify
- `src/processor.rs` (add TTL expiration check)
- `src/packet/mod.rs` (optionally update decrement_ttl logic)
- `tests/` (add TTL expiration tests)

## Effort Estimate
Small (1-2 hours)

## Dependencies
- Issue 023: IPv4 ICMP error generation must be fixed first

## Related Plans
- Plan 4: Packet Processing and Forwarding Engine
- Plan 7: ICMP Error Generation
