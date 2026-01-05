// src/lib.rs

pub mod config;
pub mod topology;
pub mod routing;
pub use routing::Destination;
pub mod tun;
pub mod packet;
pub mod simulation;
pub mod icmp;
pub mod forwarding;
pub mod processor;

use crate::config::SimulatorConfig;
use crate::topology::Fabric;
use crate::topology::router::RouterId;
use tracing::{info, error, debug};
use crate::processor::{process_packet, process_packet_multi};
use std::collections::HashMap;



/// Compute routing tables (single‑path) for the given configuration.
pub fn compute_routing_tables(cfg: &SimulatorConfig) -> HashMap<RouterId, routing::RoutingTable> {
    let mut fabric = Fabric::new();
    // Populate fabric as in `run` (routers and links)
    for router_id in cfg.topology.routers.keys() {
        let router = topology::router::Router {
            id: RouterId(router_id.clone()),
            routing: routing::RoutingTable::default(),
            stats: topology::router::RouterStats::default(),
        };
        fabric.add_router(router);
    }
    for (link_name, link_cfg) in cfg.topology.links.iter() {
        let parts: Vec<&str> = link_name.split('_').collect();
        if parts.len() != 2 { continue; }
        let a = RouterId(parts[0].to_string());
        let b = RouterId(parts[1].to_string());
        if fabric.router_index.contains_key(&a) && fabric.router_index.contains_key(&b) {
            fabric.add_link(&a, &b, link_cfg.clone());
        }
    }
    let ingress_a = RouterId(cfg.tun_ingress.tun_a_ingress.clone());
    let ingress_b = RouterId(cfg.tun_ingress.tun_b_ingress.clone());
    routing::compute_routing(&fabric, ingress_a, ingress_b)
}

/// Compute multipath routing tables (if enabled) for the given configuration.
pub fn compute_multipath_tables(cfg: &SimulatorConfig) -> HashMap<RouterId, routing::MultiPathTable> {
    if !cfg.enable_multipath {
        return HashMap::new();
    }
    let mut fabric = Fabric::new();
    for router_id in cfg.topology.routers.keys() {
        let router = topology::router::Router {
            id: RouterId(router_id.clone()),
            routing: routing::RoutingTable::default(),
            stats: topology::router::RouterStats::default(),
        };
        fabric.add_router(router);
    }
    for (link_name, link_cfg) in cfg.topology.links.iter() {
        let parts: Vec<&str> = link_name.split('_').collect();
        if parts.len() != 2 { continue; }
        let a = RouterId(parts[0].to_string());
        let b = RouterId(parts[1].to_string());
        if fabric.router_index.contains_key(&a) && fabric.router_index.contains_key(&b) {
            fabric.add_link(&a, &b, link_cfg.clone());
        }
    }
    let ingress_a = RouterId(cfg.tun_ingress.tun_a_ingress.clone());
    let ingress_b = RouterId(cfg.tun_ingress.tun_b_ingress.clone());
    routing::compute_multi_path_routing(&fabric, ingress_a, ingress_b)
}



/// Entry point called from `main.rs`. Parses the configuration, builds the fabric,
/// computes routing tables and (for now) immediately shuts down.
pub async fn run(cfg: SimulatorConfig) -> Result<Fabric, Box<dyn std::error::Error>> {
    // Build fabric (stub implementation for now)
    let mut fabric = Fabric::new();
    // Add routers from config
    for router_id in cfg.topology.routers.keys() {
        let router = topology::router::Router {
            id: RouterId(router_id.clone()),
            routing: routing::RoutingTable {
                tun_a: routing::RouteEntry { next_hop: RouterId("".to_string()), total_cost: 0 },
                tun_b: routing::RouteEntry { next_hop: RouterId("".to_string()), total_cost: 0 },
            },
            stats: topology::router::RouterStats::default(),
        };
        fabric.add_router(router);
    }
    // Add links from config (very simplified – only adds if both ends exist)
    for (link_name, link_cfg) in cfg.topology.links.iter() {
        // split on '_' to get the two router ids
        let parts: Vec<&str> = link_name.split('_').collect();
        if parts.len() != 2 { continue; }
        let a = RouterId(parts[0].to_string());
        let b = RouterId(parts[1].to_string());
        if fabric.router_index.contains_key(&a) && fabric.router_index.contains_key(&b) {
            fabric.add_link(&a, &b, link_cfg.clone());
        } else {
            error!("Link {} references unknown router(s)", link_name);
        }
    }
    info!("Fabric built with {} routers and {} links", fabric.router_index.len(), fabric.link_index.len());

    // Compute routing tables (stub – just logs)
    let ingress_a = RouterId(cfg.tun_ingress.tun_a_ingress.clone());
    let ingress_b = RouterId(cfg.tun_ingress.tun_b_ingress.clone());
    let tables = routing::compute_routing(&fabric, ingress_a.clone(), ingress_b.clone());
    let multi_tables = if cfg.enable_multipath {
        routing::compute_multi_path_routing(&fabric, ingress_a.clone(), ingress_b.clone())
    } else {
        HashMap::new()
    };
    if cfg.enable_multipath {
        info!("Multipath routing enabled");
    } else {
        info!("Multipath routing disabled");
    }

    // Demonstration: process a dummy packet at the first router (if any).
    if let Some((first_router_id, _)) = fabric.router_index.iter().next() {
        let first_router_id = first_router_id.clone();
        let dummy_packet = packet::PacketMeta {
            src_ip: "10.0.0.1".parse().unwrap(),
            dst_ip: "10.0.1.1".parse().unwrap(),
            src_port: 12345,
            dst_port: 80,
            protocol: 6, // TCP
            ttl: 64,
            raw: vec![],
        };
        debug!("Processing dummy packet at router {}", first_router_id.0);
        // Determine destination based on which ingress the router is (simplified):
        let destination = if first_router_id == ingress_a { Destination::TunB } else { Destination::TunA };
        // Use single-path forwarding for now
        process_packet(&mut fabric, &tables, first_router_id.clone(), dummy_packet.clone(), destination).await;
        // Also demonstrate multipath forwarding
        if cfg.enable_multipath {
            process_packet_multi(&mut fabric, &multi_tables, first_router_id.clone(), dummy_packet, destination).await;
        }
    }

    // Start TUN handling (stub)
    if let Err(e) = tun::start(&cfg, &mut fabric).await {
        error!("Failed to start TUN handling: {}", e);
    }
    // Print final statistics (always printed; CLI flag may control additional output)
    fabric.print_statistics();

    Ok(fabric)
}
