// src/tun/mod.rs

use crate::config::SimulatorConfig;
use crate::topology::Fabric;
use crate::routing::{compute_routing, compute_multi_path_routing, Destination};
use crate::processor::{process_packet, process_packet_multi};
use crate::topology::router::RouterId;
use crate::packet::parse;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tokio::select;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::signal;

use tun::platform::Device as TunDevice;
use tun::{Configuration};
use std::os::unix::io::{AsRawFd, FromRawFd};
use tracing::{info, error, debug, warn};
use std::net::Ipv4Addr;

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
    if let Some(ref path) = cfg.packet_file {
        info!("Reading mock packets from {}", path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        for (idx, line_res) in reader.lines().enumerate() {
            let line = line_res?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
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
            let (ingress, destination) = if packet.src_ip.to_string().starts_with("10.") {
                (ingress_a.clone(), Destination::TunB)
            } else {
                (ingress_b.clone(), Destination::TunA)
            };
            debug!("Processing mock packet {} at ingress {}", idx + 1, ingress.0);
            if cfg.enable_multipath {
                let _ = process_packet_multi(fabric, &multipath_tables, ingress, packet, destination).await;
            } else {
                let _ = process_packet(fabric, &routing_tables, ingress, packet, destination).await;
            }
        }
    } else {
        // Open a TUN device using the first configured interface name.
        let tun_name = &cfg.interfaces.real_tun.name;
        info!("Opening real TUN device {}", tun_name);
        let mut config = Configuration::default();
        let addr: Ipv4Addr = cfg.interfaces.real_tun.address.parse().unwrap_or(Ipv4Addr::new(10,0,0,1));
        let netmask: Ipv4Addr = cfg.interfaces.real_tun.netmask.parse().unwrap_or(Ipv4Addr::new(255,255,255,0));
        config.name(tun_name).address(addr).netmask(netmask).up();
        let dev = TunDevice::new(&config)
            .map_err(|e| format!("Failed to create TUN device: {}", e))?;
        let std_file = unsafe { std::fs::File::from_raw_fd(dev.as_raw_fd()) };
        let mut async_dev = tokio::fs::File::from_std(std_file);
        let mut buf = vec![0u8; cfg.simulation.mtu as usize];
        // Graceful shutdown signal future.
        let mut shutdown_signal = signal::ctrl_c();
        // Pin the shutdown future for select! macro.
        tokio::pin!(shutdown_signal);
        loop {
            select! {
                read_res = async_dev.read(&mut buf) => {
                    let n = match read_res {
                        Ok(0) => break, // EOF
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
                    let (ingress, destination) = if packet.src_ip.to_string().starts_with("10.") {
                        (ingress_a.clone(), Destination::TunB)
                    } else {
                        (ingress_b.clone(), Destination::TunA)
                    };
                    debug!("Processing packet from TUN on ingress {}", ingress.0);
                    let processed_packet = if cfg.enable_multipath {
                        process_packet_multi(fabric, &multipath_tables, ingress.clone(), packet, destination).await
                    } else {
                        process_packet(fabric, &routing_tables, ingress.clone(), packet, destination).await
                    };
                    if let Err(e) = async_dev.write_all(&processed_packet.raw).await {
                        error!("Failed to write packet back to TUN device: {}", e);
                        break;
                    }
                }
                _ = &mut shutdown_signal => {
                    info!("Shutdown signal received, exiting TUN loop");
                    break;
                }
            }
        }
    }
    Ok(())
}
