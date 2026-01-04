# Issue 003: TCP/UDP Port Extraction Not Implemented

## Summary
The packet parser sets `src_port` and `dst_port` to 0 for all packets. The plan specifies that ports should be extracted from TCP and UDP headers for 5-tuple hashing in multipath routing.

## Location
- File: `src/packet/mod.rs`
- Function: `parse()`

## Current Behavior
```rust
Ok(PacketMeta {
    // ...
    src_port: 0,
    dst_port: 0,
    // ...
})
```

Ports are always set to 0 regardless of protocol.

## Expected Behavior (from Plan 8)
- For TCP packets (protocol 6): Extract ports from TCP header (bytes 0-3 after IP header)
- For UDP packets (protocol 17): Extract ports from UDP header (bytes 0-3 after IP header)
- For other protocols (ICMP, etc.): Ports can remain 0

## Recommended Solution

1. Calculate IP header length to find transport header offset:
```rust
let ihl = (data[0] & 0x0F) as usize * 4;
```

2. Check protocol and extract ports if applicable:
```rust
let (src_port, dst_port) = if protocol == 6 || protocol == 17 {
    // TCP or UDP - ports are in first 4 bytes of transport header
    if data.len() >= ihl + 4 {
        let sp = u16::from_be_bytes([data[ihl], data[ihl + 1]]);
        let dp = u16::from_be_bytes([data[ihl + 2], data[ihl + 3]]);
        (sp, dp)
    } else {
        (0, 0) // Transport header truncated
    }
} else {
    (0, 0) // Non-TCP/UDP protocol
};
```

3. Add tests with TCP and UDP packets.

## Files to Modify
- `src/packet/mod.rs`
- `tests/packet_test.rs` (add TCP/UDP port extraction tests)

## Effort Estimate
Small (1-2 hours)

## Related Plans
- Plan 4: Packet Processing and Forwarding Engine
- Plan 8: Multi-path Routing and Load Balancing
