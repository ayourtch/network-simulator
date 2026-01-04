# Issue 018: Ingress Router Validation Missing

## Summary
The configuration allows specifying ingress routers (`tun_a_ingress` and `tun_b_ingress`) but there's no validation that these routers actually exist in the topology.

## Location
- File: `src/config.rs`, `src/lib.rs`

## Current Behavior
If `tun_a_ingress = "Rx9y9"` is specified but that router doesn't exist in the topology, the simulator will panic at runtime when trying to look up the router.

## Expected Behavior (from Plan 1)
From `docs/configuration_schema.md`:
> Validation Rules:
> 5. Ingress routers – `tun_ingress.tun_a_ingress` and `tun_ingress.tun_b_ingress` must reference valid router IDs.

## Recommended Solution

1. Add validation after config parsing:
```rust
impl SimulatorConfig {
    pub fn validate(&self) -> Result<(), String> {
        // Validate ingress routers are in the topology
        let router_names: HashSet<_> = self.topology.routers.keys().collect();
        
        if !router_names.contains(&self.tun_ingress.tun_a_ingress) {
            return Err(format!(
                "Ingress router '{}' not found in topology",
                self.tun_ingress.tun_a_ingress
            ));
        }
        
        if !router_names.contains(&self.tun_ingress.tun_b_ingress) {
            return Err(format!(
                "Ingress router '{}' not found in topology",
                self.tun_ingress.tun_b_ingress
            ));
        }
        
        // Also validate ingress router names match Rx[0-5]y[0-5] pattern
        RouterId(self.tun_ingress.tun_a_ingress.clone()).validate()?;
        RouterId(self.tun_ingress.tun_b_ingress.clone()).validate()?;
        
        Ok(())
    }
}
```

2. Call validation in `main.rs` after parsing.

3. Add test cases for missing ingress routers.

## Files to Modify
- `src/config.rs` (add/extend validate method)
- `src/main.rs` (call validation)
- `tests/` (add validation tests)

## Effort Estimate
Small (1 hour)

## Related Plans
- Plan 1: Project Setup and Configuration Parsing
