// src/topology/router.rs

use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RouterId(pub String);

impl RouterId {
    pub fn validate(&self) -> Result<(), String> {
        // Enforce router IDs within 6x6 grid: Rx0y0 .. Rx5y5
        let re = regex::Regex::new(r"^Rx[0-5]y[0-5]$").unwrap();
        if re.is_match(&self.0) {
            Ok(())
        } else {
            Err(format!(
                "Invalid router id '{}', expected Rx[0-5]y[0-5]",
                self.0
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Router {
    pub id: RouterId,
    pub routing: crate::routing::RoutingTable,
    pub stats: RouterStats,
}

impl Router {
    pub fn increment_received(&mut self) {
        self.stats.packets_received += 1;
    }
    pub fn increment_forwarded(&mut self) {
        self.stats.packets_forwarded += 1;
    }
    pub fn increment_icmp(&mut self) {
        self.stats.icmp_generated += 1;
    }
}

#[derive(Debug, Default, Clone)]
pub struct RouterStats {
    pub packets_received: u64,
    pub packets_forwarded: u64,
    pub icmp_generated: u64,
}
