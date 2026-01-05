# Issue 032: No Load Balancing Per-Packet Counter Usage in Processor

## Summary
The `Link` struct has a `counter` field (AtomicU64) that's intended for per-packet load balancing. The forwarding engine (`src/forwarding/mod.rs`) uses this counter in its hash calculation for link selection, but the processor (`src/processor.rs`) doesn't use the forwarding engine's `select_egress_link()` function - it just looks up the routing table directly.

## Location
- File: `src/processor.rs`
- Function: `process_packet()` (lines 46-56, 59-84)

## Current Behavior
```rust
// In process_packet():
let table = match tables.get(&ingress) { ... };
let next_hop = match destination {
    Destination::TunA => &table.tun_a.next_hop,
    Destination::TunB => &table.tun_b.next_hop,
};

// Directly look up link without using forwarding engine
if let Some(link) = fabric.get_link(&ingress, next_hop) {
    if let Err(e) = simulate_link(&link, &packet.raw).await {
        // ...
    }
}
```

The `select_egress_link()` function in `src/forwarding/mod.rs` implements:
- 5-tuple hashing
- Link counter-based load balancing
- Selection among multiple candidate links

But this function is never called during actual packet processing.

## Expected Behavior (from Plan 8)
1. Use the forwarding engine's `select_egress_link()` for path selection
2. Include link counters in the hash for per-packet load balancing
3. When `load_balance` is enabled on links, distribute traffic across them

## Recommended Solution

1. Integrate the forwarding engine into the processor:
```rust
use crate::forwarding::select_egress_link;

pub async fn process_packet(
    fabric: &mut Fabric,
    tables: &HashMap<RouterId, RoutingTable>,
    mut ingress: RouterId,
    mut packet: PacketMeta,
    mut destination: Destination,
) -> PacketMeta {
    loop {
        // ... TTL handling ...
        
        // Get incident links for current router
        let links: Vec<&Link> = fabric.incident_links(&ingress);
        
        // Use forwarding engine to select egress link
        let link = match select_egress_link(&ingress, &packet, &links, tables, destination) {
            Some(l) => l,
            None => {
                debug!("No egress link selected for router {}", ingress.0);
                break;
            }
        };
        
        // Simulate the selected link
        if let Err(e) = simulate_link(link, &packet.raw).await {
            // Handle error
            break;
        }
        
        // Determine next router from link
        let next_hop = if link.id.a == ingress {
            link.id.b.clone()
        } else {
            link.id.a.clone()
        };
        
        ingress = next_hop;
    }
    packet
}
```

2. Add tests verifying load balancing:
```rust
#[tokio::test]
async fn test_load_balancing_distributes_traffic() {
    // Create topology with multiple load-balanced links
    // Process many packets with different 5-tuples
    // Verify link counters show distribution
}
```

## Files to Modify
- `src/processor.rs` (integrate forwarding engine)
- `tests/` (add load balancing verification tests)

## Effort Estimate
Small (1-2 hours)

## Related Plans
- Plan 8: Multi-path Routing and Load Balancing
