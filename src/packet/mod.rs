// src/packet/mod.rs

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Calculate IPv4 header checksum (RFC 791).
pub fn calculate_ipv4_checksum(header: &[u8]) -> u16 {
    if header.len() < 20 {
        return 0;
    }
    // Header length in bytes = IHL * 4
    let ihl = (header[0] & 0x0F) as usize * 4;
    let header = &header[..ihl.min(header.len())];
    let mut sum: u32 = 0;
    for i in (0..header.len()).step_by(2) {
        // Skip checksum field at bytes 10-11
        if i == 10 {
            continue;
        }
        let word = if i + 1 < header.len() {
            u16::from_be_bytes([header[i], header[i + 1]])
        } else {
            // Odd number of bytes, pad with zero
            u16::from_be_bytes([header[i], 0])
        };
        sum += word as u32;
    }
    // Add carries
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

/// Update the IPv4 header checksum in-place after modifications.
pub fn update_ipv4_checksum(packet: &mut [u8]) {
    if packet.len() < 20 {
        return;
    }
    // Zero out checksum field
    packet[10] = 0;
    packet[11] = 0;
    let checksum = calculate_ipv4_checksum(packet);
    packet[10] = (checksum >> 8) as u8;
    packet[11] = (checksum & 0xFF) as u8;
}


/// Minimal packet metadata used by the simulator.
#[derive(Debug, Clone)]
pub struct PacketMeta {
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8, // TCP=6, UDP=17, ICMP=1, ICMPv6=58
    pub ttl: u8,
    // Original raw bytes of the packet, preserved for write‑back.
    pub raw: Vec<u8>,
}

impl PacketMeta {
    /// Decrement the TTL (or Hop Limit) of the packet.
    ///
    /// This updates both the `ttl` field and the corresponding byte in the raw packet data.
    /// Returns an error if the raw packet is malformed or the TTL is already zero.
    pub fn decrement_ttl(&mut self) -> Result<(), &'static str> {
        if self.ttl == 0 {
            return Err("TTL already zero");
        }
        // Decrement the logical TTL value.
        self.ttl = self.ttl.saturating_sub(1);
        // Update the raw byte depending on IP version.
        match self.src_ip {
            IpAddr::V4(_) => {
                // IPv4 TTL is at offset 8. If raw data is present and long enough, update it.
                if self.raw.len() > 8 {
                    let ttl_byte = self.raw[8];
                    self.raw[8] = ttl_byte.saturating_sub(1);
                    // Recalculate IPv4 header checksum after TTL change.
                    update_ipv4_checksum(&mut self.raw);
                }
            }
            IpAddr::V6(_) => {
                // IPv6 Hop Limit is at offset 7.
                if self.raw.len() > 7 {
                    let hl = self.raw[7];
                    self.raw[7] = hl.saturating_sub(1);
                }
            }
        }
        Ok(())
    }
}

/// Stub parser – in the full version this would decode raw bytes using the `pnet` crate.
pub fn parse(data: &[u8]) -> Result<PacketMeta, &'static str> {
    // Minimal IPv4 header parsing (no options).
    if data.len() < 20 {
        return Err("packet too short for IPv4 header");
    }
    let version_ihl = data[0];
    let version = version_ihl >> 4;
    if version == 4 {
        // IPv4 parsing
        let ihl = (version_ihl & 0x0F) as usize * 4;
        if ihl < 20 {
            return Err("invalid IHL");
        }
        let total_len = u16::from_be_bytes([data[2], data[3]]) as usize;
        if data.len() < total_len {
            return Err("packet length less than total_len");
        }
        let ttl = data[8];
        let protocol = data[9];
        let src_ip = Ipv4Addr::new(data[12], data[13], data[14], data[15]);
        let dst_ip = Ipv4Addr::new(data[16], data[17], data[18], data[19]);
        // Extract ports for TCP/UDP if possible
        let (src_port, dst_port) = if (protocol == 6 || protocol == 17) && data.len() >= ihl + 4 {
            let sp = u16::from_be_bytes([data[ihl], data[ihl + 1]]);
            let dp = u16::from_be_bytes([data[ihl + 2], data[ihl + 3]]);
            (sp, dp)
        } else {
            (0, 0)
        };
        Ok(PacketMeta {
            src_ip: IpAddr::V4(src_ip),
            dst_ip: IpAddr::V4(dst_ip),
            src_port,
            dst_port,
            protocol,
            ttl,
            raw: data.to_vec(),
        })
    } else if version == 6 {
        // IPv6 parsing with optional Hop-by-Hop extension header handling
        if data.len() < 40 {
            return Err("packet too short for IPv6 header");
        }
        // Next Header field at offset 6, Hop Limit at offset 7
        let mut next_header = data[6];
        let hop_limit = data[7];
        let src_ip = Ipv6Addr::new(
            u16::from_be_bytes([data[8], data[9]]),
            u16::from_be_bytes([data[10], data[11]]),
            u16::from_be_bytes([data[12], data[13]]),
            u16::from_be_bytes([data[14], data[15]]),
            u16::from_be_bytes([data[16], data[17]]),
            u16::from_be_bytes([data[18], data[19]]),
            u16::from_be_bytes([data[20], data[21]]),
            u16::from_be_bytes([data[22], data[23]]),
        );
        let dst_ip = Ipv6Addr::new(
            u16::from_be_bytes([data[24], data[25]]),
            u16::from_be_bytes([data[26], data[27]]),
            u16::from_be_bytes([data[28], data[29]]),
            u16::from_be_bytes([data[30], data[31]]),
            u16::from_be_bytes([data[32], data[33]]),
            u16::from_be_bytes([data[34], data[35]]),
            u16::from_be_bytes([data[36], data[37]]),
            u16::from_be_bytes([data[38], data[39]]),
        );
        // Determine transport offset, handling possible Hop-by-Hop extension header (Next Header == 0).
        let mut transport_offset = 40usize;
        if next_header == 0 {
            // Need at least two bytes for Hop-by-Hop header.
            if data.len() < transport_offset + 2 {
                return Err("packet too short for Hop-by-Hop header");
            }
            // The real next header is the first byte of the Hop-by-Hop header.
            let real_next = data[transport_offset];
            // Header Extension Length: length of the header in 8-byte units, not counting the first 8 bytes.
            let hdr_len = data[transport_offset + 1] as usize;
            let ext_len = (hdr_len + 1) * 8; // total size of the Hop-by-Hop header.
            next_header = real_next;
            transport_offset += ext_len;
        }
        // Extract ports for TCP/UDP if possible (offset after IPv6 (and any Hop-by-Hop) header)
        let (src_port, dst_port) = if (next_header == 6 || next_header == 17) && data.len() >= transport_offset + 4 {
            let sp = u16::from_be_bytes([data[transport_offset], data[transport_offset + 1]]);
            let dp = u16::from_be_bytes([data[transport_offset + 2], data[transport_offset + 3]]);
            (sp, dp)
        } else {
            (0, 0)
        };
        Ok(PacketMeta {
            src_ip: IpAddr::V6(src_ip),
            dst_ip: IpAddr::V6(dst_ip),
            src_port,
            dst_port,
            protocol: next_header,
            ttl: hop_limit,
            raw: data.to_vec(),
        })
    } else {
        Err("unsupported IP version")
    }
}

