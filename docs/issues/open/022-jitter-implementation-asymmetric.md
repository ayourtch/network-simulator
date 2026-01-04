# Issue 022: Jitter Implementation Could Be Improved

## Summary
The current jitter implementation adds only positive jitter (0 to jitter_ms). Standard network simulation typically uses symmetric jitter (±jitter around the base delay).

## Location
- File: `src/simulation/mod.rs`
- Function: `simulate_link()`

## Current Behavior
```rust
let jitter = if link.cfg.jitter_ms > 0 {
    rng.gen_range(0..=link.cfg.jitter_ms)
} else {
    0
};
let total_delay = link.cfg.delay_ms + jitter;
```

Jitter is always non-negative, added on top of base delay.

## Expected Behavior (from Plan 6)
From the plan:
> Jitter simulation (random delay variation)
> Jitter is uniformly distributed: delay ± jitter

This suggests symmetric jitter: the actual delay should be `base_delay ± jitter`, not `base_delay + (0..jitter)`.

## Recommended Solution

Change jitter calculation to be symmetric:
```rust
let jitter_offset = if link.cfg.jitter_ms > 0 {
    // Symmetric jitter: -jitter to +jitter
    rng.gen_range(-(link.cfg.jitter_ms as i32)..=(link.cfg.jitter_ms as i32))
} else {
    0
};

let total_delay = ((link.cfg.delay_ms as i32) + jitter_offset).max(0) as u32;
```

Or equivalently with floating point:
```rust
let jitter_offset = if link.cfg.jitter_ms > 0 {
    rng.gen_range(-(link.cfg.jitter_ms as f64)..=(link.cfg.jitter_ms as f64))
} else {
    0.0
};

let total_delay = ((link.cfg.delay_ms as f64) + jitter_offset).max(0.0) as u64;
```

Note: The `.max(0)` ensures delay never goes negative.

## Files to Modify
- `src/simulation/mod.rs`
- `tests/simulation_test.rs` (verify symmetric jitter)

## Effort Estimate
Small (< 1 hour)

## Related Plans
- Plan 6: Link Simulation (MTU, Delay, Jitter, Loss)
