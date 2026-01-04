// src/routing/multipath.rs

use serde::{Serialize, Deserialize};
use crate::topology::{Fabric, RouterId};
use crate::routing::{RouteEntry};
use std::collections::HashMap;
use petgraph::algo::dijkstra;
use petgraph::visit::EdgeRef;

/// Multi‑path routing table containing all equal‑cost next hops for each destination.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MultiPathTable {
    pub tun_a: Vec<RouteEntry>,
    pub tun_b: Vec<RouteEntry>,
}

/// Compute multi‑path routing tables for all routers.
pub fn compute_multi_path_routing(
    fabric: &Fabric,
    ingress_a: RouterId,
    ingress_b: RouterId,
) -> HashMap<RouterId, MultiPathTable> {
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
    let mut tables: HashMap<RouterId, MultiPathTable> = HashMap::new();

    for (router_id, &node_idx) in &fabric.router_index {
        // Tun A entries (traffic from ingress A towards B) use distances from ingress B.
        let mut entries_a = Vec::new();
        let mut min_cost_a = u32::MAX;
        for edge in fabric.graph.edges(node_idx) {
            let neighbor_idx = if edge.source() == node_idx { edge.target() } else { edge.source() };
            let w = edge.weight().cfg.delay_ms;
            if let Some(&neighbor_dist) = dist_b.get(&neighbor_idx) {
                if neighbor_dist != u32::MAX {
                    let cost = neighbor_dist + if w == 0 { 1 } else { w };
                    if cost < min_cost_a {
                        min_cost_a = cost;
                        entries_a.clear();
                        entries_a.push(RouteEntry {
                            next_hop: fabric.graph[neighbor_idx].id.clone(),
                            total_cost: cost,
                        });
                    } else if cost == min_cost_a {
                        entries_a.push(RouteEntry {
                            next_hop: fabric.graph[neighbor_idx].id.clone(),
                            total_cost: cost,
                        });
                    }
                }
            }
        }
        // Tun B entries (traffic from ingress B towards A) use distances from ingress A.
        let mut entries_b = Vec::new();
        let mut min_cost_b = u32::MAX;
        for edge in fabric.graph.edges(node_idx) {
            let neighbor_idx = if edge.source() == node_idx { edge.target() } else { edge.source() };
            let w = edge.weight().cfg.delay_ms;
            if let Some(&neighbor_dist) = dist_a.get(&neighbor_idx) {
                if neighbor_dist != u32::MAX {
                    let cost = neighbor_dist + if w == 0 { 1 } else { w };
                    if cost < min_cost_b {
                        min_cost_b = cost;
                        entries_b.clear();
                        entries_b.push(RouteEntry {
                            next_hop: fabric.graph[neighbor_idx].id.clone(),
                            total_cost: cost,
                        });
                    } else if cost == min_cost_b {
                        entries_b.push(RouteEntry {
                            next_hop: fabric.graph[neighbor_idx].id.clone(),
                            total_cost: cost,
                        });
                    }
                }
            }
        }
        tables.insert(
            router_id.clone(),
            MultiPathTable {
                tun_a: entries_a,
                tun_b: entries_b,
            },
        );
    }
    tables
}
