# Issue 104: Load Balancing Hash Includes Mutable Counter

## Summary
The load balancing implementation includes the sum of link packet counters in the hash calculation. Since this counter changes with every packet, the same flow (same 5-tuple) may be hashed to different links over time, breaking flow affinity.

## Priority
**High** - Causes packet reordering for TCP connections.

## Location
- File: `src/processor.rs`
- Lines: 294-298 (in `process_packet_multi`)
- Also: `src/forwarding/mod.rs` Lines 55-59 (in `select_egress_link`)

## Current Behavior

```rust
// Include sum of counters of load‑balanced links.
let total_counter: u64 = lb_links
    .iter()
    .map(|l| l.counter.load(Ordering::Relaxed))
    .sum();
total_counter.hash(&mut hasher);
let hash = hasher.finish();
let idx = (hash as usize) % lb_links.len();
```

The counter is incremented every time a packet traverses a link, so the hash result changes over time even for the same 5-tuple.

## Expected Behavior
- Flow affinity: Packets with the same 5-tuple (src_ip, dst_ip, src_port, dst_port, protocol) should always take the same path
- Counter should only be used for per-packet round-robin mode, not flow-based hashing
- The hash should be deterministic for a given flow

## Impact
- TCP connections may experience out-of-order packet delivery
- This triggers TCP fast retransmit and reduces throughput
- Breaks the expected ECMP (Equal Cost Multi-Path) behavior where flows stick to paths

## Suggested Implementation

Option 1: Remove counter from hash entirely for flow-based load balancing:
```rust
let mut hasher = DefaultHasher::new();
packet.src_ip.hash(&mut hasher);
packet.dst_ip.hash(&mut hasher);
packet.src_port.hash(&mut hasher);
packet.dst_port.hash(&mut hasher);
packet.protocol.hash(&mut hasher);
// DO NOT include counter for flow-based hashing
let hash = hasher.finish();
let idx = (hash as usize) % lb_links.len();
```

Option 2: Add explicit per-packet mode that uses counter:
```rust
if link.cfg.per_packet_load_balance {
    // Round-robin using counter
    let idx = (link.counter.fetch_add(1, Ordering::Relaxed) as usize) % lb_links.len();
    *lb_links[idx]
} else {
    // Flow-based: hash without counter
    let mut hasher = DefaultHasher::new();
    // ... hash 5-tuple only ...
    let idx = (hasher.finish() as usize) % lb_links.len();
    *lb_links[idx]
}
```

## Resolution
(To be filled when resolved)

---
*Created: 2026-01-08*
