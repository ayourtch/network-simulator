# Issue 002: TTL/Hop-Limit Decrement Not Implemented

## Summary
The plan specifies that TTL (IPv4) or Hop Limit (IPv6) should be decremented at each router hop. Currently, the `PacketMeta` structure contains the TTL field but it is never modified during packet processing.

## Location
- File: `src/processor.rs`
- Function: `process_packet()` and `process_packet_multi()`

## Current Behavior
The `packet.ttl` field is parsed from the packet but never decremented when the packet traverses a router hop. No TTL-expired check is performed.

## Expected Behavior (from Plan 4)
1. Decrement TTL at each hop
2. Check if TTL reaches 0 or 1
3. If TTL expired, generate ICMP Time Exceeded error
4. Drop the original packet when TTL expires

## Recommended Solution

1. Add a TTL decrement check in `process_packet()`:
```rust
// Check TTL before processing
let ttl = packet.ttl;
if ttl <= 1 {
    // TTL expired - generate ICMP error
    debug!("TTL expired at router {}", ingress.0);
    // Call icmp::generate_icmp_error() with Type 11 Code 0
    return;
}

// Decrement TTL for next hop
let mut packet = packet;
packet.ttl = ttl - 1;
```

2. The ICMP generation (Issue 005) should be implemented first or concurrently.

3. Add tests for TTL decrement behavior.

## Files to Modify
- `src/processor.rs`
- `tests/` (add TTL decrement tests)

## Effort Estimate
Small (1-2 hours)

## Dependencies
- Issue 005: ICMP Time Exceeded generation

## Related Plans
- Plan 4: Packet Processing and Forwarding Engine
- Plan 7: ICMP Error Generation
