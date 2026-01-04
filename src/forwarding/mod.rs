// src/forwarding/mod.rs

use crate::topology::{RouterId, Link};
use crate::packet::PacketMeta;
use std::collections::HashMap;
use tracing::debug;

pub mod multipath;

/// Choose the egress link for a packet based on routing tables and optional load‑balancing.
/// Returns a reference to a link from the provided slice that leads to the next hop.
pub fn select_egress_link<'a>(
    router_id: &RouterId,
    packet: &PacketMeta,
    links: &'a [Link],
    tables: &HashMap<RouterId, crate::routing::RoutingTable>,
) -> Option<&'a Link> {
    debug!("Selecting egress link for router {}", router_id.0);
    let routing = tables.get(router_id)?;
    let next_hop = &routing.tun_a.next_hop;

    // Gather candidate links that lead to the next_hop.
    let mut candidates: Vec<&Link> = links.iter().filter(|link| {
        (link.id.a == *router_id && link.id.b == *next_hop) ||
        (link.id.b == *router_id && link.id.a == *next_hop)
    }).collect();

    if candidates.is_empty() {
        // No direct link – fallback to all links for possible load‑balancing.
        candidates = links.iter().collect();
    }

    // If any candidate link has load_balance enabled, select based on packet hash.
    let lb_links: Vec<&&Link> = candidates.iter().filter(|&&l| l.cfg.load_balance).collect();
    if !lb_links.is_empty() {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        packet.src_ip.hash(&mut hasher);
        packet.dst_ip.hash(&mut hasher);
        packet.src_port.hash(&mut hasher);
        packet.dst_port.hash(&mut hasher);
        packet.protocol.hash(&mut hasher);
        let hash = hasher.finish();
        let idx = (hash as usize) % lb_links.len();
        let chosen = *lb_links[idx];
        debug!("Load‑balanced selection of link {:?} for router {}", chosen.id, router_id.0);
        return Some(chosen);
    }

    // Default: pick first candidate.
    let chosen = candidates[0];
    debug!("Selected link {:?} for next hop {}", chosen.id, next_hop.0);
    Some(chosen)
}
