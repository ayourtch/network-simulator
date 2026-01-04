// src/routing/mod.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::topology::{RouterId, Fabric};
use petgraph::visit::EdgeRef;
use petgraph::algo::dijkstra;

pub mod multipath;
pub use multipath::{MultiPathTable, compute_multi_path_routing};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Destination {
    TunA,
    TunB,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    pub next_hop: RouterId,
    pub total_cost: u32,
}

impl Default for RouteEntry {
    fn default() -> Self {
        RouteEntry { next_hop: RouterId("".to_string()), total_cost: 0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingTable {
    pub tun_a: RouteEntry,
    pub tun_b: RouteEntry,
}

impl Default for RoutingTable {
    fn default() -> Self {
        RoutingTable { tun_a: RouteEntry::default(), tun_b: RouteEntry::default() }
    }
}

/// Compute routing tables for all routers in the fabric.
/// Returns a map from RouterId to its RoutingTable.
pub fn compute_routing(fabric: &Fabric, ingress_a: RouterId, ingress_b: RouterId) -> HashMap<RouterId, RoutingTable> {
    // Helper to compute distances from a source router using Dijkstra.
    fn distances_from(fabric: &Fabric, src: &RouterId) -> HashMap<petgraph::prelude::NodeIndex, u32> {
        let src_idx = fabric.router_index.get(src).expect("ingress router missing in fabric");
        dijkstra(&fabric.graph, *src_idx, None, |e| {
            let w = e.weight().cfg.delay_ms;
            if w == 0 { 1 } else { w }
        })
    }

    let dist_a = distances_from(fabric, &ingress_a);
    let dist_b = distances_from(fabric, &ingress_b);

    let mut tables = HashMap::new();

    for (router_id, &node_idx) in &fabric.router_index {
        // ----- TUN A -----
        let total_cost_a = *dist_a.get(&node_idx).unwrap_or(&u32::MAX);
        let next_hop_a = if router_id == &ingress_a {
            router_id.clone()
        } else {
            let mut chosen: Option<RouterId> = None;
            for edge in fabric.graph.edges(node_idx) {
                let neighbor_idx = edge.target();
                let w = edge.weight().cfg.delay_ms;
                let neighbor_dist = *dist_a.get(&neighbor_idx).unwrap_or(&u32::MAX);
                if neighbor_dist != u32::MAX && total_cost_a != u32::MAX && neighbor_dist + if w == 0 {1} else {w} == total_cost_a {
                    chosen = Some(fabric.graph[neighbor_idx].id.clone());
                    break;
                }
            }
            chosen.unwrap_or_else(|| router_id.clone())
        };

        // ----- TUN B -----
        let total_cost_b = *dist_b.get(&node_idx).unwrap_or(&u32::MAX);
        let next_hop_b = if router_id == &ingress_b {
            router_id.clone()
        } else {
            let mut chosen: Option<RouterId> = None;
            for edge in fabric.graph.edges(node_idx) {
                let neighbor_idx = edge.target();
                let w = edge.weight().cfg.delay_ms;
                let neighbor_dist = *dist_b.get(&neighbor_idx).unwrap_or(&u32::MAX);
                if neighbor_dist != u32::MAX && total_cost_b != u32::MAX && neighbor_dist + if w == 0 {1} else {w} == total_cost_b {
                    chosen = Some(fabric.graph[neighbor_idx].id.clone());
                    break;
                }
            }
            chosen.unwrap_or_else(|| router_id.clone())
        };

        tables.insert(
            router_id.clone(),
            RoutingTable {
                tun_a: RouteEntry { next_hop: next_hop_a, total_cost: total_cost_a },
                tun_b: RouteEntry { next_hop: next_hop_b, total_cost: total_cost_b },
            },
        );
    }

    tables
}
