# Issue 001: IPv6 Packet Parsing Not Implemented

## Summary
The packet parser in `src/packet/mod.rs` only supports IPv4 packet parsing. IPv6 parsing is explicitly not implemented despite the plan requiring dual-stack support.

## Location
- File: `src/packet/mod.rs`
- Function: `parse()`

## Current Behavior
```rust
if version != 4 {
    return Err("only IPv4 parsing supported in stub");
}
```

The parser rejects any packet with version != 4.

## Expected Behavior (from Plan 4)
- Parse both IPv4 and IPv6 packets
- Extract source/destination addresses for both protocols
- Extract protocol/next-header field
- Extract TTL/hop-limit field

## Recommended Solution

1. Add IPv6 parsing logic after the IPv4 check:
```rust
if version == 6 {
    // IPv6 header is 40 bytes minimum
    if data.len() < 40 {
        return Err("packet too short for IPv6 header");
    }
    let next_header = data[6];
    let hop_limit = data[7];
    // Source address: bytes 8-23
    // Destination address: bytes 24-39
    let src_ip = Ipv6Addr::from([
        data[8], data[9], data[10], data[11],
        data[12], data[13], data[14], data[15],
        data[16], data[17], data[18], data[19],
        data[20], data[21], data[22], data[23],
    ]);
    let dst_ip = Ipv6Addr::from([
        data[24], data[25], data[26], data[27],
        data[28], data[29], data[30], data[31],
        data[32], data[33], data[34], data[35],
        data[36], data[37], data[38], data[39],
    ]);
    return Ok(PacketMeta {
        src_ip: IpAddr::V6(src_ip),
        dst_ip: IpAddr::V6(dst_ip),
        src_port: 0,
        dst_port: 0,
        protocol: next_header,
        ttl: hop_limit,
        customer_id: 0,
    });
}
```

2. Add `use std::net::Ipv6Addr;` to imports.

3. Add a unit test for IPv6 parsing in `tests/packet_test.rs`.

## Files to Modify
- `src/packet/mod.rs`
- `tests/packet_test.rs` (add IPv6 test)

## Effort Estimate
Small (1-2 hours)

## Related Plans
- Plan 4: Packet Processing and Forwarding Engine
