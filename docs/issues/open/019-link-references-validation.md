# Issue 019: Link References Unknown Routers Not Fully Validated

## Summary
Links in the configuration reference routers that must exist in the routers section. While there's a check during fabric building, there's no upfront validation during config parsing, and the error message could be more helpful.

## Location
- File: `src/lib.rs`
- Function: `run()`

## Current Behavior
In `run()`:
```rust
if fabric.router_index.contains_key(&a) && fabric.router_index.contains_key(&b) {
    fabric.add_link(&a, &b, link_cfg.clone());
} else {
    error!("Link {} references unknown router(s)", link_name);
}
```

The error is logged but execution continues, potentially leaving the fabric in an incomplete state.

## Expected Behavior (from Plan 1)
From `docs/configuration_schema.md`:
> Validation Rules:
> 4. Link endpoint existence – both router IDs in a link definition must reference existing routers.

The validation should:
1. Check before building the fabric
2. Provide specific error about which router is missing
3. Fail fast rather than continue with missing links

## Recommended Solution

1. Add link validation to config validation:
```rust
impl SimulatorConfig {
    pub fn validate(&self) -> Result<(), String> {
        let router_names: HashSet<_> = self.topology.routers.keys()
            .map(|s| s.as_str())
            .collect();
        
        for link_name in self.topology.links.keys() {
            let parts: Vec<&str> = link_name.split('_').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid link name format: {}", link_name));
            }
            
            if !router_names.contains(parts[0]) {
                return Err(format!(
                    "Link '{}' references unknown router '{}'",
                    link_name, parts[0]
                ));
            }
            
            if !router_names.contains(parts[1]) {
                return Err(format!(
                    "Link '{}' references unknown router '{}'",
                    link_name, parts[1]
                ));
            }
        }
        
        Ok(())
    }
}
```

2. Call validation in main.rs before running.

3. Add test case for links with unknown routers.

## Files to Modify
- `src/config.rs` (add/extend validate method)
- `src/main.rs` (call validation)
- `tests/` (add validation tests)

## Effort Estimate
Small (1 hour)

## Related Plans
- Plan 1: Project Setup and Configuration Parsing
