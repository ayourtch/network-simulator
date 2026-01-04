// src/forwarding/multipath.rs

use crate::topology::{RouterId, Link};
use crate::packet::PacketMeta;
use crate::routing::MultiPathTable;
use std::collections::HashMap;
use tracing::debug;

/// Select egress link using multipath routing tables.
/// Chooses a next hop from the list of equal‑cost candidates (tun_a preferred, fallback to tun_b).
/// Load‑balances among equal‑cost next hops using a hash of packet fields.
pub fn select_egress_link_multi<'a>(
    router_id: &RouterId,
    packet: &PacketMeta,
    links: &'a [Link],
    tables: &HashMap<RouterId, MultiPathTable>,
) -> Option<&'a Link> {
    let table = tables.get(router_id)?;
    // Prefer tun_a entries, otherwise tun_b.
    let candidates_next_hops = if !table.tun_a.is_empty() {
        &table.tun_a
    } else {
        &table.tun_b
    };
    if candidates_next_hops.is_empty() {
        // No routing info – fall back to any link.
        debug!("No multipath entries for router {}, falling back to any link", router_id.0);
        return links.first();
    }
    // Determine which next hop to use via packet hash.
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    packet.src_ip.hash(&mut hasher);
    packet.dst_ip.hash(&mut hasher);
    packet.src_port.hash(&mut hasher);
    packet.dst_port.hash(&mut hasher);
    packet.protocol.hash(&mut hasher);
    let hash = hasher.finish();
    let idx = (hash as usize) % candidates_next_hops.len();
    let chosen_next_hop = &candidates_next_hops[idx];
    // Find a link that connects to this next hop.
    let mut link_candidates: Vec<&Link> = links
        .iter()
        .filter(|link| {
            (link.id.a == *router_id && link.id.b == chosen_next_hop.next_hop)
                || (link.id.b == *router_id && link.id.a == chosen_next_hop.next_hop)
        })
        .collect();
    if link_candidates.is_empty() {
        // No direct link – fallback to all links (load‑balance may apply).
        link_candidates = links.iter().collect();
    }
    // If any candidate link has load_balance enabled, pick one based on hash.
    let lb_links: Vec<&&Link> = link_candidates.iter().filter(|&&l| l.cfg.load_balance).collect();
    if !lb_links.is_empty() {
        let idx = (hash as usize) % lb_links.len();
        let chosen = *lb_links[idx];
        debug!("Load‑balanced (multipath) selection of link {:?} for router {}", chosen.id, router_id.0);
        return Some(chosen);
    }
    // Default: first candidate.
    let chosen = link_candidates[0];
    debug!("Selected link {:?} for router {} via multipath next hop {}", chosen.id, router_id.0, chosen_next_hop.next_hop.0);
    Some(chosen)
}
