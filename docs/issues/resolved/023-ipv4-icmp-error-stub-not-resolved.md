# Issue 023: IPv4 ICMP Error Generation Still a Stub (Ref: Issues 005/006)

## Summary
The resolved issues 005 (ICMP Time Exceeded) and 006 (ICMP Fragmentation Needed) claimed to be fixed, but the IPv4 ICMP error generation function `generate_icmp_error()` in `src/icmp/mod.rs` is still a stub that returns only 8 bytes - not a valid ICMP packet.

## Location
- File: `src/icmp/mod.rs`
- Function: `generate_icmp_error()` (lines 99-105)

## Current Behavior
```rust
pub fn generate_icmp_error(_original: &PacketMeta, error_type: u8, code: u8) -> Vec<u8> {
    debug!("Generating ICMP error type {} code {}", error_type, code);
    let mut payload = vec![0u8; 8];
    payload[0] = error_type;
    payload[1] = code;
    payload
}
```

This returns only 8 bytes which is:
1. Missing the IP header (20 bytes minimum)
2. Missing the ICMP checksum
3. Missing the original packet data (IP header + 8 bytes)

## Expected Behavior (from Plan 7 and RFC 792/RFC 4443)
A valid ICMP error packet should contain:
1. **IPv4 header** (20 bytes):
   - Source: Router's IP address
   - Destination: Original packet's source IP
   - Protocol: 1 (ICMP)
   - TTL: 64 (typical)
2. **ICMP header** (8 bytes):
   - Type: 11 (Time Exceeded) or 3 (Dest Unreachable)
   - Code: 0 (TTL exceeded in transit) or 4 (Fragmentation needed)
   - Checksum: Properly calculated
   - 4 bytes unused (or MTU for fragmentation needed)
3. **Original packet data**:
   - Original IP header + first 8 bytes of payload

## Recommended Solution

1. Replace the stub with a full implementation:
```rust
// Constants for ICMP packet structure
const IPV4_HEADER_LEN: usize = 20;
const ICMP_HEADER_LEN: usize = 8;
const ORIGINAL_PACKET_INCLUDE_LEN: usize = 28;  // IP header (20) + 8 bytes of payload
const MIN_ICMP_ERROR_PACKET_SIZE: usize = IPV4_HEADER_LEN + ICMP_HEADER_LEN + ORIGINAL_PACKET_INCLUDE_LEN;

/// Generate a proper ICMP error packet for IPv4.
pub fn generate_icmp_error(original: &PacketMeta, error_type: u8, code: u8) -> Vec<u8> {
    debug!("Generating ICMP error type {} code {}", error_type, code);
    
    let mut packet = Vec::with_capacity(MIN_ICMP_ERROR_PACKET_SIZE);
    
    // Build IPv4 header
    packet.push(0x45); // Version 4, IHL 5
    packet.push(0x00); // TOS
    // Total length placeholder (will be set later)
    packet.extend_from_slice(&[0, 0]);
    packet.extend_from_slice(&[0, 0]); // Identification
    packet.extend_from_slice(&[0, 0]); // Flags + Fragment Offset
    packet.push(64); // TTL = 64
    packet.push(1);  // Protocol = ICMP
    // Header checksum placeholder
    packet.extend_from_slice(&[0, 0]);
    
    // Source address = destination of original packet (router)
    let src = match original.dst_ip {
        std::net::IpAddr::V4(a) => a.octets(),
        _ => [0, 0, 0, 0],
    };
    packet.extend_from_slice(&src);
    
    // Destination address = source of original packet
    let dst = match original.src_ip {
        std::net::IpAddr::V4(a) => a.octets(),
        _ => [0, 0, 0, 0],
    };
    packet.extend_from_slice(&dst);
    
    // Build ICMP header
    packet.push(error_type);
    packet.push(code);
    // Checksum placeholder
    packet.extend_from_slice(&[0, 0]);
    // Unused (4 bytes)
    packet.extend_from_slice(&[0, 0, 0, 0]);
    
    // Include original IP header + first 8 bytes of payload
    let copy_len = original.raw.len().min(28);
    packet.extend_from_slice(&original.raw[..copy_len]);
    
    // Set total length
    let total_len = packet.len() as u16;
    packet[2] = (total_len >> 8) as u8;
    packet[3] = (total_len & 0xFF) as u8;
    
    // Calculate and set IP header checksum
    crate::packet::update_ipv4_checksum(&mut packet[..20]);
    
    // Calculate and set ICMP checksum
    let icmp_start = 20;
    let icmp_checksum = calculate_icmp_checksum(&packet[icmp_start..]);
    packet[icmp_start + 2] = (icmp_checksum >> 8) as u8;
    packet[icmp_start + 3] = (icmp_checksum & 0xFF) as u8;
    
    packet
}

fn calculate_icmp_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    for i in (0..data.len()).step_by(2) {
        let word = if i + 1 < data.len() {
            u16::from_be_bytes([data[i], data[i + 1]])
        } else {
            u16::from_be_bytes([data[i], 0])
        };
        sum += word as u32;
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}
```

2. Add a separate function for Fragmentation Needed that includes the MTU:
```rust
pub fn generate_fragmentation_needed(original: &PacketMeta, mtu: u16) -> Vec<u8> {
    // Similar to above, but set:
    // - error_type = 3 (Destination Unreachable)
    // - code = 4 (Fragmentation needed)
    // - bytes 6-7 of ICMP header = Next-Hop MTU
}
```

3. Add unit tests to verify ICMP packet generation.

## Files to Modify
- `src/icmp/mod.rs`
- `tests/` (add ICMP IPv4 generation tests)

## Effort Estimate
Medium (2-4 hours)

## References
- Original Issue 005: docs/issues/resolved/005-icmp-time-exceeded-stub.md
- Original Issue 006: docs/issues/resolved/006-icmp-fragmentation-needed-stub.md
- Plan 7: ICMP Error Generation

## Related Plans
- Plan 7: ICMP Error Generation
