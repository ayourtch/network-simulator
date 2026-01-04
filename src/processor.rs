use crate::topology::{Fabric, RouterId};
use crate::routing::{RoutingTable, MultiPathTable};
use crate::forwarding;
use crate::simulation;
use crate::packet::PacketMeta;
use crate::icmp;
use std::collections::HashMap;
use tracing::{info, debug, error};

/// Process a packet using the standard routing table.
/// Implements hop‑by‑hop forwarding until TTL expires or no further hop is found.
pub async fn process_packet(
    fabric: &Fabric,
    tables: &HashMap<RouterId, RoutingTable>,
    ingress: RouterId,
    mut packet: PacketMeta,
) {
    debug!("Starting packet processing at ingress {}", ingress.0);
    let mut current = ingress.clone();
    loop {
        // Decrement TTL / Hop Limit
        if packet.ttl <= 1 {
            debug!("TTL expired at router {}. Dropping packet.", current.0);
            let _ = icmp::generate_icmp_error(&packet, 11, 0); // Time Exceeded stub
            return;
        }
        packet.ttl -= 1;
        // Find incident links for the current router
        let incident = fabric.incident_links(&current);
        if incident.is_empty() {
            error!("No links found for router {}", current.0);
            return;
        }
        // Select egress link
        let egress = forwarding::select_egress_link(&current, &packet, incident.as_slice(), tables);
        let egress = match egress {
            Some(l) => l,
            None => {
                error!("Failed to select egress link for router {}", current.0);
                return;
            }
        };
        debug!("Selected egress link {:?} from router {}", egress.id, current.0);
        // Simulate the link
        if let Err(e) = simulation::simulate_link(egress, &[]).await {
            error!("Link simulation error on router {}: {}", current.0, e);
            if e == "mtu_exceeded" {
                let _ = icmp::generate_icmp_error(&packet, 3, 4); // Fragmentation Needed stub
            }
            return;
        }
        // Move to the next router across the link
        let next_router = if egress.id.a == current { egress.id.b.clone() } else { egress.id.a.clone() };
        if next_router == current {
            debug!("Reached router {} with no further hop, stopping.", current.0);
            return;
        }
        current = next_router;
    }
}

/// Process a packet using multipath routing tables.
/// Mirrors the logic of `process_packet` but uses multipath routing.
pub async fn process_packet_multi(
    fabric: &Fabric,
    tables: &HashMap<RouterId, MultiPathTable>,
    ingress: RouterId,
    mut packet: PacketMeta,
) {
    debug!("Starting multipath packet processing at ingress {}", ingress.0);
    let mut current = ingress.clone();
    loop {
        if packet.ttl <= 1 {
            debug!("TTL expired at router {}. Dropping packet.", current.0);
            let _ = icmp::generate_icmp_error(&packet, 11, 0);
            return;
        }
        packet.ttl -= 1;
        let incident = fabric.incident_links(&current);
        if incident.is_empty() {
            error!("No links found for router {}", current.0);
            return;
        }
        let egress = forwarding::multipath::select_egress_link_multi(&current, &packet, incident.as_slice(), tables);
        let egress = match egress {
            Some(l) => l,
            None => {
                error!("Failed to select egress link for router {}", current.0);
                return;
            }
        };
        debug!("Selected multipath egress link {:?} from router {}", egress.id, current.0);
        if let Err(e) = simulation::simulate_link(egress, &[]).await {
            error!("Link simulation error on router {}: {}", current.0, e);
            if e == "mtu_exceeded" {
                let _ = icmp::generate_icmp_error(&packet, 3, 4);
            }
            return;
        }
        let next_router = if egress.id.a == current { egress.id.b.clone() } else { egress.id.a.clone() };
        if next_router == current {
            debug!("Reached router {} with no further hop, stopping.", current.0);
            return;
        }
        current = next_router;
    }
}
