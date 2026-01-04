use crate::topology::{Fabric, RouterId};
use crate::routing::{RoutingTable, MultiPathTable, Destination};
use crate::forwarding;
use crate::simulation;
use crate::packet::PacketMeta;
use crate::icmp;
use std::collections::HashMap;
use tracing::{debug, error};

fn is_ipv6(packet: &PacketMeta) -> bool {
    packet.raw.get(0).map(|b| (b >> 4) == 6).unwrap_or(false)
}

/// Process a packet using the standard routing table.
/// Returns the (potentially modified) packet after processing.
pub async fn process_packet(
    fabric: &mut Fabric,
    tables: &HashMap<RouterId, RoutingTable>,
    ingress: RouterId,
    mut packet: PacketMeta,
    destination: Destination,
) -> PacketMeta {
    debug!("Starting packet processing at ingress {}", ingress.0);
    let mut current = ingress.clone();
    loop {
        // Increment received counter at current router
        if let Some(node_idx) = fabric.router_index.get(&current) {
            if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                router.increment_received();
            }
        }
        // Decrement TTL / Hop Limit
        if packet.ttl <= 1 {
            debug!("TTL expired at router {}. Dropping packet.", current.0);
            if is_ipv6(&packet) {
                let _ = icmp::generate_icmpv6_error(&packet, 3, 0); // Time Exceeded for IPv6 (type 3)
            } else {
                let _ = icmp::generate_icmp_error(&packet, 11, 0); // Time Exceeded stub for IPv4
            }
            // Increment ICMP counter
            if let Some(node_idx) = fabric.router_index.get(&current) {
                if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                    router.increment_icmp();
                }
            }
            return packet;
        }
        // Decrement TTL in packet and raw bytes
        packet.decrement_ttl().expect("TTL decrement failed");
        // Get incident links
        let incident_links = fabric.incident_links(&current);
        // Select egress link
        let egress = match forwarding::select_egress_link(&current, &packet, incident_links.as_slice(), tables, destination) {
            Some(l) => l,
            None => {
                error!("Failed to select egress link for router {}", current.0);
                return packet;
            }
        };
        debug!("Selected egress link {:?} from router {}", egress.id, current.0);
        // Simulate the link
        if let Err(e) = simulation::simulate_link(egress, &packet.raw).await {
            error!("Link simulation error on router {}: {}", current.0, e);
            if e == "mtu_exceeded" {
                if is_ipv6(&packet) {
                    let _ = icmp::generate_icmpv6_error(&packet, 2, 0); // Fragmentation Needed for IPv6 (type 2)
                } else {
                    let _ = icmp::generate_icmp_error(&packet, 3, 4); // Fragmentation Needed stub for IPv4
                }
                // Increment ICMP counter
                if let Some(node_idx) = fabric.router_index.get(&current) {
                    if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                        router.increment_icmp();
                    }
                }
            }
            return packet;
        }
        // Compute next router
        let egress_id = egress.id.clone();
        let next_router = if egress_id.a == current { egress_id.b.clone() } else { egress_id.a.clone() };
        // Increment forwarded counter for successful link traversal
        if let Some(node_idx) = fabric.router_index.get(&current) {
            if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                router.increment_forwarded();
            }
        }
        if next_router == current {
            debug!("Reached router {} with no further hop, stopping.", current.0);
            return packet;
        }
        current = next_router;
    }
}

/// Process a packet using multipath routing tables.
/// Returns the (potentially modified) packet after processing.
pub async fn process_packet_multi(
    fabric: &mut Fabric,
    tables: &HashMap<RouterId, MultiPathTable>,
    ingress: RouterId,
    mut packet: PacketMeta,
    destination: Destination,
) -> PacketMeta {
    debug!("Starting multipath packet processing at ingress {}", ingress.0);
    let mut current = ingress.clone();
    loop {
        // Increment received counter
        if let Some(node_idx) = fabric.router_index.get(&current) {
            if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                router.increment_received();
            }
        }
        if packet.ttl <= 1 {
            debug!("TTL expired at router {}. Dropping packet.", current.0);
            if is_ipv6(&packet) {
                let _ = icmp::generate_icmpv6_error(&packet, 3, 0);
            } else {
                let _ = icmp::generate_icmp_error(&packet, 11, 0);
            }
            // Increment ICMP counter
            if let Some(node_idx) = fabric.router_index.get(&current) {
                if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                    router.increment_icmp();
                }
            }
            return packet;
        }
        // Decrement TTL in packet and raw bytes
        packet.decrement_ttl().expect("TTL decrement failed");
        // Get incident links
        let incident_links = fabric.incident_links(&current);
        if incident_links.is_empty() {
            error!("No links found for router {}", current.0);
            return packet;
        }
        // Select egress link using incident links
        let egress = match forwarding::multipath::select_egress_link_multi(&current, &packet, incident_links.as_slice(), tables, destination) {
            Some(l) => l,
            None => {
                error!("Failed to select egress link for router {}", current.0);
                return packet;
            }
        };
        debug!("Selected multipath egress link {:?} from router {}", egress.id, current.0);
        // Simulate the link
        if let Err(e) = simulation::simulate_link(egress, &packet.raw).await {
            error!("Link simulation error on router {}: {}", current.0, e);
            if e == "mtu_exceeded" {
                if is_ipv6(&packet) {
                    let _ = icmp::generate_icmpv6_error(&packet, 2, 0);
                } else {
                    let _ = icmp::generate_icmp_error(&packet, 3, 4);
                }
                // Increment ICMP counter
                if let Some(node_idx) = fabric.router_index.get(&current) {
                    if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                        router.increment_icmp();
                    }
                }
            }
            return packet;
        }
        // Compute next router
        let egress_id = egress.id.clone();
        let next_router = if egress_id.a == current { egress_id.b.clone() } else { egress_id.a.clone() };
        // Increment forwarded counter
        if let Some(node_idx) = fabric.router_index.get(&current) {
            if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                router.increment_forwarded();
            }
        }
        if next_router == current {
            debug!("Reached router {} with no further hop, stopping.", current.0);
            return packet;
        }
        current = next_router;
    }
}
