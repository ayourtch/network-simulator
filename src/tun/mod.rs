// src/tun/mod.rs

use crate::config::SimulatorConfig;
use crate::topology::Fabric;
use crate::routing::{compute_routing, compute_multi_path_routing, Destination};
use crate::processor::{process_packet, process_packet_multi};
use crate::topology::router::RouterId;
use crate::packet::parse;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tun::platform::Device as TunDevice;
use tun::{Configuration};
use std::os::unix::io::{AsRawFd, FromRawFd};
use tracing::{info, error, debug, warn};
use std::net::Ipv4Addr;
// use crate::simulation; // unused currently

/// Mock TUN handling.
/// If `packet_file` is specified in the config, each line of the file should contain a hex-encoded
/// packet (e.g., "45000014..." without spaces). The function reads the file, parses each packet,
/// and forwards it through the fabric using the appropriate routing tables.
/// In a full implementation this would interact with real TUN devices.
pub async fn start(cfg: &SimulatorConfig, fabric: &mut Fabric) -> Result<(), Box<dyn std::error::Error>> {
    // Compute routing tables once.
    let ingress_a = RouterId(cfg.tun_ingress.tun_a_ingress.clone());
    let ingress_b = RouterId(cfg.tun_ingress.tun_b_ingress.clone());
    let routing_tables = compute_routing(&fabric, ingress_a.clone(), ingress_b.clone());
    let multipath_tables = if cfg.enable_multipath {
        compute_multi_path_routing(&fabric, ingress_a.clone(), ingress_b.clone())
    } else {
        std::collections::HashMap::new()
    };

    // If a packet file is provided, process each line as a packet.
    if let Some(ref path) = cfg.packet_file { // If the file is empty, no packets will be processed; the simulator will still run successfully
        info!("Reading mock packets from {}", path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        for (idx, line_res) in reader.lines().enumerate() {
            let line = line_res?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                // Skip empty lines and comment lines starting with '#'
                continue;
            }
            // Convert hex string to bytes.
            let bytes = match hex::decode(line) {
                Ok(b) => b,
                Err(e) => {
                    warn!("Failed to decode hex on line {}: {}", idx + 1, e);
                    continue;
                }
            };
            let packet = match parse(&bytes) {
                Ok(p) => p,
                Err(e) => {
                    error!("Failed to parse packet on line {}: {}", idx + 1, e);
                    continue;
                }
            };
            // Choose ingress based on source IP (simplified heuristic).
            let (ingress, destination) = if packet.src_ip.to_string().starts_with("10.") {
                (ingress_a.clone(), Destination::TunB)
            } else {
                (ingress_b.clone(), Destination::TunA)
            };
            debug!("Processing mock packet {} at ingress {}", idx + 1, ingress.0);
            if cfg.enable_multipath {
                process_packet_multi(fabric, &multipath_tables, ingress, packet, destination).await;
            } else {
                // Use normal packet processing which will forward and simulate link.
                process_packet(fabric, &routing_tables, ingress, packet, destination).await;
            }
        }
    } else {
        // Open a TUN device using the first configured interface name.
        let tun_name = &cfg.interfaces.real_tun.name;
        info!("Opening real TUN device {}", tun_name);
        let mut config = Configuration::default();
        // Parse address and netmask, fallback to defaults on parse error
        let addr: Ipv4Addr = cfg.interfaces.real_tun.address.parse().unwrap_or(Ipv4Addr::new(10,0,0,1));
        let netmask: Ipv4Addr = cfg.interfaces.real_tun.netmask.parse().unwrap_or(Ipv4Addr::new(255,255,255,0));
        config
            .name(tun_name)
            .address(addr)
            .netmask(netmask)
            .up();
        let dev = TunDevice::new(&config)
            .map_err(|e| format!("Failed to create TUN device: {}", e))?;
        // Convert raw fd to std::fs::File, then to async.
        let std_file = unsafe { std::fs::File::from_raw_fd(dev.as_raw_fd()) };
        let mut async_dev = tokio::fs::File::from_std(std_file);
        let mut buf = vec![0u8; cfg.simulation.mtu as usize];
        loop {
            let n = match async_dev.read(&mut buf).await {
                Ok(0) => {
                    // EOF – break loop
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    error!("Error reading from TUN device: {}", e);
                    break;
                }
            };
            let packet_bytes = &buf[..n];
            let packet = match parse(packet_bytes) {
                Ok(p) => p,
                Err(e) => {
                    error!("Failed to parse packet from TUN: {}", e);
                    continue;
                }
            };
            // Choose ingress based on source IP heuristic (same as mock).
            let (ingress, destination) = if packet.src_ip.to_string().starts_with("10.") {
                (ingress_a.clone(), Destination::TunB)
            } else {
                (ingress_b.clone(), Destination::TunA)
            };
            debug!("Processing packet from TUN on ingress {}", ingress.0);
            if cfg.enable_multipath {
                process_packet_multi(fabric, &multipath_tables, ingress, packet, destination).await;
            } else {
                process_packet(fabric, &routing_tables, ingress, packet, destination).await;
            }
            // Write packet back to TUN device (preserving any modifications to raw bytes)
            if let Err(e) = async_dev.write_all(&packet.raw).await {
                error!("Failed to write packet back to TUN device: {}", e);
                break;
            }

        }
    }
    Ok(())
}
