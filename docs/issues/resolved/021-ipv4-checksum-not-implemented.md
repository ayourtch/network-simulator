# Issue 021: IPv4 Header Checksum Recalculation Not Implemented

## Summary
When TTL is decremented in an IPv4 packet (Issue 002), the IP header checksum must be recalculated. Currently, no checksum calculation code exists.

## Location
- File: `src/packet/mod.rs` (or new file)

## Current Behavior
No checksum calculation or validation code exists. The current stub parser doesn't preserve raw bytes, so there's nowhere to update the checksum.

## Expected Behavior (from Plan 4)
From the plan:
> TTL/hop limit decrement and validation
> (For IPv4, header checksum update required after TTL change)

## Recommended Solution

1. Add checksum calculation function:
```rust
/// Calculate IPv4 header checksum
pub fn calculate_ipv4_checksum(header: &[u8]) -> u16 {
    // Header length in bytes (IHL * 4)
    let header_len = ((header[0] & 0x0F) * 4) as usize;
    let header = &header[..header_len.min(header.len())];
    
    let mut sum: u32 = 0;
    
    for i in (0..header.len()).step_by(2) {
        // Skip the checksum field (bytes 10-11)
        if i == 10 {
            continue;
        }
        
        let word = if i + 1 < header.len() {
            u16::from_be_bytes([header[i], header[i + 1]])
        } else {
            u16::from_be_bytes([header[i], 0])
        };
        sum += word as u32;
    }
    
    // Add carries
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    
    !sum as u16
}
```

2. Add function to update checksum in packet:
```rust
pub fn update_ipv4_checksum(packet: &mut [u8]) {
    if packet.len() < 20 {
        return;
    }
    
    // Zero out checksum field
    packet[10] = 0;
    packet[11] = 0;
    
    // Calculate new checksum
    let checksum = calculate_ipv4_checksum(packet);
    packet[10] = (checksum >> 8) as u8;
    packet[11] = (checksum & 0xFF) as u8;
}
```

3. Call after decrementing TTL:
```rust
// Decrement TTL
packet[8] -= 1;
// Update checksum
update_ipv4_checksum(packet);
```

4. Add unit tests verifying checksum calculation.

## Files to Modify
- `src/packet/mod.rs` (add checksum functions)
- `tests/packet_test.rs` (add checksum tests)

## Effort Estimate
Small (1-2 hours)

## Dependencies
- Issue 020: Raw packet bytes preservation (need mutable raw bytes)

## Related Plans
- Plan 4: Packet Processing and Forwarding Engine
