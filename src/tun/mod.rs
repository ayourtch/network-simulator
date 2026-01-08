#![allow(clippy::collapsible_else_if)]
// src/tun/mod.rs

use crate::config::SimulatorConfig;
use crate::config::VirtualCustomerConfig;
use crate::packet::{calculate_ipv4_checksum, parse, PacketMeta};
use crate::processor::{process_packet, process_packet_multi};
use crate::routing::multipath::MultiPathTable;
use crate::routing::RoutingTable;
use crate::routing::{compute_multi_path_routing, compute_routing, Destination};
use crate::topology::router::RouterId;
use crate::topology::Fabric;

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

use futures::future::pending; // used for idle handling when no virtual‑customer interval is configured
use ipnet::IpNet;
use tokio::select;
use tokio::signal;
use tun_rs::AsyncDevice;

use tracing::{debug, error, info, warn};

fn ip_in_prefix(ip: &std::net::IpAddr, prefix: &str) -> bool {
    if prefix.is_empty() {
        return false;
    }
    match prefix.parse::<IpNet>() {
        Ok(net) => net.contains(ip),
        Err(_) => false,
    }
}

/// Mock TUN handling.
/// If `packet_file` is specified in the config, each line of the file should contain a hex-encoded
/// packet (e.g., "45000014..." without spaces). The function reads the file, parses each packet,
/// and forwards it through the fabric using the appropriate routing tables.
/// In a full implementation this would interact with real TUN devices.
// Helper function to generate a virtual‑customer packet
async fn generate_virtual_packet(
    vc: &VirtualCustomerConfig,
    cfg: &SimulatorConfig,
    fabric: &mut Fabric,
    routing_tables: &std::collections::HashMap<RouterId, RoutingTable>,
    multipath_tables: &std::collections::HashMap<RouterId, MultiPathTable>,
    ingress_a: &RouterId,
    ingress_b: &RouterId,
) {
    // Determine ingress based on CIDR prefixes using the module‑level ip_in_prefix
    if let (Some(src_str), Some(dst_str)) = (&vc.src_ip, &vc.dst_ip) {
        // IPv4 handling
        if let (Ok(src_ip), Ok(dst_ip)) = (
            src_str.parse::<std::net::Ipv4Addr>(),
            dst_str.parse::<std::net::Ipv4Addr>(),
        ) {
            let mut raw = vec![0u8; 20];
            raw[0] = 0x45;
            raw[1] = 0;
            raw[2] = 0;
            raw[3] = 20;
            raw[4] = 0;
            raw[5] = 0;
            raw[6] = 0;
            raw[7] = 0;
            raw[8] = 64;
            raw[9] = vc.protocol.unwrap_or(6);
            raw[10] = 0;
            raw[11] = 0;
            raw[12..16].copy_from_slice(&src_ip.octets());
            raw[16..20].copy_from_slice(&dst_ip.octets());
            if let Some(sz) = vc.size {
                raw.extend(vec![0u8; sz]);
            }
            let checksum = calculate_ipv4_checksum(&raw);
            raw[10] = (checksum >> 8) as u8;
            raw[11] = (checksum & 0xFF) as u8;
            let packet = PacketMeta {
                src_ip: std::net::IpAddr::V4(src_ip),
                dst_ip: std::net::IpAddr::V4(dst_ip),
                src_port: 0,
                dst_port: 0,
                protocol: raw[9],
                ttl: 64,
                raw,
            };
            let (ingress, destination) = if let Some(ref inject) = cfg.packet_inject_tun {
                match inject.as_str() {
                    "tun_a" => (ingress_a.clone(), Destination::TunB),
                    "tun_b" => (ingress_b.clone(), Destination::TunA),
                    _ => (ingress_a.clone(), Destination::TunB),
                }
            } else {
                if ip_in_prefix(&packet.src_ip, &cfg.tun_ingress.tun_a_prefix) {
                    (ingress_a.clone(), Destination::TunB)
                } else if ip_in_prefix(&packet.src_ip, &cfg.tun_ingress.tun_b_prefix) {
                    (ingress_b.clone(), Destination::TunA)
                } else {
                    (ingress_a.clone(), Destination::TunB)
                }
            };
            debug!(
                "Processing virtual customer IPv4 packet at ingress {}",
                ingress.0
            );
            if cfg.enable_multipath {
                process_packet_multi(fabric, multipath_tables, ingress, packet, destination).await;
            } else {
                process_packet(fabric, routing_tables, ingress, packet, destination).await;
            }
        } else if let (Ok(src_ip), Ok(dst_ip)) = (
            src_str.parse::<std::net::Ipv6Addr>(),
            dst_str.parse::<std::net::Ipv6Addr>(),
        ) {
            // IPv6 handling
            let mut raw = vec![0u8; 40];
            raw[0] = 0x60;
            let payload_len_pos = 4;
            raw[payload_len_pos] = 0;
            raw[payload_len_pos + 1] = 0;
            raw[6] = vc.protocol.unwrap_or(6);
            raw[7] = 64;
            raw[8..24].copy_from_slice(&src_ip.octets());
            raw[24..40].copy_from_slice(&dst_ip.octets());
            if let Some(sz) = vc.size {
                raw.extend(vec![0u8; sz]);
            }
            let payload_len = (raw.len() - 40) as u16;
            raw[payload_len_pos] = (payload_len >> 8) as u8;
            raw[payload_len_pos + 1] = (payload_len & 0xFF) as u8;
            let packet = PacketMeta {
                src_ip: std::net::IpAddr::V6(src_ip),
                dst_ip: std::net::IpAddr::V6(dst_ip),
                src_port: 0,
                dst_port: 0,
                protocol: raw[6],
                ttl: raw[7],
                raw,
            };
            let (ingress, destination) = if let Some(ref inject) = cfg.packet_inject_tun {
                match inject.as_str() {
                    "tun_a" => (ingress_a.clone(), Destination::TunB),
                    "tun_b" => (ingress_b.clone(), Destination::TunA),
                    _ => (ingress_a.clone(), Destination::TunB),
                }
            } else {
                if ip_in_prefix(&packet.src_ip, &cfg.tun_ingress.tun_a_prefix) {
                    (ingress_a.clone(), Destination::TunB)
                } else if ip_in_prefix(&packet.src_ip, &cfg.tun_ingress.tun_b_prefix) {
                    (ingress_b.clone(), Destination::TunA)
                } else {
                    (ingress_a.clone(), Destination::TunB)
                }
            };
            debug!(
                "Processing virtual customer IPv6 packet at ingress {}",
                ingress.0
            );
            if cfg.enable_multipath {
                process_packet_multi(fabric, multipath_tables, ingress, packet, destination).await;
            } else {
                process_packet(fabric, routing_tables, ingress, packet, destination).await;
            }
        } else {
            warn!(
                "Invalid IPs in virtual_customer: src='{}', dst='{}'",
                src_str, dst_str
            );
        }
    } else {
        warn!("virtual_customer missing src_ip or dst_ip");
    }
}

pub async fn start(
    cfg: &SimulatorConfig,
    fabric: &mut Fabric,
) -> Result<(), Box<dyn std::error::Error>> {
    // ip_in_prefix helper defined at module level above
    // Optional interval for periodic virtual‑customer packet generation
    let mut _vc_interval: Option<tokio::time::Interval> = None;
    // If real TUN devices are not configured (empty address) and no mock or virtual customer handling, skip TUN handling.
    if cfg.interfaces.real_tun_a.address.is_empty()
        && cfg.interfaces.real_tun_b.address.is_empty()
        && cfg.packet_file.is_none()
        && cfg.packet_files.is_none()
        && cfg.virtual_customer.is_none()
    {
        // No real TUN to handle and nothing to mock; nothing to do.
        return Ok(());
    }
    // Compute routing tables once.
    // Compute routing tables once.
    let ingress_a = RouterId(cfg.tun_ingress.tun_a_ingress.clone());
    let ingress_b = RouterId(cfg.tun_ingress.tun_b_ingress.clone());
    let routing_tables = compute_routing(fabric, ingress_a.clone(), ingress_b.clone());
    let multipath_tables = if cfg.enable_multipath {
        compute_multi_path_routing(fabric, ingress_a.clone(), ingress_b.clone())
    } else {
        std::collections::HashMap::new()
    };
    // ip_in_prefix function defined above; vc_interval already declared above

    // Virtual customer packet generation (burst)
    if let Some(vc) = &cfg.virtual_customer {
        // Initial burst based on rate (default 1)
        let packet_count = vc.rate.unwrap_or(1) as usize;
        for _ in 0..packet_count {
            generate_virtual_packet(
                vc,
                cfg,
                fabric,
                &routing_tables,
                &multipath_tables,
                &ingress_a,
                &ingress_b,
            )
            .await;
        }
        // Setup periodic interval if rate > 0
        if let Some(rate) = vc.rate {
            if rate > 0 {
                _vc_interval = Some(tokio::time::interval(std::time::Duration::from_secs_f64(
                    1.0 / rate as f64,
                )));
            }
        }
    }
    if let Some(ref path) = cfg.packet_file {
        info!("Reading mock packets from {}", path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        // Prepare output file to capture packets exiting the mock TUN.
        let out_path = format!("{}_out.txt", path);
        let mut out_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&out_path)
            .map_err(|e| format!("Failed to open output file {}: {}", out_path, e))?;
        for (idx, line_res) in reader.lines().enumerate() {
            let raw_line = line_res?;
            let line = raw_line.trim();
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
            // Determine injection direction: use explicit config if provided, otherwise infer from IP.
            let (ingress, destination) = if let Some(ref inject) = cfg.packet_inject_tun {
                match inject.as_str() {
                    "tun_a" => (ingress_a.clone(), Destination::TunB),
                    "tun_b" => (ingress_b.clone(), Destination::TunA),
                    _ => {
                        // CIDR based injection direction detection
                        if ip_in_prefix(&packet.src_ip, &cfg.tun_ingress.tun_a_prefix) {
                            (ingress_a.clone(), Destination::TunB)
                        } else if ip_in_prefix(&packet.src_ip, &cfg.tun_ingress.tun_b_prefix) {
                            (ingress_b.clone(), Destination::TunA)
                        } else if ip_in_prefix(&packet.src_ip, &cfg.tun_ingress.tun_a_ipv6_prefix) {
                            (ingress_a.clone(), Destination::TunB)
                        } else if ip_in_prefix(&packet.src_ip, &cfg.tun_ingress.tun_b_ipv6_prefix) {
                            (ingress_b.clone(), Destination::TunA)
                        } else {
                            // Default fallback to original heuristic (10.)
                            if packet.src_ip.to_string().starts_with("10.") {
                                (ingress_a.clone(), Destination::TunB)
                            } else {
                                (ingress_b.clone(), Destination::TunA)
                            }
                        }
                    }
                }
            } else {
                // CIDR based injection direction detection
                if ip_in_prefix(&packet.src_ip, &cfg.tun_ingress.tun_a_prefix) {
                    (ingress_a.clone(), Destination::TunB)
                } else if ip_in_prefix(&packet.src_ip, &cfg.tun_ingress.tun_b_prefix) {
                    (ingress_b.clone(), Destination::TunA)
                } else if ip_in_prefix(&packet.src_ip, &cfg.tun_ingress.tun_a_ipv6_prefix) {
                    (ingress_a.clone(), Destination::TunB)
                } else if ip_in_prefix(&packet.src_ip, &cfg.tun_ingress.tun_b_ipv6_prefix) {
                    (ingress_b.clone(), Destination::TunA)
                } else {
                    // Default fallback: no CIDR prefix matched. Log warning and default to ingress A.
                    warn!(
                        "No CIDR prefix matched for source IP {}. Defaulting to ingress A.",
                        packet.src_ip
                    );
                    (ingress_a.clone(), Destination::TunB)
                }
            };
            debug!(
                "Processing mock packet {} at ingress {}",
                idx + 1,
                ingress.0
            );
            let processed = if cfg.enable_multipath {
                process_packet_multi(fabric, &multipath_tables, ingress, packet, destination).await
            } else {
                process_packet(fabric, &routing_tables, ingress, packet, destination).await
            };
            // Write processed packet raw bytes as hex to output file.
            let hex_str = hex::encode(&processed.raw);
            if let Err(e) = writeln!(out_file, "{}", hex_str) {
                error!("Failed to write processed packet to output file: {}", e);
            }
        }
    } else if let Some(ref files) = cfg.packet_files {
        // Multiple packet files handling.
        let injects = cfg.packet_inject_tuns.clone().unwrap_or_default();
        for (i, path) in files.iter().enumerate() {
            info!("Reading mock packets from {}", path);
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let out_path = format!("{}_out.txt", path);
            let mut out_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&out_path)
                .map_err(|e| format!("Failed to open output file {}: {}", out_path, e))?;
            let inject_opt = injects.get(i).cloned();
            for (idx, line_res) in reader.lines().enumerate() {
                let raw_line = line_res?;
                let line = raw_line.trim();
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
                let (ingress, destination) = if let Some(ref inject) = inject_opt {
                    match inject.as_str() {
                        "tun_a" => (ingress_a.clone(), Destination::TunB),
                        "tun_b" => (ingress_b.clone(), Destination::TunA),
                        _ => {
                            // CIDR based detection for ambiguous injection direction in multi‑file handling
                            let src_ip = &packet.src_ip;
                            if ip_in_prefix(src_ip, &cfg.tun_ingress.tun_a_prefix) {
                                (ingress_a.clone(), Destination::TunB)
                            } else if ip_in_prefix(src_ip, &cfg.tun_ingress.tun_b_prefix) {
                                (ingress_b.clone(), Destination::TunA)
                            } else if ip_in_prefix(src_ip, &cfg.tun_ingress.tun_a_ipv6_prefix) {
                                (ingress_a.clone(), Destination::TunB)
                            } else if ip_in_prefix(src_ip, &cfg.tun_ingress.tun_b_ipv6_prefix) {
                                (ingress_b.clone(), Destination::TunA)
                            } else {
                                // Default fallback to original heuristic (10.)
                                if src_ip.to_string().starts_with("10.") {
                                    (ingress_a.clone(), Destination::TunB)
                                } else {
                                    (ingress_b.clone(), Destination::TunA)
                                }
                            }
                        }
                    }
                } else {
                    // No explicit injection, use CIDR prefixes similar to single‑file handling.
                    let src_ip = &packet.src_ip;
                    if ip_in_prefix(src_ip, &cfg.tun_ingress.tun_a_prefix) {
                        (ingress_a.clone(), Destination::TunB)
                    } else if ip_in_prefix(src_ip, &cfg.tun_ingress.tun_b_prefix) {
                        (ingress_b.clone(), Destination::TunA)
                    } else if ip_in_prefix(src_ip, &cfg.tun_ingress.tun_a_ipv6_prefix) {
                        (ingress_a.clone(), Destination::TunB)
                    } else if ip_in_prefix(src_ip, &cfg.tun_ingress.tun_b_ipv6_prefix) {
                        (ingress_b.clone(), Destination::TunA)
                    } else {
                        // Default fallback: no CIDR prefix matched. Log warning and default to ingress A.
                        warn!(
                            "No CIDR prefix matched for source IP {}. Defaulting to ingress A.",
                            src_ip
                        );
                        (ingress_a.clone(), Destination::TunB)
                    }
                };
                let processed = if cfg.enable_multipath {
                    process_packet_multi(fabric, &multipath_tables, ingress, packet, destination)
                        .await
                } else {
                    process_packet(fabric, &routing_tables, ingress, packet, destination).await
                };
                let hex_str = hex::encode(&processed.raw);
                if let Err(e) = writeln!(out_file, "{}", hex_str) {
                    error!("Failed to write processed packet to output file: {}", e);
                }
            }
        }
    }

    // If mock packet handling was performed, skip real TUN handling.
    if cfg.packet_file.is_some() || cfg.packet_files.is_some() {
        return Ok(());
    }
    // Open two real TUN devices (real_tun_a and real_tun_b).
    // Packets read from tun_a are considered ingress_a and sent out via tun_b, and vice versa.

    // Helper to create async TUN device from config using tun-rs.
    fn create_async_tun(
        name: &str,
        addr_str: &str,
        netmask_str: &str,
    ) -> Result<AsyncDevice, String> {
        use tun_rs::DeviceBuilder;

        // Parse address, supporting both IPv4 and IPv6.
        let ip_addr = addr_str
            .parse::<std::net::IpAddr>()
            .map_err(|_| format!("Invalid IP address for TUN {}: '{}'", name, addr_str))?;

        let mut builder = DeviceBuilder::new().name(name);

        match ip_addr {
            std::net::IpAddr::V4(v4) => {
                // Parse netmask as prefix length or dotted notation
                let prefix: u8 = if netmask_str.is_empty() {
                    24
                } else if let Ok(p) = netmask_str.parse::<u8>() {
                    p
                } else if let Ok(mask) = netmask_str.parse::<std::net::Ipv4Addr>() {
                    // Convert netmask to prefix length
                    mask.octets().iter().map(|b| b.count_ones() as u8).sum()
                } else {
                    24
                };
                builder = builder.ipv4(v4, prefix, None);
            }
            std::net::IpAddr::V6(v6) => {
                let prefix: u8 = if netmask_str.is_empty() {
                    64
                } else {
                    netmask_str.parse::<u8>().unwrap_or(64)
                };
                builder = builder.ipv6(v6, prefix);
            }
        }

        // Build the async TUN device
        builder.mtu(1500).build_async().map_err(|e| e.to_string())
    }

    let async_dev_a = match create_async_tun(
        &cfg.interfaces.real_tun_a.name,
        &cfg.interfaces.real_tun_a.address,
        &cfg.interfaces.real_tun_a.netmask,
    ) {
        Ok(dev) => dev,
        Err(e) => {
            if e.contains("Operation not permitted")
                || e.contains("EPERM")
                || e.contains("permission")
            {
                warn!("Skipping real TUN A due to insufficient permissions: {}", e);
                return Ok(());
            } else {
                return Err(e.into());
            }
        }
    };
    let async_dev_b = match create_async_tun(
        &cfg.interfaces.real_tun_b.name,
        &cfg.interfaces.real_tun_b.address,
        &cfg.interfaces.real_tun_b.netmask,
    ) {
        Ok(dev) => dev,
        Err(e) => {
            if e.contains("Operation not permitted")
                || e.contains("EPERM")
                || e.contains("permission")
            {
                warn!("Skipping real TUN B due to insufficient permissions: {}", e);
                return Ok(());
            } else {
                return Err(e.into());
            }
        }
    };

    let mut buf_a = vec![0u8; cfg.simulation.mtu as usize + 100];
    let mut buf_b = vec![0u8; cfg.simulation.mtu as usize + 100];
    // Graceful shutdown signal future.
    let shutdown_signal = signal::ctrl_c();
    // Pin the shutdown future for select! macro.
    tokio::pin!(shutdown_signal);
    loop {
        debug!("Entering dual‑TUN processing loop");
        select! {
            // Periodic virtual‑customer generation tick
            _ = async {
                if let Some(ref mut int) = _vc_interval {
                    int.tick().await;
                } else {
                    pending::<()>().await;
                }
            } => {
                if let Some(vc) = &cfg.virtual_customer {
                    generate_virtual_packet(vc, cfg, fabric, &routing_tables, &multipath_tables, &ingress_a, &ingress_b).await;
                }
            },

            // Read from TUN A, forward to B.
            read_res = async_dev_a.recv(&mut buf_a) => {
                debug!("Read result from TUN A: {:?}", read_res);
                let n = match read_res {
                    Ok(0) => { debug!("Read zero bytes from TUN device, continuing"); continue; },
                    Ok(n) => n,
                    Err(e) => {
                        error!("Error reading from TUN A: {}", e);
                        break;
                    }
                };
                // tun-rs provides consistent IP packets across platforms (no 4-byte header)
                let packet_slice = &buf_a[..n];
                let packet = match parse(packet_slice) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Failed to parse packet from TUN A: {}", e);
                        continue;
                    }
                };
                let (ingress, destination) = (ingress_a.clone(), Destination::TunB);
                debug!("Processing packet from TUN A on ingress {}", ingress.0);
                let processed = if cfg.enable_multipath {
                    process_packet_multi(fabric, &multipath_tables, ingress.clone(), packet, destination).await
                } else {
                    process_packet(fabric, &routing_tables, ingress.clone(), packet, destination).await
                };
                // tun-rs handles the packet format consistently, so we just send the raw IP packet
                if let Err(e) = async_dev_b.send(&processed.raw).await {
                    let err_msg = e.to_string();
                    if err_msg.contains("seek on unseekable file") {
                        warn!("Write to TUN B failed (unseekable), likely due to mock mode; ignoring.");
                    } else {
                        error!("Failed to write packet to TUN B: {}", e);
                        break;
                    }
                }
            }
            // Read from TUN B, forward to A.
            read_res = async_dev_b.recv(&mut buf_b) => {
                debug!("Read result from TUN B: {:?}", read_res);
                let n = match read_res {
                    Ok(0) => { debug!("Read zero bytes from TUN device, continuing"); continue; },
                    Ok(n) => { debug!("Read {} bytes from B", n); n }
                    Err(e) => {
                        error!("Error reading from TUN B: {}", e);
                        break;
                    }
                };
                // tun-rs provides consistent IP packets across platforms (no 4-byte header)
                let packet_slice = &buf_b[..n];
                let packet = match parse(packet_slice) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Failed to parse packet from TUN B: {}", e);
                        continue;
                    }
                };
                let (ingress, destination) = (ingress_b.clone(), Destination::TunA);
                debug!("Processing packet from TUN B on ingress {}", ingress.0);
                let processed = if cfg.enable_multipath {
                    process_packet_multi(fabric, &multipath_tables, ingress.clone(), packet, destination).await
                } else {
                    process_packet(fabric, &routing_tables, ingress.clone(), packet, destination).await
                };
                // tun-rs handles the packet format consistently
                if let Err(e) = async_dev_a.send(&processed.raw).await {
                    let err_msg = e.to_string();
                    if err_msg.contains("seek on unseekable file") {
                        warn!("Write to TUN A failed (unseekable), likely due to mock mode; ignoring.");
                    } else {
                        error!("Failed to write packet to TUN A: {}", e);
                        break;
                    }
                }
            }
            _ = &mut shutdown_signal => {
                info!("Shutdown signal received, exiting dual‑TUN loop");
                // TUN interfaces will be cleaned up when async_dev_a and async_dev_b are dropped
                break;
            }
        }
    }
    Ok(())
}
