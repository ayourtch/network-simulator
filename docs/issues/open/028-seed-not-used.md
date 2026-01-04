# Issue 028: Seed for Reproducible Random Behavior Not Used

## Summary
The configuration includes a `seed` field for reproducible random behavior, but it's never used in the simulation.

## Location
- File: `src/config.rs`, `src/simulation/mod.rs`

## Current Behavior
In config:
```rust
pub struct SimulationConfig {
    // ...
    pub seed: Option<u64>,
}
```

In simulation:
```rust
let mut rng = rand::thread_rng(); // Ignores seed
```

The seed is parsed but never applied.

## Expected Behavior
When a seed is specified:
1. All random number generators should be seeded with the same value
2. This allows reproducible simulation runs for debugging
3. Same seed + same input = same output

## Recommended Solution

1. Create a global RNG or pass seeded RNG to functions:
```rust
use rand::SeedableRng;
use rand::rngs::StdRng;

pub fn create_rng(seed: Option<u64>) -> StdRng {
    match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    }
}
```

2. Pass RNG to simulation functions:
```rust
pub async fn simulate_link(link: &Link, _packet: &[u8], rng: &mut impl Rng) -> Result<(), &'static str> {
    if rng.gen_range(0.0..100.0) < link.cfg.loss_percent as f64 {
        return Err("packet lost");
    }
    // ...
}
```

3. Initialize RNG at startup:
```rust
let rng = create_rng(cfg.simulation.seed);
```

4. For testing, always use a known seed to get reproducible results.

## Files to Modify
- `src/simulation/mod.rs`
- `src/lib.rs` (initialize RNG)
- `src/processor.rs` (pass RNG)
- `tests/` (use seeded RNG for reproducible tests)

## Effort Estimate
Small (1-2 hours)

## Related Plans
- Plan 6: Link Simulation (mentions random for jitter and loss)
