# Issue 006: ICMP Fragmentation Needed Generation Not Implemented

## Summary
The simulator does not generate proper ICMP Destination Unreachable (Fragmentation Needed) packets when a packet exceeds the MTU of a link. The current implementation calls `icmp::generate_icmp_error(&packet, 3, 4)`, which creates a generic ICMP error but does **not** include the required Next‑Hop MTU field (bytes 6‑7) as specified in RFC 792.

## Location
- File: `src/icmp/mod.rs`
- Function: `generate_icmp_error()`
- Call site: `src/processor.rs` (MTU exceed handling)

## Expected Behavior
When `SimulationError::MtuExceeded { packet_size, mtu }` is returned, the simulator should:
1. Call a dedicated function (e.g., `generate_fragmentation_needed`) that builds an ICMP Destination Unreachable packet with:
   - Type 3, Code 4
   - Unused bytes set to zero
   - Next‑Hop MTU field set to the link MTU (big‑endian)
   - Original IP header + first 8 bytes of payload copied into the data section
2. Compute correct ICMP checksum and IPv4 header checksum.
3. Forward the generated ICMP packet back through the fabric.

## Recommended Solution (low‑skill developer)
1. Add a new helper in `src/icmp/mod.rs`:
```rust
pub fn generate_fragmentation_needed(original: &PacketMeta, mtu: u32) -> Vec<u8> {
    // Build IPv4 header (same as generate_icmp_error but set MTU field)
    // ... copy original code and insert mtu as u16
}
```
2. In `src/processor.rs` where the MTU error is handled, replace the generic call with the new helper, passing `mtu` from the error.
3. Add a unit test in `tests/icmp_fragmentation_test.rs` that verifies the MTU field is present and correct.

## Files to Modify
- `src/icmp/mod.rs` (add function and adjust `generate_icmp_error` if needed)
- `src/processor.rs` (use the new function when handling `SimulationError::MtuExceeded`)
- `tests/icmp_fragmentation_test.rs` (new test file)

*This issue is opened because the original resolved issue 006 states the feature is still missing.*