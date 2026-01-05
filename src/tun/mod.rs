// src/tun/mod.rs

use crate::config::SimulatorConfig;
use crate::topology::Fabric;
use crate::routing::{compute_routing, compute_multi_path_routing, Destination};
use crate::processor::{process_packet, process_packet_multi};
use crate::topology::router::RouterId;
use crate::packet::{calculate_ipv4_checksum, PacketMeta, parse};
use crate::routing::RoutingTable;
use crate::routing::multipath::MultiPathTable;
use crate::config::VirtualCustomerConfig;

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;
use std::net::Ipv4Addr;
use tokio::select;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::signal;
use ipnet::IpNet;
use futures::future::pending;

fn ip_in_prefix(ip: &std::net::IpAddr, prefix: &str) -> bool {
    if prefix.is_empty() { return false; }
    match prefix.parse::<IpNet>() {
        Ok(net) => net.contains(ip),
        Err(_) => false,
    }
}


use tun::platform::Device as TunDevice;
use tun::{Configuration};


use tracing::{info, error, debug, warn};


/// Mock TUN handling.
/// If `packet_file` is specified in the config, each line of the file should contain a hex-encoded
/// packet (e.g., "45000014..." without spaces). The function reads the file, parses each packet,
/// and forwards it through the fabric using the appropriate routing tables.
/// In a full implementation this would interact with real TUN devices.
pub async fn start(cfg: &SimulatorConfig, fabric: &mut Fabric) -> Result<(), Box<dyn std::error::Error>> {
    // ip_in_prefix helper defined at module level above
    // Optional interval for periodic virtual‑customer packet generation
    let mut vc_interval: Option<tokio::time::Interval> = None;
    // Compute routing tables once.
    let ingress_a = RouterId(cfg.tun_ingress.tun_a_ingress.clone());
    let ingress_b = RouterId(cfg.tun_ingress.tun_b_ingress.clone());
    let routing_tables = compute_routing(&fabric, ingress_a.clone(), ingress_b.clone());
    let multipath_tables = if cfg.enable_multipath {
        compute_multi_path_routing(&fabric, ingress_a.clone(), ingress_b.clone())
    } else {
        std::collections::HashMap::new()
    };
    // ip_in_prefix function defined above; vc_interval already declared above


    // Virtual customer packet generation (burst)
    if let Some(vc) = &cfg.virtual_customer {
        // Initial burst based on rate (default 1)
        let packet_count = vc.rate.unwrap_or(1) as usize;
        for _ in 0..packet_count {
            generate_virtual_packet(vc, cfg, fabric, &routing_tables, &multipath_tables, &ingress_a, &ingress_b).await;
        }
        // Setup periodic interval if rate > 0
        if let Some(rate) = vc.rate {
            if rate > 0 {
                vc_interval = Some(tokio::time::interval(std::time::Duration::from_secs_f64(1.0 / rate as f64)));
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
                    // Default fallback to original heuristic (10.)
                    if packet.src_ip.to_string().starts_with("10.") {
                        (ingress_a.clone(), Destination::TunB)
                    } else {
                        (ingress_b.clone(), Destination::TunA)
                    }
                }
            };
            debug!("Processing mock packet {} at ingress {}", idx + 1, ingress.0);
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
                        // Default fallback to original heuristic (10.)
                        if src_ip.to_string().starts_with("10.") {
                            (ingress_a.clone(), Destination::TunB)
                        } else {
                            (ingress_b.clone(), Destination::TunA)
                        }
                    }
                };
                let processed = if cfg.enable_multipath {
                    process_packet_multi(fabric, &multipath_tables, ingress, packet, destination).await
                } else {
                    process_packet(fabric, &routing_tables, ingress, packet, destination).await
                };
                let hex_str = hex::encode(&processed.raw);
                if let Err(e) = writeln!(out_file, "{}", hex_str) {
                    error!("Failed to write processed packet to output file: {}", e);
                }
            }
        }

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
            raw[2] = 0; raw[3] = 20;
            raw[4] = 0; raw[5] = 0;
            raw[6] = 0; raw[7] = 0;
            raw[8] = 64;
            raw[9] = vc.protocol.unwrap_or(6);
            raw[10] = 0; raw[11] = 0;
            raw[12..16].copy_from_slice(&src_ip.octets());
            raw[16..20].copy_from_slice(&dst_ip.octets());
            if let Some(sz) = vc.size { raw.extend(vec![0u8; sz]); }
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
            debug!("Processing virtual customer IPv4 packet at ingress {}", ingress.0);
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
            raw[payload_len_pos] = 0; raw[payload_len_pos+1] = 0;
            raw[6] = vc.protocol.unwrap_or(6);
            raw[7] = 64;
            raw[8..24].copy_from_slice(&src_ip.octets());
            raw[24..40].copy_from_slice(&dst_ip.octets());
            if let Some(sz) = vc.size { raw.extend(vec![0u8; sz]); }
            let payload_len = (raw.len() - 40) as u16;
            raw[payload_len_pos] = (payload_len >> 8) as u8;
            raw[payload_len_pos+1] = (payload_len & 0xFF) as u8;
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
            debug!("Processing virtual customer IPv6 packet at ingress {}", ingress.0);
            if cfg.enable_multipath {
                process_packet_multi(fabric, multipath_tables, ingress, packet, destination).await;
            } else {
                process_packet(fabric, routing_tables, ingress, packet, destination).await;
            }
        } else {
            warn!("Invalid IPs in virtual_customer: src='{}', dst='{}'", src_str, dst_str);
        }
    } else {
        warn!("virtual_customer missing src_ip or dst_ip");
    }
}
// Open two real TUN devices (real_tun_a and real_tun_b).
// Packets read from tun_a are considered ingress_a and sent out via tun_b, and vice versa.

// Helper to create async TUN device from config.
fn create_async_tun(name: &str, addr_str: &str, netmask_str: &str) -> Result<tokio::fs::File, String> {
    let mut cfg = Configuration::default();
    // Parse address, supporting both IPv4 and IPv6.
    let ip_addr = addr_str.parse::<std::net::IpAddr>()
        .map_err(|_| format!("Invalid IP address for TUN {}: '{}'", name, addr_str))?;
    match ip_addr {
        std::net::IpAddr::V4(v4) => {
            // IPv4: use provided netmask (fallback defaults).
            let netmask = netmask_str.parse::<Ipv4Addr>()
                .unwrap_or(Ipv4Addr::new(255, 255, 255, 0));
            cfg.name(name).address(v4).netmask(netmask).up();
        },
        std::net::IpAddr::V6(v6) => {
            // IPv6: apply prefix length (netmask_str) if provided, default /64.
            cfg.name(name).address(std::net::IpAddr::V6(v6)).up();
            // After interface is up, configure IPv6 address with prefix using system command (Linux).
            // If netmask_str is empty, default to 64.
            let prefix = if netmask_str.is_empty() {
                64u8
            } else {
                netmask_str.parse::<u8>().map_err(|_| format!("Invalid IPv6 prefix '{}', expected 0-128", netmask_str))?
            };
            #[cfg(target_os = "linux")] {
                use std::process::Command;
                let addr_with_prefix = format!("{}/{}", addr_str, prefix);
                let _ = Command::new("ip")
                    .args(["-6", "addr", "add", &addr_with_prefix, "dev", name])
                    .status();
            }
        },
    }
    use std::os::fd::{FromRawFd, IntoRawFd};
    let dev = TunDevice::new(&cfg)
        .map_err(|e| format!("Failed to create TUN device {}: {}", name, e))?;
    let raw_fd = dev.into_raw_fd();
    let std_file = unsafe { std::fs::File::from_raw_fd(raw_fd) };
    Ok(tokio::fs::File::from_std(std_file))
}


let async_dev_a = create_async_tun(&cfg.interfaces.real_tun_a.name, &cfg.interfaces.real_tun_a.address, &cfg.interfaces.real_tun_a.netmask)?;
let async_dev_b = create_async_tun(&cfg.interfaces.real_tun_b.name, &cfg.interfaces.real_tun_b.address, &cfg.interfaces.real_tun_b.netmask)?;
let mut async_dev_a = async_dev_a;
let mut async_dev_b = async_dev_b;
let mut buf_a = vec![0u8; cfg.simulation.mtu as usize];
let mut buf_b = vec![0u8; cfg.simulation.mtu as usize];
// Graceful shutdown signal future.
let shutdown_signal = signal::ctrl_c();
// Pin the shutdown future for select! macro.
tokio::pin!(shutdown_signal);
loop {
    select! {
        // Periodic virtual‑customer generation tick
        _ = async {
            if let Some(ref mut int) = vc_interval {
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
        read_res = async_dev_a.read(&mut buf_a) => {
            let n = match read_res {
                Ok(0) => break, // EOF
                Ok(n) => n,
                Err(e) => {
                    error!("Error reading from TUN A: {}", e);
                    break;
                }
            };
            let packet_bytes = &buf_a[..n];
            let packet = match parse(packet_bytes) {
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
            if let Err(e) = async_dev_b.write_all(&processed.raw).await {
                error!("Failed to write packet to TUN B: {}", e);
                break;
            }
        }
        // Read from TUN B, forward to A.
        read_res = async_dev_b.read(&mut buf_b) => {
            let n = match read_res {
                Ok(0) => break, // EOF
                Ok(n) => n,
                Err(e) => {
                    error!("Error reading from TUN B: {}", e);
                    break;
                }
            };
            let packet_bytes = &buf_b[..n];
            let packet = match parse(packet_bytes) {
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
            if let Err(e) = async_dev_a.write_all(&processed.raw).await {
                error!("Failed to write packet to TUN A: {}", e);
                break;
            }
        }
        _ = &mut shutdown_signal => {
            info!("Shutdown signal received, exiting dual‑TUN loop");
            // Bring down the TUN interfaces to avoid leaving them up after exit.
            #[cfg(target_os = "linux")] {
                use std::process::Command;
                let _ = Command::new("ip")
                    .args(["link", "set", "dev", &cfg.interfaces.real_tun_a.name, "down"])
                    .status();
                let _ = Command::new("ip")
                    .args(["link", "set", "dev", &cfg.interfaces.real_tun_b.name, "down"])
                    .status();
            }
            break;
        }
    }
}

    }
    Ok(())
}
