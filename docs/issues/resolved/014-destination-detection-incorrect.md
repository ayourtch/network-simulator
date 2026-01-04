# Issue 014: Destination Detection Based on Routing Table Incorrect

## Summary
In `src/forwarding/mod.rs`, the path selection always uses `routing.tun_a.next_hop` regardless of the actual packet destination. There's no mechanism to determine whether a packet is heading toward TUN A or TUN B.

## Location
- File: `src/forwarding/mod.rs`
- Function: `select_egress_link()`

## Current Behavior
```rust
pub fn select_egress_link<'a>(
    router_id: &RouterId,
    packet: &PacketMeta,
    links: &'a [&Link],
    tables: &HashMap<RouterId, crate::routing::RoutingTable>,
) -> Option<&'a Link> {
    // ...
    let routing = tables.get(router_id)?;
    let next_hop = &routing.tun_a.next_hop; // Always uses tun_a!
    // ...
}
```

This always selects the next hop toward TUN A, even when the packet should be routed toward TUN B.

## Expected Behavior (from Plan 5)
The routing decision should be based on the packet's destination:
- Packets from TUN A should route toward TUN B (use `routing.tun_b`)
- Packets from TUN B should route toward TUN A (use `routing.tun_a`)

## Recommended Solution

1. Add a `Destination` parameter to the egress link selection:
```rust
pub fn select_egress_link<'a>(
    router_id: &RouterId,
    packet: &PacketMeta,
    links: &'a [&Link],
    tables: &HashMap<RouterId, crate::routing::RoutingTable>,
    destination: crate::routing::Destination,  // Add this parameter
) -> Option<&'a Link> {
    let routing = tables.get(router_id)?;
    let next_hop = match destination {
        crate::routing::Destination::TunA => &routing.tun_a.next_hop,
        crate::routing::Destination::TunB => &routing.tun_b.next_hop,
    };
    // ...
}
```

2. Determine destination based on packet source in processor.rs:
```rust
// Packets from TUN A go to TUN B and vice versa
let destination = if is_from_tun_a(&ingress) {
    Destination::TunB
} else {
    Destination::TunA
};
```

3. Update callers to pass the destination.

4. Similarly update `forwarding/multipath.rs`.

## Files to Modify
- `src/forwarding/mod.rs`
- `src/forwarding/multipath.rs`
- `src/processor.rs`
- `tests/` (update tests)

## Effort Estimate
Small (1-2 hours)

## Related Plans
- Plan 5: Routing Table Computation
- Plan 9: Integration and End-to-End Testing
