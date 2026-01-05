// src/processor.rs

use crate::routing::{Destination, RoutingTable};
use crate::routing::multipath::MultiPathTable;
use crate::topology::{Fabric, RouterId};
use crate::packet::{self, PacketMeta};

use crate::simulation::{simulate_link, SimulationError};
use crate::forwarding::select_egress_link;
use crate::icmp;
use tracing::{debug, error};
use std::collections::HashMap;

/// Helper to determine if a packet is IPv6.
fn is_ipv6(packet: &PacketMeta) -> bool {
    matches!(packet.src_ip, std::net::IpAddr::V6(_))
}

/// Returns the opposite destination (used for ICMP replies).
/// Returns the opposite destination (used for ICMP replies).
/// This function swaps the direction of traffic when generating an ICMP error response,
/// so that the reply is sent back towards the original packet source.
fn opposite_destination(dest: Destination) -> Destination {
    match dest {
        Destination::TunA => Destination::TunB,
        Destination::TunB => Destination::TunA,
    }
}

/// Process a packet using single‑path routing tables.
/// Returns the packet after processing (may be modified, e.g., TTL decrement).
pub async fn process_packet(
    fabric: &mut Fabric,
    tables: &HashMap<RouterId, RoutingTable>,
    mut ingress: RouterId,
    mut packet: PacketMeta,
    mut destination: Destination,
) -> PacketMeta {
    // Loop forwarding hop‑by‑hop until we cannot forward further.
    let mut hop_count = 0usize;
    loop {
        hop_count += 1;
        if hop_count > 100 {
            debug!("Hop limit exceeded, breaking to avoid infinite loop");
            break;
        }
        // Increment received packet counter for the current router.
        if let Some(node_idx) = fabric.router_index.get(&ingress) {
            if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                router.increment_received();
            }
        }
        // Check for TTL expiration before decrementing.
        if packet.ttl <= 1 {
            // TTL will expire; generate ICMP Time Exceeded (IPv4 type 11, code 0) or ICMPv6 Time Exceeded (type 3, code 0).
            let icmp_bytes = if is_ipv6(&packet) {
                icmp::generate_icmpv6_error(&packet, 3, 0)
            } else {
                icmp::generate_icmp_error(&packet, 11, 0)
            };
            // Increment ICMP counter for this router.
            if let Some(node_idx) = fabric.router_index.get(&ingress) {
                if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                    router.increment_icmp();
                }
            }
            // Parse ICMP packet and set up reverse routing.
            if let Ok(icmp_packet) = packet::parse(&icmp_bytes) {
                packet = icmp_packet;
                destination = opposite_destination(destination);
                // Do not decrement TTL for the original packet; continue processing the ICMP reply.
                continue;
            } else {
                // If parsing fails, return the original packet unchanged.
                break;
            }
        }
        // TTL decrement moved after destination detection.

        // Get routing table for current router.
        let table = match tables.get(&ingress) {
            Some(t) => t,
            None => {
                debug!("No routing table for router {}", ingress.0);
                break;
            }
        };
        let next_hop = match destination {
            Destination::TunA => &table.tun_a.next_hop,
            Destination::TunB => &table.tun_b.next_hop,
        };

        // Destination detection: if next hop is the current router, packet has arrived at its destination.
        if next_hop == &ingress {
            debug!("Packet reached destination router {}", ingress.0);
            break;
        }

        // Decrement TTL / Hop Limit after confirming we are not at destination.
        if let Err(e) = packet.decrement_ttl() {
            error!("Failed to decrement TTL: {}", e);
            break;
        }

        // Select egress link using forwarding engine (supports load‑balancing).
        let incident_links = fabric.incident_links(&ingress);
        let link_opt = select_egress_link(&ingress, &packet, &incident_links, tables, destination);
        let link = match link_opt {
            Some(l) => l,
            None => {
                debug!("No egress link selected for router {}", ingress.0);
                break;
            }
        };
        // Determine the next hop router from the selected link.
        let next_hop = if link.id.a == ingress {
            link.id.b.clone()
        } else {
            link.id.a.clone()
        };
        if let Err(e) = simulate_link(link, &packet.raw).await {
            match e {
                SimulationError::MtuExceeded { .. } => {
                    let icmp_bytes = if is_ipv6(&packet) {
                        icmp::generate_icmpv6_error(&packet, 2, 0)
                    } else {
                        icmp::generate_icmp_error(&packet, 3, 4)
                    };
                    if let Some(node_idx) = fabric.router_index.get(&ingress) {
                        if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                            router.increment_icmp();
                        }
                    }
                    if let Ok(icmp_packet) = packet::parse(&icmp_bytes) {
                        packet = icmp_packet;
                        destination = opposite_destination(destination);
                        continue;
                    } else {
                        return packet;
                    }
                },
                SimulationError::PacketLost => {
                    debug!("Packet lost on link between {} and {}", ingress.0, next_hop.0);
                    if let Some(node_idx) = fabric.router_index.get(&ingress) {
                        if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                            router.increment_lost();
                        }
                    }
                    break;
                },
                _ => {
                    break;
                }
            }
        } else {
            // Successful forwarding – increment forwarded counter.
            if let Some(node_idx) = fabric.router_index.get(&ingress) {
                if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                    router.increment_forwarded();
                }
            }
            // Move to next router for next hop.
            ingress = next_hop.clone();
            continue;
        }
    }
    packet
}

/// Process a packet using multipath routing tables.
/// Currently a placeholder that reuses single‑path logic.
pub async fn process_packet_multi(
    fabric: &mut Fabric,
    tables: &HashMap<RouterId, MultiPathTable>,
    mut ingress: RouterId,
    mut packet: PacketMeta,
    mut destination: Destination,
) -> PacketMeta {
    // Multipath processing loop similar to single‑path but selects from equal‑cost next hops.
    let mut hop_count = 0usize;
    loop {
        // Increment hop count to avoid infinite loops.
        hop_count += 1;
        if hop_count > 100 {
            debug!("Hop limit exceeded in multipath processing, breaking to avoid infinite loop");
            break;
        }
        // Increment received counter for the current router.
        if let Some(node_idx) = fabric.router_index.get(&ingress) {
            if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                router.increment_received();
            }
        }
        // TTL expiration handling (same as single‑path).
        if packet.ttl <= 1 {
            let icmp_bytes = if is_ipv6(&packet) {
                icmp::generate_icmpv6_error(&packet, 3, 0)
            } else {
                icmp::generate_icmp_error(&packet, 11, 0)
            };
            if let Some(node_idx) = fabric.router_index.get(&ingress) {
                if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                    router.increment_icmp();
                }
            }
            if let Ok(icmp_packet) = packet::parse(&icmp_bytes) {
                packet = icmp_packet;
                destination = opposite_destination(destination);
                continue;
            } else {
                break;
            }
        }
        // Decrement TTL.
        if let Err(e) = packet.decrement_ttl() {
            error!("Failed to decrement TTL: {}", e);
            break;
        }
        // Retrieve multipath table for current router.
        let mtable = match tables.get(&ingress) {
            Some(t) => t,
            None => {
                debug!("No multipath table for router {}", ingress.0);
                break;
            }
        };
        // Select appropriate next‑hop list based on destination.
        let entries = match destination {
            Destination::TunA => &mtable.tun_a,
            Destination::TunB => &mtable.tun_b,
        };
        if entries.is_empty() {
            debug!("No multipath entries for router {}", ingress.0);
            break;
        }
        // Choose a next hop using a hash of the packet 5‑tuple.
        let next_hop_id = select_next_hop_by_hash(&packet, entries);
        // Simulate the link.
        if let Some(link) = fabric.get_link(&ingress, next_hop_id) {
            if let Err(e) = simulate_link(&link, &packet.raw).await {
                match e {
                    SimulationError::MtuExceeded { .. } => {
                        let icmp_bytes = if is_ipv6(&packet) {
                            icmp::generate_icmpv6_error(&packet, 2, 0)
                        } else {
                            icmp::generate_icmp_error(&packet, 3, 4)
                        };
                        if let Some(node_idx) = fabric.router_index.get(&ingress) {
                            if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                                router.increment_icmp();
                            }
                        }
                        if let Ok(icmp_packet) = packet::parse(&icmp_bytes) {
                            packet = icmp_packet;
                            destination = opposite_destination(destination);
                            continue;
                        } else {
                            return packet;
                        }
                    },
                    SimulationError::PacketLost => {
                        debug!("Packet lost on link between {} and {}", ingress.0, next_hop_id.0);
                        if let Some(node_idx) = fabric.router_index.get(&ingress) {
                            if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                                router.increment_lost();
                            }
                        }
                        break;
                    },
                    _ => {
                        break;
                    }
                }
            }
            // Successful forwarding – increment counter.
            if let Some(node_idx) = fabric.router_index.get(&ingress) {
                if let Some(router) = fabric.graph.node_weight_mut(*node_idx) {
                    router.increment_forwarded();
                }
            }
        } else {
            debug!("No link between {} and {}", ingress.0, next_hop_id.0);
            break;
        }
        // Move to next router.
        ingress = next_hop_id.clone();
    }
    packet
}

/// Select a next‑hop router from a list of equal‑cost entries using a hash of the packet's 5‑tuple.
fn select_next_hop_by_hash<'a>(packet: &PacketMeta, entries: &'a [crate::routing::RouteEntry]) -> &'a RouterId {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    packet.src_ip.hash(&mut hasher);
    packet.dst_ip.hash(&mut hasher);
    packet.src_port.hash(&mut hasher);
    packet.dst_port.hash(&mut hasher);
    packet.protocol.hash(&mut hasher);
    let hash = hasher.finish();
    let idx = (hash as usize) % entries.len();
    &entries[idx].next_hop
}
