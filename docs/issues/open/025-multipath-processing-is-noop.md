# Issue 025: Multipath Packet Processing Is a No-Op

## Summary
The `process_packet_multi()` function in `src/processor.rs` doesn't actually use multipath routing tables. Instead, it creates a dummy single-path routing table that forwards packets to the same router (infinite loop potential), making multipath routing non-functional.

## Location
- File: `src/processor.rs`
- Function: `process_packet_multi()` (lines 98-117)

## Current Behavior
```rust
pub async fn process_packet_multi(
    fabric: &mut Fabric,
    _tables: &HashMap<RouterId, MultiPathTable>,  // Note: tables are IGNORED
    ingress: RouterId,
    packet: PacketMeta,
    destination: Destination,
) -> PacketMeta {
    // Construct a dummy single‑path routing table that forwards to itself.
    let dummy_entry = crate::routing::RouteEntry {
        next_hop: ingress.clone(),  // Forwards to SELF - broken!
        total_cost: 0,
    };
    let dummy_table = RoutingTable {
        tun_a: dummy_entry.clone(),
        tun_b: dummy_entry,
    };
    let mut map = HashMap::new();
    map.insert(ingress.clone(), dummy_table);
    process_packet(fabric, &map, ingress, packet, destination).await
}
```

Issues:
1. The `_tables` parameter (multipath tables) is completely ignored
2. A dummy routing table is created that points to the ingress router itself
3. This causes the forwarding loop to immediately fail (no link from router to itself)
4. Multipath load-balancing never occurs

## Expected Behavior (from Plan 8)
1. Use the provided `MultiPathTable` for routing decisions
2. Select from multiple equal-cost next hops using 5-tuple hashing
3. Forward packets using the selected path
4. Maintain consistent path selection for flows (same 5-tuple = same path)

## Recommended Solution

1. Implement proper multipath processing:
```rust
pub async fn process_packet_multi(
    fabric: &mut Fabric,
    tables: &HashMap<RouterId, MultiPathTable>,
    mut ingress: RouterId,
    mut packet: PacketMeta,
    mut destination: Destination,
) -> PacketMeta {
    loop {
        // Check TTL expiration
        if packet.ttl <= 1 {
            // Generate ICMP Time Exceeded (same as single-path)
            break;
        }
        
        // Decrement TTL
        if let Err(e) = packet.decrement_ttl() {
            error!("Failed to decrement TTL: {}", e);
            break;
        }
        
        // Get multipath routing table for current router
        let table = match tables.get(&ingress) {
            Some(t) => t,
            None => {
                debug!("No multipath routing table for router {}", ingress.0);
                break;
            }
        };
        
        // Get equal-cost next hops based on destination
        let entries = match destination {
            Destination::TunA => &table.tun_a,
            Destination::TunB => &table.tun_b,
        };
        
        if entries.is_empty() {
            debug!("No routes available at router {}", ingress.0);
            break;
        }
        
        // Select next hop using 5-tuple hash
        let next_hop = select_next_hop_by_hash(&packet, entries);
        
        // Simulate link and forward
        if let Some(link) = fabric.get_link(&ingress, next_hop) {
            if let Err(e) = simulate_link(&link, &packet.raw).await {
                // Handle errors (MTU exceeded, packet loss, etc.)
                break;
            }
        } else {
            debug!("No link between {} and {}", ingress.0, next_hop.0);
            break;
        }
        
        // Move to next router
        ingress = next_hop.clone();
    }
    packet
}

fn select_next_hop_by_hash<'a>(packet: &PacketMeta, entries: &'a [RouteEntry]) -> &'a RouterId {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    let mut hasher = DefaultHasher::new();
    packet.src_ip.hash(&mut hasher);
    packet.dst_ip.hash(&mut hasher);
    packet.src_port.hash(&mut hasher);
    packet.dst_port.hash(&mut hasher);
    packet.protocol.hash(&mut hasher);
    let hash = hasher.finish();
    
    let idx = (hash as usize) % entries.len();
    &entries[idx].next_hop
}
```

2. Add tests for multipath routing:
```rust
#[tokio::test]
async fn test_multipath_selects_from_equal_cost_paths() {
    // Create topology with multiple equal-cost paths
    // Process packets with different 5-tuples
    // Verify different paths are selected
}

#[tokio::test]
async fn test_multipath_consistent_path_for_same_flow() {
    // Process multiple packets with same 5-tuple
    // Verify same path is selected each time
}
```

## Files to Modify
- `src/processor.rs` (rewrite `process_packet_multi`)
- `tests/` (add multipath processing tests)

## Effort Estimate
Medium (3-4 hours)

## Related Plans
- Plan 8: Multi-path Routing and Load Balancing
