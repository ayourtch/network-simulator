// src/processor.rs

use crate::routing::{Destination, RoutingTable};
use crate::routing::multipath::MultiPathTable;
use crate::topology::{Fabric, RouterId};
use crate::packet::{self, PacketMeta};
use crate::simulation::simulate_link;
use crate::icmp;
use tracing::{debug, error};
use std::collections::HashMap;

/// Helper to determine if a packet is IPv6.
fn is_ipv6(packet: &PacketMeta) -> bool {
    matches!(packet.src_ip, std::net::IpAddr::V6(_))
}

/// Returns the opposite destination (used for ICMP replies).
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
    loop {
        // Decrement TTL / Hop Limit.
        if let Err(e) = packet.decrement_ttl() {
            error!("Failed to decrement TTL: {}", e);
            break;
        }

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

        // Simulate the link.
        if let Some(link) = fabric.get_link(&ingress, next_hop) {
            if let Err(e) = simulate_link(&link, &packet.raw).await {
                // Packet dropped – possibly generate ICMP error.
                if e == "mtu_exceeded" {
                    let icmp_bytes = if is_ipv6(&packet) {
                        icmp::generate_icmpv6_error(&packet, 2, 0)
                    } else {
                        icmp::generate_icmp_error(&packet, 3, 4)
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
                        continue; // forward the ICMP reply
                    } else {
                        return packet;
                    }
                }
                break;
            }
        } else {
            debug!("No link between {} and {}", ingress.0, next_hop.0);
            break;
        }

        // Move to next router.
        ingress = next_hop.clone();
    }
    packet
}

/// Process a packet using multipath routing tables.
/// Currently a placeholder that reuses single‑path logic.
pub async fn process_packet_multi(
    fabric: &mut Fabric,
    _tables: &HashMap<RouterId, MultiPathTable>,
    ingress: RouterId,
    packet: PacketMeta,
    destination: Destination,
) -> PacketMeta {
    // Construct a dummy single‑path routing table that forwards to itself.
    let dummy_entry = crate::routing::RouteEntry {
        next_hop: ingress.clone(),
        total_cost: 0,
    };
    let dummy_table = RoutingTable {
        tun_a: dummy_entry.clone(),
        tun_b: dummy_entry,
    };
    let mut map = HashMap::new();
    map.insert(ingress.clone(), dummy_table);
    process_packet(fabric, &map, ingress, packet, destination).await
}
