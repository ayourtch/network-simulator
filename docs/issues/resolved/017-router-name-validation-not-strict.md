# Issue 017: Router Name Validation Not Strict

## Summary
Plan 1 specifies that router names must match the pattern `Rx[0-5]y[0-5]` for a 6x6 grid. The current implementation uses `Rx\d+y\d+` which allows any digits, not just 0-5.

## Location
- File: `src/topology/router.rs`
- Function: `RouterId::validate()`

## Current Behavior
```rust
impl RouterId {
    pub fn validate(&self) -> Result<(), String> {
        let re = regex::Regex::new(r"^Rx\d+y\d+$").unwrap();
        // ...
    }
}
```

This allows `Rx10y20`, `Rx99y99`, etc., which are outside the valid 6x6 grid.

## Expected Behavior (from Plan 1 and docs)
From `docs/configuration_schema.md`:
> Routers are identified by a router ID of the form `Rx{X}y{Y}` where `X` and `Y` are integers in the range `0..=5` for the default 6×6 fabric.

## Recommended Solution

1. Update the regex to enforce 0-5:
```rust
impl RouterId {
    pub fn validate(&self) -> Result<(), String> {
        let re = regex::Regex::new(r"^Rx[0-5]y[0-5]$").unwrap();
        if re.is_match(&self.0) {
            Ok(())
        } else {
            Err(format!(
                "Invalid router id '{}', expected Rx[0-5]y[0-5]", 
                self.0
            ))
        }
    }
}
```

2. Add test cases:
```rust
#[test]
fn test_valid_router_ids() {
    assert!(RouterId("Rx0y0".to_string()).validate().is_ok());
    assert!(RouterId("Rx5y5".to_string()).validate().is_ok());
    assert!(RouterId("Rx3y2".to_string()).validate().is_ok());
}

#[test]
fn test_invalid_router_ids() {
    assert!(RouterId("Rx6y0".to_string()).validate().is_err());
    assert!(RouterId("Rx0y6".to_string()).validate().is_err());
    assert!(RouterId("Rx10y5".to_string()).validate().is_err());
}
```

## Files to Modify
- `src/topology/router.rs`
- `tests/` (add router validation tests)

## Effort Estimate
Small (< 1 hour)

## Related Plans
- Plan 1: Project Setup and Configuration Parsing
- Plan 2: Core Data Structures and Router Model
