# Issue 033: Packet Loss in simulate_link Not Handled Properly

## Summary
When `simulate_link()` returns `Err("packet lost")` due to configured packet loss, the processor just breaks out of the loop without any specific handling. Unlike MTU exceeded (which generates ICMP), packet loss should be silent - which is correct - but the handling could be more explicit and there's no logging or statistics for packet loss events.

## Location
- File: `src/processor.rs`
- Function: `process_packet()` (lines 60-84)
- File: `src/simulation/mod.rs`
- Function: `simulate_link()` (lines 36-40)

## Current Behavior
```rust
// In process_packet():
if let Err(e) = simulate_link(&link, &packet.raw).await {
    if e == "mtu_exceeded" {
        // Generate ICMP
    }
    break;  // For all other errors including "packet lost", just break
}

// In simulate_link():
if rng.gen_range(0.0..100.0) < link.cfg.loss_percent as f64 {
    debug!("Packet dropped on link {:?} due to loss", link.id);
    return Err("packet lost");
}
```

Issues:
1. No explicit handling for packet loss vs other errors
2. No statistics tracking for lost packets
3. The debug log is at link level, but no router-level tracking
4. Return type uses `&'static str` instead of proper error type

## Expected Behavior
1. Packet loss should be silent (correct)
2. Statistics should track packets lost per router/link
3. Error types should be distinct for different failure modes

## Recommended Solution

1. Create proper error types for simulation:
```rust
// In src/simulation/mod.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimulationError {
    PacketLost,
    MtuExceeded { packet_size: usize, mtu: u32 },
    Other(String),
}

pub async fn simulate_link(link: &Link, packet: &[u8]) -> Result<(), SimulationError> {
    // ... existing logic ...
    
    if rng.gen_range(0.0..100.0) < link.cfg.loss_percent as f64 {
        debug!("Packet dropped on link {:?} due to loss", link.id);
        return Err(SimulationError::PacketLost);
    }
    
    if let Some(mtu) = link.cfg.mtu {
        if packet.len() > mtu as usize {
            return Err(SimulationError::MtuExceeded {
                packet_size: packet.len(),
                mtu,
            });
        }
    }
    
    Ok(())
}
```

2. Add packet loss counter to router statistics:
```rust
// In src/topology/router.rs
#[derive(Debug, Default, Clone)]
pub struct RouterStats {
    pub packets_received: u64,
    pub packets_forwarded: u64,
    pub packets_lost: u64,     // New: track lost packets
    pub icmp_generated: u64,
}

impl Router {
    pub fn increment_lost(&mut self) {
        self.stats.packets_lost += 1;
    }
}
```

3. Handle packet loss explicitly in processor:
```rust
match simulate_link(&link, &packet.raw).await {
    Ok(()) => {
        // Successfully forwarded
    }
    Err(SimulationError::PacketLost) => {
        // Track statistics for packet loss
        if let Some(node_idx) = fabric.router_index.get(&ingress) {
            if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                router.increment_lost();
            }
        }
        break;  // Silent drop
    }
    Err(SimulationError::MtuExceeded { mtu, .. }) => {
        // Generate ICMP Fragmentation Needed
        // ...
    }
    Err(SimulationError::Other(msg)) => {
        error!("Simulation error: {}", msg);
        break;
    }
}
```

4. Add tests for packet loss statistics:
```rust
#[tokio::test]
async fn test_packet_loss_tracked_in_statistics() {
    // Configure link with 100% loss
    // Process packets
    // Verify packets_lost counter is non-zero
}
```

## Files to Modify
- `src/simulation/mod.rs` (add proper error types)
- `src/topology/router.rs` (add packets_lost counter)
- `src/processor.rs` (handle packet loss explicitly)
- `tests/` (add packet loss statistics tests)

## Effort Estimate
Small (1-2 hours)

## Related Plans
- Plan 6: Link Simulation (MTU, Delay, Jitter, Loss)
