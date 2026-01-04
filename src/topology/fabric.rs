// src/topology/fabric.rs

use petgraph::graph::{NodeIndex, UnGraph};
use tracing::info;
use std::collections::HashMap;
use crate::topology::{Router, RouterId, Link, LinkId, LinkConfig, RouterStats};
use petgraph::graph::EdgeIndex;
// duplicate NodeIndex import removed

#[derive(Debug)]
pub struct Fabric {
    pub graph: UnGraph<Router, Link>,
    pub router_index: HashMap<RouterId, NodeIndex>,
    pub link_index: HashMap<LinkId, EdgeIndex>,
}

impl Fabric {
    // existing methods...
    /// Return a vector of references to all links incident to the given router.
    pub fn incident_links(&self, router_id: &RouterId) -> Vec<&Link> {
        let mut result = Vec::new();
        if let Some(&node_idx) = self.router_index.get(router_id) {
            for edge_ref in self.graph.edges(node_idx) {
                result.push(edge_ref.weight());
            }
        }
        result
    }

    /// Print statistics for all routers.
    pub fn print_statistics(&self) {
        for (router_id, node_idx) in &self.router_index {
            if let Some(router) = self.graph.node_weight(*node_idx) {
                let stats = &router.stats;
                info!("Router {}: recv={}, fwd={}, icmp={}", router_id.0, stats.packets_received, stats.packets_forwarded, stats.icmp_generated);
            }
        }
    }

    /// Return a map of router IDs to their statistics.
    pub fn get_statistics(&self) -> std::collections::HashMap<RouterId, RouterStats> {
        let mut map = std::collections::HashMap::new();
        for (router_id, node_idx) in &self.router_index {
            if let Some(router) = self.graph.node_weight(*node_idx) {
                map.insert(router_id.clone(), router.stats.clone());
            }
        }
        map
    }
}

impl Fabric {
    pub fn new() -> Self {
        Self {
            graph: UnGraph::new_undirected(),
            router_index: HashMap::new(),
            link_index: HashMap::new(),
        }
    }

    pub fn add_router(&mut self, router: Router) {
        // Validate router id format
        router.id.validate().expect("Invalid router id");
        let idx = self.graph.add_node(router.clone());
        self.router_index.insert(router.id.clone(), idx);
    }

    pub fn add_link(&mut self, a: &RouterId, b: &RouterId, cfg: LinkConfig) {
        // Ensure both routers exist
        let a_idx = self.router_index.get(a).expect("Router A missing");
        let b_idx = self.router_index.get(b).expect("Router B missing");
        let id = LinkId::new(a.clone(), b.clone());
        let link = Link { id: id.clone(), cfg, counter: std::sync::atomic::AtomicU64::new(0) };
        let edge_idx = self.graph.add_edge(*a_idx, *b_idx, link);
        self.link_index.insert(id, edge_idx);
    }
}
