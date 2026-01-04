# Issue 006: ICMP Fragmentation Needed Generation Not Implemented

## Summary
There is no implementation for generating ICMP Destination Unreachable (Fragmentation Needed) packets. This is required when a packet exceeds the link MTU.

## Location
- File: `src/icmp/mod.rs`

## Current Behavior
Only a generic stub `generate_icmp_error()` exists which returns an empty vector. No specific implementation for Fragmentation Needed (Type 3, Code 4).

## Expected Behavior (from Plan 7)
Generate ICMP Destination Unreachable packet with:
1. Type: 3 (Destination Unreachable)
2. Code: 4 (Fragmentation needed and DF set)
3. Bytes 4-5: Unused (zeros)
4. Bytes 6-7: Next-Hop MTU (the MTU that would allow the packet through)
5. Original IP header + first 8 bytes of original payload

## Recommended Solution

1. Add a new function for Fragmentation Needed:
```rust
pub fn generate_fragmentation_needed(
    original: &PacketMeta, 
    router_ip: std::net::IpAddr, 
    original_bytes: &[u8],
    mtu: u32
) -> Vec<u8> {
    let mut packet = Vec::with_capacity(56);
    
    // Build IP header
    // ... similar to Time Exceeded
    
    // Build ICMP header
    packet.push(3);  // Type: Destination Unreachable
    packet.push(4);  // Code: Fragmentation needed
    packet.extend_from_slice(&[0, 0]); // Checksum placeholder
    packet.extend_from_slice(&[0, 0]); // Unused
    packet.extend_from_slice(&(mtu as u16).to_be_bytes()); // Next-Hop MTU
    
    // Include original IP header + 8 bytes
    let data_len = original_bytes.len().min(28);
    packet.extend_from_slice(&original_bytes[..data_len]);
    
    // Calculate checksums
    // ...
    
    packet
}
```

2. Integrate with the MTU check in `simulation/mod.rs` (see Issue 004).

3. Add unit tests.

## Files to Modify
- `src/icmp/mod.rs`
- `tests/` (add ICMP tests)

## Effort Estimate
Medium (2-4 hours)

## Dependencies
- Issue 004: MTU enforcement (provides the trigger for this ICMP)

## Related Plans
- Plan 7: ICMP Error Generation
