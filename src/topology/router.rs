// src/topology/router.rs

use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RouterId(pub String);

impl RouterId {
    pub fn validate(&self) -> Result<(), String> {
        let re = regex::Regex::new(r"^Rx\d+y\d+$").unwrap();
        if re.is_match(&self.0) {
            Ok(())
        } else {
            Err(format!("Invalid router id '{}', expected RxXyY", self.0))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Router {
    pub id: RouterId,
    pub routing: crate::routing::RoutingTable,
    pub stats: RouterStats,
}

#[derive(Debug, Default, Clone)]
pub struct RouterStats {
    pub packets_received: u64,
    pub packets_forwarded: u64,
    pub icmp_generated: u64,
}
