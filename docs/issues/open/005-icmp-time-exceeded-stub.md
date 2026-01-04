# Issue 005: ICMP Time Exceeded Generation Is Stub

## Summary
The ICMP error generation function in `src/icmp/mod.rs` is a stub that returns an empty vector. It does not generate actual ICMP Time Exceeded packets as specified in Plan 7.

## Location
- File: `src/icmp/mod.rs`
- Function: `generate_icmp_error()`

## Current Behavior
```rust
pub fn generate_icmp_error(_original: &PacketMeta, _error_type: u8, _code: u8) -> Vec<u8> {
    debug!("Generating ICMP error type {} code {}", _error_type, _code);
    vec![] // placeholder
}
```

Returns an empty vector instead of a valid ICMP packet.

## Expected Behavior (from Plan 7)
Generate a valid ICMP Time Exceeded packet containing:
1. IP header with router's source address, original packet's source as destination
2. ICMP header: Type 11, Code 0
3. 4 bytes unused (zeros)
4. Original IP header + first 8 bytes of original payload

## Recommended Solution

1. Create a new function for Time Exceeded:
```rust
pub fn generate_time_exceeded(original: &PacketMeta, router_ip: std::net::IpAddr, original_bytes: &[u8]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(56); // IP(20) + ICMP(8) + Original(28)
    
    // Build IP header
    packet.push(0x45); // Version 4, IHL 5
    packet.push(0x00); // TOS
    // ... total length, ID, flags, TTL, protocol=1 (ICMP)
    // ... source = router_ip, dest = original.src_ip
    
    // Build ICMP header
    packet.push(11); // Type: Time Exceeded
    packet.push(0);  // Code: TTL exceeded in transit
    packet.extend_from_slice(&[0, 0]); // Checksum placeholder
    packet.extend_from_slice(&[0, 0, 0, 0]); // Unused
    
    // Include original IP header + 8 bytes of data
    let data_len = original_bytes.len().min(28);
    packet.extend_from_slice(&original_bytes[..data_len]);
    
    // Calculate and insert checksums
    // ...
    
    packet
}
```

2. Add helper functions for IP and ICMP checksum calculation.

3. Add unit tests for ICMP packet generation and validation.

## Files to Modify
- `src/icmp/mod.rs`
- `tests/` (add ICMP generation tests)

## Effort Estimate
Medium (2-4 hours)

## Related Plans
- Plan 7: ICMP Error Generation
