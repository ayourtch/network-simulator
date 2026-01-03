// src/topology/link.rs

use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU64;
use crate::topology::router::RouterId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinkId {
    pub a: RouterId,
    pub b: RouterId,
}

impl LinkId {
    pub fn new(r1: RouterId, r2: RouterId) -> Self {
        if r1.0 <= r2.0 {
            Self { a: r1, b: r2 }
        } else {
            Self { a: r2, b: r1 }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkConfig {
    #[serde(default)]
    pub mtu: Option<u32>,
    #[serde(default = "default_delay")]
    pub delay_ms: u32,
    #[serde(default = "default_jitter")]
    pub jitter_ms: u32,
    #[serde(default = "default_loss")]
    pub loss_percent: f32,
    #[serde(default)]
    pub load_balance: bool,
}

fn default_delay() -> u32 { 0 }
fn default_jitter() -> u32 { 0 }
fn default_loss() -> f32 { 0.0 }

#[derive(Debug)]
pub struct Link {
    pub id: LinkId,
    pub cfg: LinkConfig,
    pub counter: AtomicU64,
}
