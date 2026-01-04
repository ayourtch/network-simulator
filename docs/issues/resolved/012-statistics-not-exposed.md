# Issue 012: Statistics Collection Not Exposed

## Summary
The `RouterStats` structure exists with fields for packets_received, packets_forwarded, and icmp_generated, but these statistics are never updated or exposed to users.

## Location
- File: `src/topology/router.rs`
- Struct: `RouterStats`

## Current Behavior
```rust
#[derive(Debug, Default, Clone)]
pub struct RouterStats {
    pub packets_received: u64,
    pub packets_forwarded: u64,
    pub icmp_generated: u64,
}
```

The structure exists but:
1. Statistics are never incremented during packet processing
2. No API to retrieve current statistics
3. No logging or reporting of statistics

## Expected Behavior (from Plan 6 and docs)
From `docs/build_and_run_instructions.md`:
> Metrics (Future Extension) - The current version does not expose Prometheus metrics, but the logging infrastructure makes it easy to add counters.

While Prometheus is marked as future, basic statistics collection should:
1. Increment counters during packet processing
2. Provide a way to retrieve/display statistics
3. Log summary statistics at shutdown

## Recommended Solution

1. Add methods to update statistics in `Router` struct:
```rust
impl Router {
    pub fn increment_received(&mut self) {
        self.stats.packets_received += 1;
    }
    
    pub fn increment_forwarded(&mut self) {
        self.stats.packets_forwarded += 1;
    }
    
    pub fn increment_icmp(&mut self) {
        self.stats.icmp_generated += 1;
    }
}
```

2. Update statistics in `processor.rs`:
```rust
// When a packet is received at a router
router.increment_received();

// When a packet is successfully forwarded
router.increment_forwarded();

// When an ICMP error is generated
router.increment_icmp();
```

3. Add a method to print summary statistics:
```rust
impl Fabric {
    pub fn print_statistics(&self) {
        for (id, node_idx) in &self.router_index {
            let router = &self.graph[*node_idx];
            info!("Router {}: recv={}, fwd={}, icmp={}",
                id.0,
                router.stats.packets_received,
                router.stats.packets_forwarded,
                router.stats.icmp_generated
            );
        }
    }
}
```

4. Call print_statistics at shutdown.

## Files to Modify
- `src/topology/router.rs` (add methods)
- `src/topology/fabric.rs` (add print method)
- `src/processor.rs` (update stats during processing)
- `src/lib.rs` or `src/tun/mod.rs` (call at shutdown)

## Effort Estimate
Small (2-3 hours)

## Related Plans
- Plan 6: Link Simulation (mentions statistics)
- Plan 9: Integration and End-to-End Testing
