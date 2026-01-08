# Issue 105: ICMPv6 Error Type Handling Inconsistent

## Summary
The ICMPv6 error generation function has inconsistent handling of the 4-byte unused/reserved field. It only adds this field for type 3 (Time Exceeded), but ICMPv6 Destination Unreachable (type 1) also requires a 4-byte unused field. Additionally, ICMPv6 Packet Too Big (type 2) requires a 4-byte MTU field that isn't handled.

## Priority
**Medium** - Affects correctness of some ICMPv6 messages.

## Location
- File: `src/icmp/mod.rs`
- Function: `generate_icmpv6_error`
- Lines: 64-67

## Current Behavior

```rust
if error_type == 3 {
    // Time Exceeded includes 4‑byte unused field
    buf.extend_from_slice(&[0, 0, 0, 0]);
}
```

Only type 3 (Time Exceeded) gets the 4-byte field. Other types are not handled:
- Type 1 (Destination Unreachable): Needs 4-byte unused field
- Type 2 (Packet Too Big): Needs 4-byte MTU field

## Expected Behavior

Per RFC 4443:

**Type 1 - Destination Unreachable:**
```
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|     Type      |     Code      |          Checksum             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                             Unused                            |  <- 4 bytes
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    As much of invoking packet                 |
```

**Type 2 - Packet Too Big:**
```
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|     Type      |     Code      |          Checksum             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                             MTU                               |  <- 4 bytes
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    As much of invoking packet                 |
```

**Type 3 - Time Exceeded:**
```
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|     Type      |     Code      |          Checksum             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                             Unused                            |  <- 4 bytes
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    As much of invoking packet                 |
```

## Impact
- ICMPv6 Destination Unreachable messages may be malformed
- ICMPv6 Packet Too Big messages don't include the MTU value
- IPv6 Path MTU Discovery won't work correctly

## Suggested Implementation

1. Add MTU parameter to the function signature for Packet Too Big:
```rust
pub fn generate_icmpv6_error(
    packet: &PacketMeta, 
    error_type: u8, 
    code: u8,
    mtu: Option<u32>,  // Only used for type 2
) -> Vec<u8>
```

2. Handle all types correctly:
```rust
match error_type {
    1 => {
        // Destination Unreachable: 4-byte unused field
        buf.extend_from_slice(&[0, 0, 0, 0]);
    }
    2 => {
        // Packet Too Big: 4-byte MTU field
        let mtu_val = mtu.unwrap_or(1280);
        buf.extend_from_slice(&mtu_val.to_be_bytes());
    }
    3 => {
        // Time Exceeded: 4-byte unused field
        buf.extend_from_slice(&[0, 0, 0, 0]);
    }
    _ => {
        // Unknown type: add 4-byte unused field as fallback
        buf.extend_from_slice(&[0, 0, 0, 0]);
    }
}
```

3. Update callers in `processor.rs` to pass MTU when generating Packet Too Big.

## Resolution
**Resolved: 2026-01-08**

- Added `mtu: Option<u32>` parameter to `generate_icmpv6_error` function
- Implemented proper match statement for all ICMPv6 error types:
  - Type 1 (Destination Unreachable): 4-byte unused field
  - Type 2 (Packet Too Big): 4-byte MTU field
  - Type 3 (Time Exceeded): 4-byte unused field
  - Default: 4-byte unused field as fallback
- Updated all callers to pass MTU when generating Packet Too Big errors

---
*Created: 2026-01-08*
