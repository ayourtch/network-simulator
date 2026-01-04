use crate::topology::{Fabric, RouterId};
use crate::routing::{RoutingTable, MultiPathTable};
use crate::forwarding;
use crate::simulation;
use crate::packet::PacketMeta;
use crate::icmp;
use std::collections::HashMap;
use tracing::{info, debug, error};

/// Process a packet using the standard routing table.
pub async fn process_packet(
    fabric: &Fabric,
    tables: &HashMap<RouterId, RoutingTable>,
    ingress: RouterId,
    mut packet: PacketMeta,
) {
    debug!("Processing packet at ingress {}", ingress.0);
    // Decrement TTL / Hop Limit
    if packet.ttl <= 1 {
        // TTL expired, drop packet
        debug!("TTL expired for packet at router {}. Dropping packet.", ingress.0);
        return;
    }
    packet.ttl -= 1;
    let incident = fabric.incident_links(&ingress);
    if incident.is_empty() {
        error!("No links found for router {}", ingress.0);
        return;
    }
    let egress = forwarding::select_egress_link(&ingress, &packet, incident.as_slice(), tables);
    match egress {
        Some(link) => {
            debug!("Selected egress link {:?}", link.id);
            // Simulate link and handle possible errors (loss or MTU)
            match simulation::simulate_link(link, &[]).await {
                Ok(_) => {
                    info!("Packet forwarded from {} via link {:?}", ingress.0, link.id);
                },
                Err(e) => {
                    error!("Link simulation error: {}", e);
                    if e == "mtu_exceeded" {
                        // Generate ICMP Fragmentation Needed error (stub)
                        let _icmp = icmp::generate_icmp_error(&packet, 3, 4); // Type 3 Code 4
                    }
                    return;
                }
            }
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
    mut packet: PacketMeta,
) {
    debug!("Processing packet (multipath) at ingress {}", ingress.0);
    // Decrement TTL / Hop Limit
    if packet.ttl <= 1 {
        debug!("TTL expired for packet at router {}. Dropping packet.", ingress.0);
        return;
    }
    packet.ttl -= 1;
    let incident = fabric.incident_links(&ingress);
    if incident.is_empty() {
        error!("No links found for router {}", ingress.0);
        return;
    }
    let egress = forwarding::multipath::select_egress_link_multi(&ingress, &packet, incident.as_slice(), tables);
    match egress {
        Some(link) => {
            debug!("Selected egress link {:?}", link.id);
            // Simulate link and handle possible errors (loss or MTU)
            match simulation::simulate_link(link, &[]).await {
                Ok(_) => {
                    info!("Packet forwarded (multipath) from {} via link {:?}", ingress.0, link.id);
                },
                Err(e) => {
                    error!("Link simulation error: {}", e);
                    if e == "mtu_exceeded" {
                        let _icmp = icmp::generate_icmp_error(&packet, 3, 4);
                    }
                    return;
                }
            }
        }
        None => {
            error!("Failed to select egress link for router {}", ingress.0);
        }
    }
}
