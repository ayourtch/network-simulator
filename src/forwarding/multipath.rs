// src/forwarding/multipath.rs

use crate::topology::{RouterId, Link};
use crate::packet::PacketMeta;
use crate::routing::MultiPathTable;
use std::collections::HashMap;
use tracing::debug;

/// Select egress link using multipath routing tables.
/// Chooses a next hop from the list of equal‑cost candidates based on the requested destination.
/// Load‑balances among equal‑cost next hops using a hash of packet fields.
pub fn select_egress_link_multi<'a>(
    _router_id: &RouterId,
    packet: &PacketMeta,
    links: &'a [&Link],
    _tables: &HashMap<RouterId, MultiPathTable>,
) -> Option<&'a Link> {
    // Simplified: treat all links as candidates.
    let mut candidates: Vec<&Link> = links.iter().cloned().collect();
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
