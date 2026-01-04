# Issue 030: Default 6x6 Fabric Not Auto-Generated

## Summary
The plans describe a 6x6 router fabric (36 routers), but the configuration requires manually listing each router. There's no option to auto-generate a full fabric.

## Location
- File: `src/config.rs`

## Current Behavior
Users must manually list every router in the configuration:
```toml
[topology.routers]
Rx0y0 = {}
Rx0y1 = {}
# ... list all 36 routers manually
```

## Expected Behavior
While explicit configuration is valid, there could be a convenience option to auto-generate the full 6x6 fabric:
```toml
[topology]
auto_generate_fabric = true
# Automatically creates Rx0y0 through Rx5y5
```

Or a CLI option:
```bash
--generate-fabric 6x6
```

## Recommended Solution

This is more of an enhancement than a bug. Options:

### Option A: Add auto-generation flag
```rust
// In config.rs
pub struct TopologyConfig {
    #[serde(default)]
    pub auto_generate: bool,
    // ...
}

// In lib.rs - run()
if cfg.topology.auto_generate {
    for x in 0..=5 {
        for y in 0..=5 {
            let id = RouterId(format!("Rx{}y{}", x, y));
            // Add router if not already present
        }
    }
}
```

### Option B: Provide complete example config
Create `examples/full_6x6_fabric.toml` with all 36 routers and common link patterns.

### Option C: CLI command to generate config
```bash
cargo run -- --generate-config 6x6 > full_config.toml
```

### Recommended Approach
**Option B** - Simpler and doesn't require code changes.

## Files to Create/Modify
For Option B:
- `examples/full_6x6_fabric.toml`

For Option A:
- `src/config.rs`
- `src/lib.rs`

## Effort Estimate
Small for Option B (< 1 hour)
Medium for Option A (2-3 hours)

## Related Plans
- Plan 2: Core Data Structures and Router Model
