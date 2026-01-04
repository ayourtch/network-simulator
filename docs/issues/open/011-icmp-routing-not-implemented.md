# Issue 011: ICMP Routing Back to Source Not Implemented

## Summary
When an ICMP error is generated (e.g., TTL exceeded), it needs to be routed back to the original packet's source IP address. Currently, even when ICMP generation is implemented, there's no mechanism to route the ICMP packet back.

## Location
- Files: `src/icmp/mod.rs`, `src/processor.rs`

## Current Behavior
The ICMP generation function (currently a stub) is not integrated with the routing system. Even if it generated valid ICMP packets, they would not be delivered to the source.

## Expected Behavior (from Plan 7 and Plan 9)
1. Generate ICMP error packet at the router where the error occurred
2. Inject the ICMP packet into the fabric at the current router
3. Route the ICMP packet toward the original source IP
4. Deliver to appropriate TUN interface

## Recommended Solution

1. Determine the correct destination for ICMP based on original source:
```rust
fn route_icmp_packet(
    fabric: &Fabric,
    tables: &HashMap<RouterId, RoutingTable>,
    icmp_packet: Vec<u8>,
    current_router: RouterId,
    original_src: IpAddr,
) -> Result<(), Error> {
    // Determine which TUN the original source came from
    // This requires knowing which subnet/IP range belongs to each TUN
    
    // Route ICMP packet to that TUN
    let destination = if is_from_tun_a_subnet(&original_src) {
        Destination::TunA
    } else {
        Destination::TunB
    };
    
    // Process the ICMP packet through the fabric
    // (reverse direction from original)
}
```

2. The ICMP packet needs to traverse the fabric like a regular packet (but in the opposite direction).

3. Consider the ICMP packet's own TTL to prevent infinite loops.

## Files to Modify
- `src/processor.rs` (add ICMP routing logic)
- `src/icmp/mod.rs` (integrate with processor)
- `tests/` (add ICMP routing tests)

## Effort Estimate
Medium (3-4 hours)

## Dependencies
- Issue 005: ICMP Time Exceeded generation
- Issue 006: ICMP Fragmentation Needed generation
- Issue 008: Hop-by-hop forwarding

## Related Plans
- Plan 7: ICMP Error Generation
- Plan 9: Integration and End-to-End Testing
