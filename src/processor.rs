use crate::topology::{Fabric, RouterId};
use crate::routing::{RoutingTable, MultiPathTable};
use crate::forwarding;
use crate::simulation;
use crate::packet::PacketMeta;
use std::collections::HashMap;
use tracing::{info, debug, error};

/// Process a packet using the standard routing table.
pub async fn process_packet(
    fabric: &Fabric,
    tables: &HashMap<RouterId, RoutingTable>,
    ingress: RouterId,
    packet: PacketMeta,
) {
    debug!("Processing packet at ingress {}", ingress.0);
    let incident = fabric.incident_links(&ingress);
    if incident.is_empty() {
        error!("No links found for router {}", ingress.0);
        return;
    }
    let egress = forwarding::select_egress_link(&ingress, &packet, incident.as_slice(), tables);
    match egress {
        Some(link) => {
            debug!("Selected egress link {:?}", link.id);
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

/// Process a packet using multipath routing tables.
pub async fn process_packet_multi(
    fabric: &Fabric,
    tables: &HashMap<RouterId, MultiPathTable>,
    ingress: RouterId,
    packet: PacketMeta,
) {
    debug!("Processing packet (multipath) at ingress {}", ingress.0);
    let incident = fabric.incident_links(&ingress);
    if incident.is_empty() {
        error!("No links found for router {}", ingress.0);
        return;
    }
    let egress = forwarding::multipath::select_egress_link_multi(&ingress, &packet, incident.as_slice(), tables);
    match egress {
        Some(link) => {
            debug!("Selected egress link {:?}", link.id);
            if let Err(e) = simulation::simulate_link(link, &[]).await {
                error!("Link simulation error: {}", e);
                return;
            }
            info!("Packet forwarded (multipath) from {} via link {:?}", ingress.0, link.id);
        }
        None => {
            error!("Failed to select egress link for router {}", ingress.0);
        }
    }
}
