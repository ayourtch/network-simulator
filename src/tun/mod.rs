// src/tun/mod.rs

use crate::config::SimulatorConfig;
use crate::topology::Fabric;
use tracing::info;

/// Placeholder for TUN interface handling.
/// In the full implementation this will create the two TUN devices, spawn async tasks
/// that read packets, assign a customer ID, and inject them into the fabric at the
/// configured ingress routers.
pub async fn start(_cfg: &SimulatorConfig, _fabric: &mut Fabric) -> Result<(), Box<dyn std::error::Error>> {
    info!("TUN handling stub started (no actual I/O yet)");
    Ok(())
}
