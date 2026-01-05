# Issue 027: Forwarding Loop Lacks Proper Destination Detection

## Summary
The packet forwarding loop in `process_packet()` continues until no link is found, but there's no proper detection of when a packet has reached its destination (ingress router for the target TUN). This means packets may loop indefinitely or fail to be properly delivered.

## Location
- File: `src/processor.rs`
- Function: `process_packet()` (lines 38-93)

## Current Behavior
```rust
loop {
    // ... TTL decrement ...
    
    let table = match tables.get(&ingress) { ... };
    let next_hop = match destination {
        Destination::TunA => &table.tun_a.next_hop,
        Destination::TunB => &table.tun_b.next_hop,
    };
    
    if let Some(link) = fabric.get_link(&ingress, next_hop) {
        // ... simulate link ...
    } else {
        debug!("No link between {} and {}", ingress.0, next_hop.0);
        break;  // Exit only when no link found
    }
    
    ingress = next_hop.clone();
}
```

Issues:
1. Loop only exits when `fabric.get_link()` returns `None`
2. No check for whether the packet has reached its destination
3. When routing table points router to itself (at destination), there's no link to self, so it breaks - but this is accidental, not intentional
4. The packet is returned but there's no indication of success/failure

## Expected Behavior (from Plan 4 and Plan 9)
1. Detect when packet reaches the destination ingress router
2. Return packet with success indication for delivery to TUN
3. Clearly distinguish between:
   - Successfully delivered (reached destination)
   - Dropped due to TTL expiration
   - Dropped due to MTU exceeded
   - Dropped due to packet loss
   - Failed due to routing error

## Recommended Solution

1. Add destination detection to the forwarding loop:
```rust
pub async fn process_packet(
    fabric: &mut Fabric,
    tables: &HashMap<RouterId, RoutingTable>,
    mut ingress: RouterId,
    mut packet: PacketMeta,
    destination: Destination,
    destination_router: &RouterId,  // Add parameter for destination
) -> (PacketMeta, ForwardingResult) {
    loop {
        // Check if we've reached the destination
        if ingress == *destination_router {
            debug!("Packet reached destination router {}", ingress.0);
            return (packet, ForwardingResult::Delivered);
        }
        
        // Check TTL
        if packet.ttl <= 1 {
            // Generate ICMP and return
            return (packet, ForwardingResult::TtlExpired);
        }
        
        // Decrement TTL
        if let Err(e) = packet.decrement_ttl() {
            return (packet, ForwardingResult::Error(e.to_string()));
        }
        
        // ... routing and forwarding ...
    }
}

#[derive(Debug)]
pub enum ForwardingResult {
    Delivered,
    TtlExpired,
    MtuExceeded(u32),  // Include the MTU that was exceeded
    PacketLost,
    RoutingError(String),
    Error(String),
}
```

2. Update callers to pass the destination router:
```rust
// In tun/mod.rs
let destination_router = match destination {
    Destination::TunA => &ingress_a,
    Destination::TunB => &ingress_b,
};
let (processed, result) = process_packet(
    fabric, &routing_tables, ingress, packet, destination, destination_router
).await;

match result {
    ForwardingResult::Delivered => {
        // Write to TUN
    }
    ForwardingResult::TtlExpired => {
        // ICMP already generated and routed
    }
    // ... handle other cases ...
}
```

3. Add tests for destination detection:
```rust
#[tokio::test]
async fn test_packet_reaches_destination() {
    // Create simple topology: A -> B -> C
    // Inject at A, destination C
    // Verify packet is delivered with ForwardingResult::Delivered
}

#[tokio::test]
async fn test_packet_stops_at_destination() {
    // Verify forwarding loop exits when destination is reached
    // Not when link is missing
}
```

## Files to Modify
- `src/processor.rs` (add destination detection and result type)
- `src/tun/mod.rs` (update to use ForwardingResult)
- `tests/` (add destination detection tests)

## Effort Estimate
Medium (2-3 hours)

## Related Plans
- Plan 4: Packet Processing and Forwarding Engine
- Plan 9: Integration and End-to-End Testing
