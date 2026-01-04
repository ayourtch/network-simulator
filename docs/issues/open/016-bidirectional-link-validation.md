# Issue 016: Bidirectional Link Validation Not Implemented

## Summary
Plan 1 specifies that configuration parsing should detect and reject duplicate bidirectional links (e.g., both `Rx0y0_Rx0y1` and `Rx0y1_Rx0y0`). This validation is not implemented in the current config parsing code.

## Location
- File: `src/config.rs`

## Current Behavior
The config parser does not validate for duplicate bidirectional links. If both `Rx0y0_Rx0y1` and `Rx0y1_Rx0y0` are defined in the config, both would be added to the links HashMap (overwriting each other since they share the same key after normalization in `LinkId::new`).

## Expected Behavior (from Plan 1)
```
Validation Rules:
2. Bidirectional consistency – if both A_B and B_A sections exist, all fields must match.
3. Unique links – duplicate link definitions (same unordered pair) are not allowed.
```

## Recommended Solution

1. Add validation in config parsing or fabric building:
```rust
impl SimulatorConfig {
    pub fn validate(&self) -> Result<(), String> {
        use std::collections::HashSet;
        let mut seen_links: HashSet<(String, String)> = HashSet::new();
        
        for link_name in self.topology.links.keys() {
            let parts: Vec<&str> = link_name.split('_').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid link name: {}", link_name));
            }
            
            // Normalize order
            let (a, b) = if parts[0] < parts[1] {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                (parts[1].to_string(), parts[0].to_string())
            };
            
            let key = (a, b);
            if seen_links.contains(&key) {
                return Err(format!(
                    "Duplicate bidirectional link: {} (already defined in opposite direction)",
                    link_name
                ));
            }
            seen_links.insert(key);
        }
        
        Ok(())
    }
}
```

2. Call validation after parsing config in `main.rs`:
```rust
let cfg: SimulatorConfig = toml::from_str(&cfg_str)?;
cfg.validate()?;
```

3. Add test case for duplicate link detection.

## Files to Modify
- `src/config.rs` (add validate method)
- `src/main.rs` (call validation)
- `tests/` (add validation tests)

## Effort Estimate
Small (1-2 hours)

## Related Plans
- Plan 1: Project Setup and Configuration Parsing
