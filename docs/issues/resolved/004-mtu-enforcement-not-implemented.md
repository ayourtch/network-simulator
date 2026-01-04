# Issue 004: MTU Enforcement Not Implemented

## Summary
Link MTU is defined in the configuration but never enforced during packet forwarding. Packets larger than the link MTU should be dropped with an ICMP Fragmentation Needed error.

## Location
- File: `src/simulation/mod.rs`
- Function: `simulate_link()`

## Current Behavior
The `simulate_link()` function ignores the `_packet` parameter and doesn't check its size against the link MTU:
```rust
pub async fn simulate_link(link: &Link, _packet: &[u8]) -> Result<(), &'static str> {
    // Packet size vs MTU is never checked
}
```

## Expected Behavior (from Plan 6)
1. Check packet size against link MTU
2. If packet size > MTU and DF (Don't Fragment) bit is set, drop the packet
3. Generate ICMP Destination Unreachable (Type 3, Code 4 - Fragmentation Needed)
4. Include the link MTU in the ICMP error

## Recommended Solution

1. Add MTU check in `simulate_link()`:
```rust
pub async fn simulate_link(link: &Link, packet: &[u8]) -> Result<(), String> {
    // Get effective MTU (link-specific or default)
    let mtu = link.cfg.mtu.unwrap_or(1500) as usize;
    
    // Check packet size
    if packet.len() > mtu {
        debug!("Packet size {} exceeds MTU {}", packet.len(), mtu);
        return Err(format!("mtu_exceeded:{}", mtu));
    }
    
    // ... rest of existing logic
}
```

2. Handle the MTU error in `processor.rs` to generate ICMP error.

3. Add test cases for MTU enforcement.

## Files to Modify
- `src/simulation/mod.rs`
- `src/processor.rs` (handle MTU error)
- `tests/simulation_test.rs` (add MTU tests)

## Effort Estimate
Small (1-2 hours)

## Dependencies
- Issue 006: ICMP Fragmentation Needed generation

## Related Plans
- Plan 6: Link Simulation (MTU, Delay, Jitter, Loss)
- Plan 7: ICMP Error Generation
