# Issue 026: Router Statistics Never Updated During Packet Processing (Ref: Issue 012)

## Summary
Issue 012 claimed to be resolved, noting that statistics methods exist in the Router struct. However, the `increment_received()` and `increment_forwarded()` methods are never called during actual packet processing, so statistics are never collected.

## Location
- File: `src/processor.rs`
- Functions: `process_packet()` and `process_packet_multi()`

## Current Behavior
The `Router` struct has these methods (in `src/topology/router.rs`):
```rust
impl Router {
    pub fn increment_received(&mut self) {
        self.stats.packets_received += 1;
    }
    pub fn increment_forwarded(&mut self) {
        self.stats.packets_forwarded += 1;
    }
    pub fn increment_icmp(&mut self) {
        self.stats.icmp_generated += 1;
    }
}
```

But in `process_packet()`:
- `increment_received()` is never called when a packet arrives at a router
- `increment_forwarded()` is never called when a packet is successfully forwarded
- Only `increment_icmp()` is called (line 71), but only for MTU exceeded errors

## Expected Behavior (from Plan 6 and docs)
Statistics should be updated at each step:
1. When a packet arrives at a router → `increment_received()`
2. When a packet is successfully forwarded to next hop → `increment_forwarded()`
3. When an ICMP error is generated → `increment_icmp()`

This enables:
- Monitoring packet flow through the fabric
- Debugging routing issues
- Performance analysis
- Verification that packets are traversing expected routers

## Recommended Solution

1. Add statistics updates in the processing loop:
```rust
pub async fn process_packet(
    fabric: &mut Fabric,
    tables: &HashMap<RouterId, RoutingTable>,
    mut ingress: RouterId,
    mut packet: PacketMeta,
    mut destination: Destination,
) -> PacketMeta {
    loop {
        // Increment received counter for current router
        if let Some(node_idx) = fabric.router_index.get(&ingress) {
            if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                router.increment_received();
            }
        }
        
        // ... TTL check and decrement ...
        
        // Get routing table and next hop
        let table = match tables.get(&ingress) { ... };
        let next_hop = match destination { ... };
        
        // Simulate the link
        if let Some(link) = fabric.get_link(&ingress, next_hop) {
            if let Err(e) = simulate_link(&link, &packet.raw).await {
                // Handle errors (ICMP, etc.)
                // ...
                break;
            }
            
            // Successfully forwarded - increment counter
            if let Some(node_idx) = fabric.router_index.get(&ingress) {
                if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                    router.increment_forwarded();
                }
            }
        } else {
            break;
        }
        
        // Move to next router
        ingress = next_hop.clone();
    }
    packet
}
```

2. Add ICMP counter increment for TTL expiration:
```rust
if packet.ttl <= 1 {
    // Generate ICMP
    let icmp_bytes = if is_ipv6(&packet) {
        icmp::generate_icmpv6_error(&packet, 3, 0)
    } else {
        icmp::generate_icmp_error(&packet, 11, 0)
    };
    
    // Increment ICMP counter
    if let Some(node_idx) = fabric.router_index.get(&ingress) {
        if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
            router.increment_icmp();
        }
    }
    // ...
}
```

3. Update `process_packet_multi()` with the same statistics tracking.

4. Add tests to verify statistics are updated:
```rust
#[tokio::test]
async fn test_router_statistics_collected() {
    // Create fabric and routing tables
    // Process packets
    // Verify statistics are non-zero for routers in the path
}
```

## Files to Modify
- `src/processor.rs` (add statistics updates)
- `tests/` (add statistics verification tests)

## Effort Estimate
Small (1-2 hours)

## References
- Original Issue 012: docs/issues/resolved/012-statistics-not-exposed.md

## Related Plans
- Plan 6: Link Simulation (mentions statistics)
- Plan 9: Integration and End-to-End Testing
