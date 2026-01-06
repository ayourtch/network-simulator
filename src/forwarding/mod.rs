// src/forwarding/mod.rs

use crate::packet::PacketMeta;
use crate::topology::{Link, RouterId};
use std::collections::HashMap;
use tracing::debug;

pub mod multipath;

/// Choose the egress link for a packet based on routing tables and optional load‑balancing.
/// Returns a reference to a link from the provided slice that leads to the next hop.
pub fn select_egress_link<'a>(
    router_id: &RouterId,
    packet: &PacketMeta,
    links: &'a [&Link],
    tables: &HashMap<RouterId, crate::routing::RoutingTable>,
    destination: crate::routing::Destination,
) -> Option<&'a Link> {
    debug!("Selecting egress link for router {}", router_id.0);
    let routing = tables.get(router_id)?;
    let next_hop = match destination {
        crate::routing::Destination::TunA => &routing.tun_a.next_hop,
        crate::routing::Destination::TunB => &routing.tun_b.next_hop,
    };

    // Gather candidate links that lead to the next_hop.
    let mut candidates: Vec<&Link> = links
        .iter()
        .cloned()
        .filter(|link| {
            (link.id.a == *router_id && link.id.b == *next_hop)
                || (link.id.b == *router_id && link.id.a == *next_hop)
        })
        .collect();

    if candidates.is_empty() {
        // No direct link – fallback to all links for possible load‑balancing.
        candidates = links.to_vec();
    }

    // Load balancing among links with load_balance enabled.
    let lb_links: Vec<&&Link> = candidates.iter().filter(|&&l| l.cfg.load_balance).collect();
    if !lb_links.is_empty() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::sync::atomic::Ordering;
        let mut hasher = DefaultHasher::new();
        // Hash the 5‑tuple
        packet.src_ip.hash(&mut hasher);
        packet.dst_ip.hash(&mut hasher);
        packet.src_port.hash(&mut hasher);
        packet.dst_port.hash(&mut hasher);
        packet.protocol.hash(&mut hasher);
        // Include the sum of counters of all load‑balanced links to vary per packet
        let total_counter: u64 = lb_links
            .iter()
            .map(|l| l.counter.load(Ordering::Relaxed))
            .sum();
        total_counter.hash(&mut hasher);
        let hash = hasher.finish();
        let idx = (hash as usize) % lb_links.len();
        let chosen = *lb_links[idx];
        debug!(
            "Load‑balanced selection of link {:?} for router {} (with counter)",
            chosen.id, router_id.0
        );
        return Some(chosen);
    }

    // Default: pick first candidate.
    let chosen = candidates[0];
    debug!("Selected link {:?} for next hop {}", chosen.id, next_hop.0);
    Some(chosen)
}
