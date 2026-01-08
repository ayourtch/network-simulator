# Issue 101: Routers Lack IP Addresses for ICMP Error Generation

## Summary
When generating ICMP error messages (Time Exceeded, Fragmentation Needed, Destination Unreachable), the simulator uses the destination IP address of the original packet as the source address of the ICMP error. This is incorrect - ICMP errors should be sourced from an IP address belonging to the router that generates the error.

## Priority
**Critical** - This affects all ICMP error generation and makes the simulator unusable for realistic network testing.

## Location
- File: `src/icmp/mod.rs`
- Lines: 49-52 (ICMPv6), 117-121 (ICMPv4)
- Also: `src/topology/router.rs` - Router struct lacks IP address fields

## Current Behavior
```rust
// IPv4 (line 117-121)
let src_ip = match original.dst_ip {
    std::net::IpAddr::V4(a) => a.octets(),
    _ => [0, 0, 0, 0],
};

// IPv6 (line 49-52)
let src = match packet.dst_ip {
    std::net::IpAddr::V6(a) => a,
    _ => Ipv6Addr::UNSPECIFIED,
};
```

The code uses `original.dst_ip` (the intended destination of the packet) as the ICMP source, not the router's own address.

## Expected Behavior
- Each router should have its own IP address(es)
- ICMP errors should use the router's address as the source IP
- The destination IP of the original packet is the remote host, not the router

## Impact
- All ICMP Time Exceeded messages have wrong source addresses (traceroute won't work correctly)
- All ICMP Fragmentation Needed messages have wrong source addresses
- Network diagnostic tools will show incorrect hop information
- The simulator cannot be used for realistic Path MTU Discovery testing

## Suggested Implementation

1. Add IP address fields to the Router struct:
```rust
// src/topology/router.rs
pub struct Router {
    pub id: RouterId,
    pub ipv4_addr: Option<Ipv4Addr>,  // e.g., 10.x.y.1 based on grid position
    pub ipv6_addr: Option<Ipv6Addr>,  // e.g., fd00::x:y based on grid position
    pub routing: crate::routing::RoutingTable,
    pub stats: RouterStats,
}
```

2. Generate deterministic addresses from RouterId:
```rust
impl Router {
    pub fn generate_addresses(id: &RouterId) -> (Ipv4Addr, Ipv6Addr) {
        // Parse Rx{x}y{y} to extract x and y
        let x = id.0.chars().nth(2).unwrap().to_digit(10).unwrap() as u8;
        let y = id.0.chars().nth(4).unwrap().to_digit(10).unwrap() as u8;
        
        let ipv4 = Ipv4Addr::new(10, 100 + x, y, 1);
        let ipv6 = Ipv6Addr::new(0xfd00, 0, 0, 0, 0, 0, x as u16, y as u16);
        (ipv4, ipv6)
    }
}
```

3. Update ICMP generation functions to accept router address:
```rust
pub fn generate_icmp_error(
    original: &PacketMeta, 
    error_type: u8, 
    code: u8,
    router_addr: Ipv4Addr,  // New parameter
) -> Vec<u8>
```

4. Update processor.rs to pass the router's address when generating ICMP.

## Resolution
**Resolved: 2026-01-08**

- Added `ipv4_addr` and `ipv6_addr` fields to `Router` struct
- Implemented `Router::new(id)` constructor with auto-generated addresses
- Addresses are deterministic based on grid position:
  - IPv4: `10.{100+x}.{y}.1` (e.g., Rx2y3 -> 10.102.3.1)
  - IPv6: `fd00::{x}:{y}` (e.g., Rx2y3 -> fd00::2:3)
- Updated ICMP functions to accept router address parameter
- Updated processor.rs to pass router addresses to all ICMP generation calls

---
*Created: 2026-01-08*
