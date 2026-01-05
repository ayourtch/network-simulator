// src/config.rs

/// Configuration for the network simulator. Includes a flag to enable multipath routing.

use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use crate::topology::router::RouterId;

#[derive(Debug, Deserialize)]
pub struct SimulatorConfig {
    #[serde(default)]
    pub simulation: SimulationConfig,
    #[serde(default)]
    pub interfaces: InterfacesConfig,
    #[serde(rename = "tun_ingress", default)]
    pub tun_ingress: TunIngressConfig,
    #[serde(default)]
    pub topology: TopologyConfig,
    #[serde(default = "default_enable_multipath")]
    pub enable_multipath: bool,
    #[serde(default)]
    pub packet_file: Option<String>, // Optional path to a file containing hex‑encoded mock packets for the TUN interface (overridden by CLI flag)
    #[serde(default)]
    pub packet_files: Option<Vec<String>>, // Optional multiple packet files for mock TUNs
    #[serde(default)]
    pub packet_inject_tun: Option<String>, // Optional: "tun_a" or "tun_b" to force injection direction for single file
    #[serde(default)]
    pub packet_inject_tuns: Option<Vec<String>>, // Optional injection directions per file
    #[serde(default)]
    pub virtual_customer: Option<VirtualCustomerConfig>, // Optional virtual customer configuration


}

impl SimulatorConfig {
    pub fn validate(&self) -> Result<(), String> {
        // First, validate packet injection configuration consistency before other checks,
        // so that errors about mutually exclusive fields or missing files are reported early.
        // Ensure mutually exclusive use of single and multiple packet files.
        if self.packet_file.is_some() && self.packet_files.is_some() {
            return Err("Both 'packet_file' and 'packet_files' are set; only one may be specified".to_string());
        }
        // Ensure injection direction specified only when corresponding packet file(s) are set.
        if self.packet_inject_tun.is_some() && self.packet_file.is_none() {
            return Err("'packet_inject_tun' specified without a 'packet_file'".to_string());
        }
        if self.packet_inject_tuns.is_some() && self.packet_files.is_none() {
            return Err("'packet_inject_tuns' specified without 'packet_files'".to_string());
        }
        // Validate injection direction values.
        if let Some(ref dir) = self.packet_inject_tun {
            if dir != "tun_a" && dir != "tun_b" {
                return Err(format!("Invalid packet_inject_tun value '{}', expected 'tun_a' or 'tun_b'", dir));
            }
        }
        if let Some(ref dirs) = self.packet_inject_tuns {
            for d in dirs {
                if d != "tun_a" && d != "tun_b" {
                    return Err(format!("Invalid packet_inject_tuns value '{}', expected 'tun_a' or 'tun_b'", d));
                }
            }
        }
        // Validate packet injection configuration consistency (matching lengths).
        if let (Some(files), Some(injects)) = (&self.packet_files, &self.packet_inject_tuns) {
            if files.len() != injects.len() {
                return Err(format!(
                    "Number of packet files ({}) does not match number of injection directions ({})",
                    files.len(),
                    injects.len()
                ));
            }
            // Ensure packet_files is not empty when provided
            if files.is_empty() {
                return Err("packet_files list cannot be empty".to_string());
            }
        }
        // Validate existence of packet file(s) if provided.
        use std::path::Path;
use std::net::Ipv4Addr;
        if let Some(ref path) = self.packet_file {
            if !Path::new(path).exists() {
                return Err(format!("packet_file '{}' does not exist", path));
            }
        }
        if let Some(ref files) = self.packet_files {
            for p in files {
                if !Path::new(p).exists() {
                    return Err(format!("packet file '{}' does not exist", p));
                }
            }
        }
        // Ensure link definitions are unique regardless of direction.
        let mut seen: HashSet<(String, String)> = HashSet::new();
        // Collect existing router IDs for reference validation
        let router_ids: HashSet<String> = self.topology.routers.keys().cloned().collect();
        // Ensure at least one router is defined in the topology
        if router_ids.is_empty() {
            return Err("Topology must define at least one router".to_string());
        }
        for link_name in self.topology.links.keys() {
            let parts: Vec<&str> = link_name.split('_').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid link name '{}', expected 'A_B' format", link_name));
            }
            let a = parts[0].to_string();
            let b = parts[1].to_string();
            // Validate that both routers exist
            if !router_ids.contains(&a) {
                return Err(format!("Link '{}' references unknown router '{}'", link_name, a));
            }
            if !router_ids.contains(&b) {
                return Err(format!("Link '{}' references unknown router '{}'", link_name, b));
            }
            // Normalize order for undirected comparison
            let key = if a < b { (a.clone(), b.clone()) } else { (b.clone(), a.clone()) };
            if seen.contains(&key) {
                return Err(format!(
                    "Duplicate bidirectional link detected: '{}' and its opposite already defined",
                    link_name
                ));
            }
            seen.insert(key);
        }
        // Validate ingress routers exist in topology
        if !router_ids.contains(&self.tun_ingress.tun_a_ingress) {
            return Err(format!("Ingress router '{}' not found in topology", self.tun_ingress.tun_a_ingress));
        }
        if !router_ids.contains(&self.tun_ingress.tun_b_ingress) {
            return Err(format!("Ingress router '{}' not found in topology", self.tun_ingress.tun_b_ingress));
        }
        // Also ensure ingress IDs are valid format
        RouterId(self.tun_ingress.tun_a_ingress.clone()).validate()?;
        RouterId(self.tun_ingress.tun_b_ingress.clone()).validate()?;
        // Validate real TUN address and netmask fields (support both IPv4 and IPv6)
        let rt_a = &self.interfaces.real_tun_a;
        let rt_b = &self.interfaces.real_tun_b;
        for (label, cfg) in &[ ("real_tun_a", rt_a), ("real_tun_b", rt_b) ] {
            // Parse address as generic IpAddr
            let ip_addr = cfg.address.parse::<std::net::IpAddr>()
                .map_err(|_| format!("Invalid IP address for {}.address: '{}'", label, cfg.address))?;
            match ip_addr {
                std::net::IpAddr::V4(_v4) => {
                    // IPv4: validate netmask as IPv4 address
                    if cfg.netmask.parse::<Ipv4Addr>().is_err() {
                        return Err(format!("Invalid IPv4 netmask for {}.netmask: '{}'", label, cfg.netmask));
                    }
                },
                std::net::IpAddr::V6(_v6) => {
                    // IPv6: treat netmask as prefix length (0-128). If empty, default to 64.
                    if cfg.netmask.is_empty() {
                        // empty netmask is acceptable, will use default /64 in tun creation
                    } else {
                        let prefix: u8 = cfg.netmask.parse::<u8>()
                            .map_err(|_| format!("Invalid IPv6 netmask/prefix for {}.netmask: '{}' (expected 0-128)", label, cfg.netmask))?;
                        if prefix > 128 {
                            return Err(format!("IPv6 netmask/prefix out of range for {}.netmask: '{}' (max 128)", label, cfg.netmask));
                        }
                    }
                },
            }
        }
        Ok(())
    }
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            simulation: SimulationConfig::default(),
            interfaces: InterfacesConfig::default(),
            tun_ingress: TunIngressConfig::default(),
            topology: TopologyConfig::default(),
            enable_multipath: false,
            packet_file: None,
            packet_files: None,
            packet_inject_tun: None,
            packet_inject_tuns: None,
            virtual_customer: None,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct SimulationConfig {
    #[serde(default = "default_mtu")]
    pub mtu: u32,
    #[serde(default)]
    pub seed: Option<u64>,
}

fn default_enable_multipath() -> bool { false }
fn default_mtu() -> u32 { 1500 }

#[derive(Debug, Deserialize, Default)]
pub struct InterfacesConfig {
    #[serde(default = "default_tun_a")]
    pub tun_a: String,
    #[serde(default = "default_tun_b")]
    pub tun_b: String,
    #[serde(default = "default_real_tun_a")]
    pub real_tun_a: RealTunConfig,
    #[serde(default = "default_real_tun_b")]
    pub real_tun_b: RealTunConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct RealTunConfig {
    #[serde(default = "default_real_tun_name")]
    pub name: String,
    #[serde(default = "default_real_tun_addr")]
    pub address: String,
    #[serde(default = "default_real_tun_netmask")]
    pub netmask: String,
}

fn default_tun_a() -> String { "tunA".to_string() }
fn default_tun_b() -> String { "tunB".to_string() }

fn default_real_tun_name() -> String { "tun0".to_string() }
fn default_real_tun_addr() -> String { "10.0.0.1".to_string() }
fn default_real_tun_netmask() -> String { "255.255.255.0".to_string() }
fn default_real_tun_a() -> RealTunConfig { RealTunConfig { name: "tun0a".to_string(), address: "10.0.0.1".to_string(), netmask: "255.255.255.0".to_string() } }

fn default_real_tun_b() -> RealTunConfig { RealTunConfig { name: "tun0b".to_string(), address: "10.0.1.1".to_string(), netmask: "255.255.255.0".to_string() } }

#[derive(Debug, Deserialize, Default)]
pub struct TunIngressConfig {
    #[serde(default = "default_ingress_a")]
    pub tun_a_ingress: String,
    #[serde(default = "default_ingress_b")]
    pub tun_b_ingress: String,
    #[serde(default = "default_prefix_a")]
    pub tun_a_prefix: String,
    #[serde(default = "default_prefix_b")]
    pub tun_b_prefix: String,
    #[serde(default = "default_ipv6_prefix_a")]
    pub tun_a_ipv6_prefix: String,
    #[serde(default = "default_ipv6_prefix_b")]
    pub tun_b_ipv6_prefix: String,
}

fn default_ingress_a() -> String { "Rx0y0".to_string() }
fn default_ingress_b() -> String { "Rx5y5".to_string() }
fn default_prefix_a() -> String { "10.".to_string() }
fn default_prefix_b() -> String { "".to_string() }
fn default_ipv6_prefix_a() -> String { "".to_string() }
fn default_ipv6_prefix_b() -> String { "".to_string() }

#[derive(Debug, Deserialize, Default)]
pub struct VirtualCustomerConfig {
    // Example fields for a virtual traffic generator
    pub src_ip: Option<String>,
    pub dst_ip: Option<String>,
    pub protocol: Option<u8>, // e.g., 6 for TCP, 17 for UDP
    pub size: Option<usize>, // packet size in bytes
    pub rate: Option<u64>, // packets per second
}

#[derive(Debug, Deserialize, Default)]
pub struct TopologyConfig {
    #[serde(default)]
    pub routers: HashMap<String, toml::Value>, // empty tables just indicate existence
    #[serde(default)]
    pub links: HashMap<String, super::topology::link::LinkConfig>,
}
