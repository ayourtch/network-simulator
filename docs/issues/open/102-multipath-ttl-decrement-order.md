# Issue 102: Multipath TTL Decrement Happens Before Destination Check

## Summary
In the multipath packet processing path (`process_packet_multi`), the TTL is decremented before checking if the packet has arrived at its destination. This is inconsistent with the single-path processing and causes packets to have their TTL decremented one extra time.

## Priority
**Critical** - Affects correctness of packet forwarding in multipath mode.

## Location
- File: `src/processor.rs`
- Lines: 224-228 (multipath - buggy) vs 103-112 (single-path - correct)

## Current Behavior

**Multipath (INCORRECT)** - Lines 224-262:
```rust
// TTL decrement happens FIRST
if let Err(e) = packet.decrement_ttl() {
    error!("Failed to decrement TTL: {}", e);
    break;
}
// ... routing table lookup ...
// ... then destination check happens LATER
if entries.is_empty() {
    debug!("No multipath entries for router {}", ingress.0);
    break;
}
```

**Single-path (CORRECT)** - Lines 103-112:
```rust
// Destination check happens FIRST
if next_hop == &ingress {
    debug!("Packet reached destination router {}", ingress.0);
    break;
}
// THEN TTL decrement
if let Err(e) = packet.decrement_ttl() {
    error!("Failed to decrement TTL: {}", e);
    break;
}
```

## Expected Behavior
Both processing paths should check if the packet has reached its destination BEFORE decrementing TTL. A packet arriving at its final destination should not have its TTL decremented.

## Impact
- Packets in multipath mode have TTL decremented one extra time
- Edge cases with low TTL values may trigger unnecessary Time Exceeded errors
- Inconsistent behavior between single-path and multipath modes

## Suggested Implementation

Reorder the multipath processing to match single-path:

```rust
// In process_packet_multi, after TTL expiration check (line 223):

// First, retrieve multipath table
let mtable = match tables.get(&ingress) {
    Some(t) => t,
    None => { /* handle error */ }
};

// Get entries for destination
let entries = match destination {
    Destination::TunA => &mtable.tun_a,
    Destination::TunB => &mtable.tun_b,
};

// Destination check BEFORE TTL decrement
if entries.is_empty() {
    debug!("Packet reached destination (no further hops)");
    break;
}

// NOW decrement TTL
if let Err(e) = packet.decrement_ttl() {
    error!("Failed to decrement TTL: {}", e);
    break;
}
```

## Resolution
(To be filled when resolved)

---
*Created: 2026-01-08*
