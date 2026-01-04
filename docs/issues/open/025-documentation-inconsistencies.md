# Issue 025: Documentation Inconsistencies with Implementation

## Summary
Several documentation files reference features, configurations, or behaviors that don't match the current implementation.

## Locations
- `docs/configuration_schema.md`
- `docs/build_and_run_instructions.md`
- `examples/README.md`
- `README.md`

## Specific Inconsistencies

### 1. Configuration Schema vs Actual Config Structure
`docs/configuration_schema.md` describes:
```toml
[global]
tun_a = "tunA"
tun_b = "tunB"
ingress_a = "Rx0y0"
ingress_b = "Rx5y5"
```

But actual config uses:
```toml
[interfaces]
tun_a = "tunA"
tun_b = "tunB"

[tun_ingress]
tun_a_ingress = "Rx0y0"
tun_b_ingress = "Rx5y5"
```

### 2. Link Configuration Fields
Documentation mentions:
- `per_packet_lb` field

Actual implementation uses:
- `load_balance` field

### 3. Module Structure
`docs/project_layout.md` might describe the plan's module structure rather than actual implementation.

### 4. CLI Options
Documentation may not cover all implemented CLI options like `--packet-file`.

## Recommended Solution

1. Review each documentation file against actual implementation.

2. Update documentation to match current behavior:
   - Configuration field names
   - Module structure
   - CLI options
   - Feature status (implemented vs. stub)

3. Add a documentation consistency check to CI (optional).

4. Create a mapping of plan → implementation differences.

## Files to Modify
- `docs/configuration_schema.md`
- `docs/build_and_run_instructions.md`
- `docs/project_layout.md`
- `examples/README.md`
- `README.md`

## Effort Estimate
Medium (2-3 hours for comprehensive review)

## Related Documents
All documentation files
