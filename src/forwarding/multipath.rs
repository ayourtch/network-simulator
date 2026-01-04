// src/forwarding/multipath.rs

use crate::topology::{RouterId, Link};
use crate::packet::PacketMeta;
use crate::routing::{MultiPathTable, Destination};
use std::collections::HashMap;
use tracing::debug;

/// Select egress link using multipath routing tables.
/// Chooses a next hop from the list of equal‑cost candidates based on the requested destination.
/// Load‑balances among equal‑cost next hops using a hash of packet fields.
pub fn select_egress_link_multi<'a>(
    router_id: &RouterId,
    packet: &PacketMeta,
    links: &'a [&Link],
    tables: &HashMap<RouterId, MultiPathTable>,
    destination: Destination,
) -> Option<&'a Link> {
    // Retrieve routing entries for the given destination.
    let routing = tables.get(router_id)?;
    let entries = match destination {
        Destination::TunA => &routing.tun_a,
        Destination::TunB => &routing.tun_b,
    };
    // Build set of next_hop ids for quick lookup.
    let next_hops: std::collections::HashSet<_> = entries.iter().map(|e| &e.next_hop).collect();
    // Filter candidate links that lead to any of the next_hops.
    let mut candidates: Vec<&Link> = links
        .iter()
        .cloned()
        .filter(|link| {
            (next_hops.contains(&link.id.a) && link.id.b == *router_id)
                || (next_hops.contains(&link.id.b) && link.id.a == *router_id)
        })
        .collect();
    if candidates.is_empty() {
        // Fallback: all links if no specific next_hop matches.
        candidates = links.iter().cloned().collect();
    }
    // Load balancing among links with load_balance enabled.
    let lb_links: Vec<&&Link> = candidates.iter().filter(|&&l| l.cfg.load_balance).collect();
    if !lb_links.is_empty() {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        use std::sync::atomic::Ordering;
        let mut hasher = DefaultHasher::new();
        packet.src_ip.hash(&mut hasher);
        packet.dst_ip.hash(&mut hasher);
        packet.src_port.hash(&mut hasher);
        packet.dst_port.hash(&mut hasher);
        packet.protocol.hash(&mut hasher);
        let total_counter: u64 = lb_links.iter().map(|l| l.counter.load(Ordering::Relaxed)).sum();
        total_counter.hash(&mut hasher);
        let hash = hasher.finish();
        let idx = (hash as usize) % lb_links.len();
        let chosen = *lb_links[idx];
        debug!("Load‑balanced (multipath) selection of link {:?}", chosen.id);
        return Some(chosen);
    }
    // Default: first candidate.
    let chosen = candidates[0];
    debug!("Selected link {:?} (multipath fallback)", chosen.id);
    Some(chosen)
}
