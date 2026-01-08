# Issue 103: ICMP Fragmentation Needed Missing Unused Bytes

## Summary
The ICMP Destination Unreachable - Fragmentation Needed (Type 3, Code 4) packet is missing 2 bytes of the required "unused" field, causing the MTU to be placed in the wrong location and the packet structure to be malformed.

## Priority
**High** - Causes malformed ICMP packets that won't be correctly interpreted by receivers.

## Location
- File: `src/icmp/mod.rs`
- Function: `generate_fragmentation_needed`
- Lines: 178-184

## Current Behavior

```rust
// ICMP header for Fragmentation Needed
packet.push(3); // Type
packet.push(4); // Code
packet.extend_from_slice(&[0, 0]); // Checksum placeholder
// Missing 2 unused bytes here!
let mtu16 = (mtu as u16).to_be_bytes();
packet.extend_from_slice(&mtu16);
```

Current packet structure (bytes 20-27 of IP packet):
```
Offset:  20   21   22   23   24   25   26   27
Content: Type Code Cksum[0] Cksum[1] MTU[0] MTU[1] OrigPkt...
```

## Expected Behavior

Per RFC 792, ICMP Destination Unreachable has this structure:
```
Offset:  20   21   22   23   24   25   26   27
Content: Type Code Cksum[0] Cksum[1] Unused Unused MTU[0] MTU[1]
```

The unused bytes (offset 24-25) must be zero, and the Next-Hop MTU is at offset 26-27.

## Impact
- ICMP Fragmentation Needed messages are malformed
- Path MTU Discovery will not work correctly
- Receiving hosts will misinterpret the MTU value
- The original packet data starts 2 bytes too early

## Suggested Implementation

Add the missing unused bytes:

```rust
// ICMP header for Fragmentation Needed
packet.push(3);   // Type: Destination Unreachable
packet.push(4);   // Code: Fragmentation needed and DF set
packet.extend_from_slice(&[0, 0]);  // Checksum placeholder
packet.extend_from_slice(&[0, 0]);  // Unused (2 bytes) - THIS IS MISSING
let mtu16 = (mtu as u16).to_be_bytes();
packet.extend_from_slice(&mtu16);   // Next-hop MTU (2 bytes)
```

Also update the test in `tests/icmp_fragmentation_test.rs` to verify the correct offset:
```rust
// MTU should be in bytes 6-7 of ICMP header (offset 26-27 in packet)
let mtu_in_packet = u16::from_be_bytes([
    packet[ip_header_len + 6],  // Was checking offset 24-26, should be 26-27
    packet[ip_header_len + 7],
]);
```

## Resolution
**Resolved: 2026-01-08**

- Added missing 2-byte unused field before MTU in `generate_fragmentation_needed`
- Packet structure now correctly follows RFC 792:
  - Type (1 byte) + Code (1 byte) + Checksum (2 bytes) + Unused (2 bytes) + MTU (2 bytes)

---
*Created: 2026-01-08*
