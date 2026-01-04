# Issue 007: ICMPv6 Support Not Implemented

## Summary
ICMPv6 error messages are not implemented. While Plan 7 specifies both ICMPv4 and ICMPv6 support, only a stub for IPv4 ICMP exists.

## Location
- File: `src/icmp/mod.rs`

## Current Behavior
The current ICMP module only has a generic stub function. No ICMPv6 support exists.

## Expected Behavior (from Plan 7)
For IPv6 packets, the simulator should generate appropriate ICMPv6 errors:
1. **Time Exceeded (Type 3, Code 0)**: When Hop Limit reaches 0
2. **Packet Too Big (Type 2, Code 0)**: When packet exceeds link MTU

ICMPv6 has a different format than ICMPv4:
- ICMPv6 checksum includes a pseudo-header with IPv6 addresses
- ICMPv6 Packet Too Big includes 4-byte MTU field

## Recommended Solution

1. Add ICMPv6 Time Exceeded function:
```rust
pub fn generate_icmpv6_time_exceeded(
    original: &PacketMeta,
    router_ip: std::net::Ipv6Addr,
    original_bytes: &[u8]
) -> Vec<u8> {
    let mut packet = Vec::new();
    
    // IPv6 header (40 bytes)
    // ...
    
    // ICMPv6 header
    packet.push(3);  // Type: Time Exceeded
    packet.push(0);  // Code: Hop Limit exceeded
    packet.extend_from_slice(&[0, 0]); // Checksum placeholder
    packet.extend_from_slice(&[0, 0, 0, 0]); // Unused
    
    // Original packet (as much as fits in MTU)
    // ICMPv6 should include as much of the original as possible
    // while keeping the whole packet <= 1280 bytes
    
    packet
}
```

2. Add ICMPv6 Packet Too Big function:
```rust
pub fn generate_icmpv6_packet_too_big(
    original: &PacketMeta,
    router_ip: std::net::Ipv6Addr,
    original_bytes: &[u8],
    mtu: u32
) -> Vec<u8> {
    // Similar structure with Type 2
}
```

3. Add ICMPv6 checksum calculation with pseudo-header.

4. Add unit tests.

## Files to Modify
- `src/icmp/mod.rs`
- `tests/` (add ICMPv6 tests)

## Effort Estimate
Medium (3-4 hours)

## Dependencies
- Issue 001: IPv6 packet parsing

## Related Plans
- Plan 7: ICMP Error Generation
