

## Summary
The `Link` struct has an atomic counter for per-packet load balancing, but it's only incremented in `simulate_link()` and not used in the path selection hash as specified in Plan 8.

## Location
- Files: `src/topology/link.rs`, `src/forwarding/mod.rs`, `src/forwarding/multipath.rs`

## Current Behavior
In `src/simulation/mod.rs`:
```rust
// Counter is incremented
link.counter.fetch_add(1, Ordering::Relaxed);
```

In `src/forwarding/mod.rs` and `multipath.rs`:
```rust
// load_balance flag is checked, but counter is NOT included in hash
let lb_links: Vec<&&Link> = candidates.iter().filter(|&&l| l.cfg.load_balance).collect();
// ... hash is computed without counter
```

The `load_balance` flag filters links, but the actual counter value is not included in the hash computation.

## Expected Behavior (from Plan 8)
When `load_balance = true` on a link:
1. The per-packet counter should participate in the hash
2. This causes different packets (even from the same flow) to potentially take different paths
3. This enables per-packet load balancing (vs flow-based sticky routing)

## Recommended Solution

1. Pass the link counter into the hash when `load_balance` is enabled:
```rust
// In forwarding/mod.rs - select_egress_link()
if !lb_links.is_empty() {
    let mut hasher = DefaultHasher::new();
    // Hash 5-tuple
    packet.src_ip.hash(&mut hasher);
    packet.dst_ip.hash(&mut hasher);
    packet.src_port.hash(&mut hasher);
    packet.dst_port.hash(&mut hasher);
    packet.protocol.hash(&mut hasher);
    
    // For per-packet LB, include counter
    // Get total counter from all LB links
    let total_counter: u64 = lb_links.iter()
        .map(|l| l.counter.load(Ordering::Relaxed))
        .sum();
    total_counter.hash(&mut hasher);
    
    let hash = hasher.finish();
    let idx = (hash as usize) % lb_links.len();
    // ...
}
```

2. Similarly update `forwarding/multipath.rs`.

3. Add tests that verify packets spread across paths when `load_balance = true`.

## Files to Modify
- `src/forwarding/mod.rs`
- `src/forwarding/multipath.rs`
- `tests/multipath_forwarding_test.rs` (add per-packet LB test)

## Effort Estimate
Small (1-2 hours)

## Related Plans
- Plan 8: Multi-path Routing and Load Balancing
