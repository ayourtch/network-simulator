// src/forwarding/mod.rs

use crate::topology::{RouterId, Link};
use crate::packet::PacketMeta;
use std::collections::HashMap;
use tracing::debug;

/// Choose the egress link for a packet based on routing tables and optional load‑balancing.
/// Returns a reference to a link from the provided slice that leads to the next hop.
pub fn select_egress_link<'a>(
    router_id: &RouterId,
    packet: &PacketMeta,
    links: &'a [Link],
    tables: &HashMap<RouterId, crate::routing::RoutingTable>,
) -> Option<&'a Link> {
    debug!("Selecting egress link for router {}", router_id.0);
    // Determine which destination (TunA or TunB) the packet is destined for.
    // For simplicity we route all packets via the TunA table entry. In a full
    // implementation this would inspect the packet's destination IP/subnet.
    let routing = tables.get(router_id)?;
    let next_hop = &routing.tun_a.next_hop;

    // Find the link whose opposite endpoint matches the next_hop router.
    for link in links {
        if (link.id.a == *router_id && link.id.b == *next_hop) ||
           (link.id.b == *router_id && link.id.a == *next_hop) {
            debug!("Selected link {:?} for next hop {}", link.id, next_hop.0);
            return Some(link);
        }
    }
    // No direct link – fallback to the first link (could be expanded with multipath).
    debug!("No direct link found for router {}; falling back to first link", router_id.0);
    links.get(0)
}
