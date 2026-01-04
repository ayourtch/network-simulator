# Issue 023: Bandwidth Limiting Not Implemented

## Summary
The `LinkConfig` in the configuration supports a `bandwidth_mbps` field, but bandwidth limiting is not implemented in the simulation.

## Location
- File: `src/topology/link.rs`, `src/simulation/mod.rs`
- Config: `config.toml`

## Current Behavior
The config file shows:
```toml
Rx0y0_Rx5y5 = { delay_ms = 10, jitter_ms = 0, loss_percent = 0, mtu = 1500, bandwidth_mbps = 100 }
```

But `LinkConfig` doesn't include `bandwidth_mbps` and the simulation doesn't implement bandwidth limiting.

## Expected Behavior
While bandwidth limiting is not explicitly mentioned in the plans, if the configuration supports it, the implementation should either:
1. Implement bandwidth limiting
2. Remove the field from example configs
3. Document it as not implemented

## Recommended Solution

### Option A: Implement bandwidth limiting (Enhancement)
```rust
// In LinkConfig:
pub bandwidth_mbps: Option<f64>,

// In simulate_link():
if let Some(bw) = link.cfg.bandwidth_mbps {
    // Calculate transmission delay based on packet size and bandwidth
    let packet_bits = (packet_size * 8) as f64;
    let transmission_delay_ms = packet_bits / (bw * 1_000_000.0) * 1000.0;
    sleep(Duration::from_micros((transmission_delay_ms * 1000.0) as u64)).await;
}
```

### Option B: Remove from example config (Simplification)
Remove `bandwidth_mbps` from `config.toml` example since it's not implemented.

### Recommended Approach
**Option B** - Document as not implemented and remove from example to avoid confusion.

## Files to Modify
For Option B:
- `config.toml` (remove bandwidth_mbps)
- Create issue doc noting this is a future enhancement

For Option A:
- `src/topology/link.rs`
- `src/simulation/mod.rs`
- `tests/simulation_test.rs`

## Effort Estimate
Small for Option B (< 1 hour)
Medium for Option A (2-3 hours)

## Related Plans
- Plan 6: Link Simulation (mentions delay, jitter, loss, MTU - not bandwidth)
