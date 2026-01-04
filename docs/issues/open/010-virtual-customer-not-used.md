# Issue 010: VirtualCustomer Feature Not Used

## Summary
The `VirtualCustomer` concept is mentioned in the plans and the `customer_id` field exists in `PacketMeta`, but it's never assigned a meaningful value or used in packet processing.

## Location
- Files: `src/packet/mod.rs`, `src/processor.rs`
- Struct: `PacketMeta`

## Current Behavior
```rust
// In parse():
Ok(PacketMeta {
    // ...
    customer_id: 0, // Always hardcoded to 0
})
```

The `customer_id` is never:
- Assigned based on source TUN or other criteria
- Used for any isolation or differentiation
- Logged or tracked

## Expected Behavior (from Plan 2 and docs/core_data_structures.md)
From `core_data_structures.md`:
> Virtual customer number – assigned when the packet first enters the simulator.

The feature was intended to support:
1. Multiple virtual customers with isolated topologies (future enhancement)
2. Per-customer statistics
3. Customer-based routing decisions

## Recommended Solution

Since this is marked as a "Future enhancement (NOT in v1)" in the Master Plan, there are two options:

### Option A: Remove the unused field (Simplification)
Remove `customer_id` from `PacketMeta` to reduce confusion:
```rust
pub struct PacketMeta {
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8,
    pub ttl: u8,
    // customer_id removed
}
```

### Option B: Implement basic customer assignment (Enhancement)
1. Add a configuration option for customer assignment rules
2. Assign customer ID based on source TUN interface:
```rust
// Packets from TUN A get customer_id = 1
// Packets from TUN B get customer_id = 2
let customer_id = if source_tun == "tunA" { 1 } else { 2 };
```
3. Include in logging and statistics

## Recommended Approach
**Option A** - Remove the unused field for now. The Master Plan explicitly states this is "NOT in v1".

## Files to Modify
- `src/packet/mod.rs`
- `src/processor.rs` (if needed)
- `docs/core_data_structures.md` (update documentation)

## Effort Estimate
Small (1 hour)

## Related Plans
- Plan 2: Core Data Structures and Router Model
- Master Plan (Future Enhancements section)
