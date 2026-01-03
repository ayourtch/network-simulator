// src/lib.rs

pub mod config;
pub mod topology;
pub mod routing;
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
use crate::processor::process_packet;

/// Entry point called from `main.rs`. Parses the configuration, builds the fabric,
/// computes routing tables and (for now) immediately shuts down.
pub async fn run(config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let cfg_str = std::fs::read_to_string(config_path)?;
    let cfg: SimulatorConfig = toml::from_str(&cfg_str)?;
    info!("Configuration loaded from {}", config_path);

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
    info!("Routing tables computed");

    // Demonstration: process a dummy packet at the first router (if any).
    if let Some((&first_router_id, _)) = fabric.router_index.iter().next() {
        let dummy_packet = packet::PacketMeta {
            src_ip: "10.0.0.1".parse().unwrap(),
            dst_ip: "10.0.1.1".parse().unwrap(),
            src_port: 12345,
            dst_port: 80,
            protocol: 6, // TCP
            ttl: 64,
            customer_id: 0,
        };
        debug!("Processing dummy packet at router {}", first_router_id.0);
        process_packet(&fabric, &tables, first_router_id.clone(), dummy_packet).await;
    }

    // TODO: start TUN handling, packet processing, etc.
    Ok(())
}
