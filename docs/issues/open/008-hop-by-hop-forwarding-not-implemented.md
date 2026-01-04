# Issue 008: Hop-by-Hop Forwarding Loop Not Implemented

## Summary
The packet processor only forwards a packet to a single next hop. It doesn't implement the full hop-by-hop forwarding through the entire fabric until the packet reaches its destination.

## Location
- File: `src/processor.rs`
- Functions: `process_packet()` and `process_packet_multi()`

## Current Behavior
The processor:
1. Selects an egress link from the ingress router
2. Simulates that single link
3. Logs "Packet forwarded" and returns

The packet is never actually forwarded to subsequent routers until it reaches the destination (ingress_b or ingress_a).

## Expected Behavior (from Plan 9)
A complete forwarding pipeline that:
1. Starts at ingress router
2. Decrements TTL at each hop
3. Looks up routing table at each router
4. Selects next hop (with multipath if enabled)
5. Simulates link characteristics (delay, jitter, loss)
6. Moves to the next router
7. Repeats until reaching destination ingress router
8. Delivers packet to appropriate TUN interface

## Recommended Solution

1. Implement a forwarding loop in `processor.rs`:
```rust
pub async fn process_packet(
    fabric: &Fabric,
    tables: &HashMap<RouterId, RoutingTable>,
    ingress: RouterId,
    mut packet: PacketMeta,
    destination: Destination,
) -> Option<PacketMeta> {
    let mut current_router = ingress;
    let target_router = match destination {
        Destination::TunA => &tables.get(&current_router)?.tun_a.next_hop,
        Destination::TunB => &tables.get(&current_router)?.tun_b.next_hop,
    };
    
    loop {
        // Check if we've reached destination
        if is_destination(&current_router, destination, tables) {
            return Some(packet);
        }
        
        // Decrement TTL
        if packet.ttl <= 1 {
            // Generate ICMP Time Exceeded
            return None;
        }
        packet.ttl -= 1;
        
        // Select next hop
        let routing_table = tables.get(&current_router)?;
        let next_hop = match destination {
            Destination::TunA => &routing_table.tun_a.next_hop,
            Destination::TunB => &routing_table.tun_b.next_hop,
        };
        
        // Get link to next hop and simulate it
        let link = find_link(fabric, &current_router, next_hop)?;
        if let Err(e) = simulation::simulate_link(link, &[]).await {
            // Packet dropped
            return None;
        }
        
        // Move to next router
        current_router = next_hop.clone();
    }
}
```

2. The function should return the processed packet for delivery to TUN.

3. Add integration tests for multi-hop forwarding.

## Files to Modify
- `src/processor.rs`
- `src/tun/mod.rs` (integrate with TUN delivery)
- `tests/` (add multi-hop forwarding tests)

## Effort Estimate
Medium (3-5 hours)

## Related Plans
- Plan 9: Integration and End-to-End Testing
