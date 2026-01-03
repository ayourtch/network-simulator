use crate::topology::{Fabric, RouterId};
use crate::routing::RoutingTable;
use crate::forwarding;
use crate::simulation;
use crate::packet::PacketMeta;
use std::collections::HashMap;
use tracing::{info, debug, error};

/// Process a single packet starting at the given ingress router.
/// This function demonstrates a minimal forwarding path: it looks up the routing
/// table for the ingress router, selects the appropriate egress link, simulates the
/// link characteristics, and logs the result. In a full implementation it would
/// continue hopping until the packet reaches its destination (e.g., a TUN
/// interface) and would handle ICMP generation, TTL decrement, etc.
pub async fn process_packet(
    fabric: &Fabric,
    tables: &HashMap<RouterId, RoutingTable>,
    ingress: RouterId,
    packet: PacketMeta,
) {
    debug!("Processing packet at ingress {}", ingress.0);
    // Retrieve all incident links for the ingress router.
    let incident = fabric.incident_links(&ingress);
    if incident.is_empty() {
        error!("No links found for router {}", ingress.0);
        return;
    }
    // Choose egress link based on routing tables.
    let egress = forwarding::select_egress_link(&ingress, &packet, incident.as_slice(), tables);
    match egress {
        Some(link) => {
            debug!("Selected egress link {:?}", link.id);
            // Simulate link characteristics (delay, loss, jitter).
            if let Err(e) = simulation::simulate_link(link, &[]).await {
                error!("Link simulation error: {}", e);
                return;
            }
            info!("Packet forwarded from {} via link {:?}", ingress.0, link.id);
        }
        None => {
            error!("Failed to select egress link for router {}", ingress.0);
        }
    }
}
