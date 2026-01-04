/// Configuration for the network simulator. Includes a flag to enable multipath routing.


use serde::Deserialize;
use std::collections::HashMap;

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
}

fn default_tun_a() -> String { "tunA".to_string() }
fn default_tun_b() -> String { "tunB".to_string() }

#[derive(Debug, Deserialize, Default)]
pub struct TunIngressConfig {
    #[serde(default = "default_ingress_a")]
    pub tun_a_ingress: String,
    #[serde(default = "default_ingress_b")]
    pub tun_b_ingress: String,
}

fn default_ingress_a() -> String { "Rx0y0".to_string() }
fn default_ingress_b() -> String { "Rx5y5".to_string() }

#[derive(Debug, Deserialize, Default)]
pub struct TopologyConfig {
    #[serde(default)]
    pub routers: HashMap<String, toml::Value>, // empty tables just indicate existence
    #[serde(default)]
    pub links: HashMap<String, super::topology::link::LinkConfig>,
}
