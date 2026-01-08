// src/topology/router.rs

use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::net::{Ipv4Addr, Ipv6Addr};

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

    /// Parse the grid coordinates from a RouterId (e.g., "Rx2y3" -> (2, 3))
    pub fn grid_position(&self) -> Option<(u8, u8)> {
        if self.0.len() >= 5 && self.0.starts_with("Rx") && self.0.contains('y') {
            let x = self.0.chars().nth(2)?.to_digit(10)? as u8;
            let y = self.0.chars().nth(4)?.to_digit(10)? as u8;
            Some((x, y))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Router {
    pub id: RouterId,
    /// IPv4 address of this router (used as source for ICMP errors)
    pub ipv4_addr: Ipv4Addr,
    /// IPv6 address of this router (used as source for ICMPv6 errors)
    pub ipv6_addr: Ipv6Addr,
    pub routing: crate::routing::RoutingTable,
    pub stats: RouterStats,
}

impl Router {
    /// Create a new router with automatically generated addresses based on grid position.
    /// IPv4: 10.{100+x}.{y}.1 (e.g., Rx2y3 -> 10.102.3.1)
    /// IPv6: fd00::{x}:{y} (e.g., Rx2y3 -> fd00::2:3)
    pub fn new(id: RouterId) -> Self {
        let (ipv4_addr, ipv6_addr) = Self::generate_addresses(&id);
        Router {
            id,
            ipv4_addr,
            ipv6_addr,
            routing: crate::routing::RoutingTable::default(),
            stats: RouterStats::default(),
        }
    }

    /// Generate deterministic IPv4 and IPv6 addresses from a RouterId.
    pub fn generate_addresses(id: &RouterId) -> (Ipv4Addr, Ipv6Addr) {
        if let Some((x, y)) = id.grid_position() {
            // IPv4: 10.{100+x}.{y}.1 - gives us 10.100.0.1 to 10.105.5.1
            let ipv4 = Ipv4Addr::new(10, 100 + x, y, 1);
            // IPv6: fd00::{x}:{y} - private address space
            let ipv6 = Ipv6Addr::new(0xfd00, 0, 0, 0, 0, 0, x as u16, y as u16);
            (ipv4, ipv6)
        } else {
            // Fallback for invalid router IDs
            (Ipv4Addr::new(0, 0, 0, 0), Ipv6Addr::UNSPECIFIED)
        }
    }
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
    pub fn increment_lost(&mut self) {
        self.stats.packets_lost += 1;
    }

    /// Get the router's IPv4 address
    pub fn ipv4_addr(&self) -> Ipv4Addr {
        self.ipv4_addr
    }

    /// Get the router's IPv6 address
    pub fn ipv6_addr(&self) -> Ipv6Addr {
        self.ipv6_addr
    }
}

#[derive(Debug, Default, Clone)]
pub struct RouterStats {
    pub packets_received: u64,
    pub packets_forwarded: u64,
    pub packets_lost: u64,
    pub icmp_generated: u64,
}
