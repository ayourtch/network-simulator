# Plan 4: Packet Processing and Forwarding Engine

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the core packet processing logic including IPv4/IPv6 parsing, TTL handling, and basic forwarding through the virtual router fabric.

**Architecture:** Parse IP headers to extract version, TTL/hop limit, source/destination addresses. Decrement TTL on each hop. Forward packets based on routing table lookups. Detect TTL expiration and packet size issues.

**Tech Stack:** pnet_packet for packet parsing, or custom parsing with byte manipulation

---

## Task 1: Add Packet Processing Dependencies

**Files:**
- Modify: `Cargo.toml`
- Create: `src/packet/mod.rs`
- Modify: `src/lib.rs`

**Step 1: Add packet parsing dependencies**

Update `Cargo.toml`:
```toml
[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
thiserror = "1.0"
anyhow = "1.0"
regex = "1.10"
clap = { version = "4.4", features = ["derive"] }
tun = { version = "0.6", features = ["async"] }
bytes = "1.5"
pnet_packet = "0.34"

[dev-dependencies]
tempfile = "3.8"
```

**Step 2: Create packet module**

Create `src/packet/mod.rs`:
```rust
pub mod parser;
pub mod ipv4;
pub mod ipv6;

pub use parser::{PacketInfo, PacketError, parse_packet};
```

Update `src/lib.rs`:
```rust
pub mod config;
pub mod topology;
pub mod tun;
pub mod packet;
```

**Step 3: Build to verify**

Run:
```bash
cargo build
```

Expected: Success

**Step 4: Commit dependency addition**

```bash
git add Cargo.toml src/packet/mod.rs src/lib.rs
git commit -m "feat: add packet processing module and dependencies"
```

---

## Task 2: Implement IPv4 Packet Parsing

**Files:**
- Create: `src/packet/parser.rs`
- Create: `src/packet/ipv4.rs`
- Create: `tests/packet_test.rs`

**Step 1: Write test for IPv4 parsing**

Create `tests/packet_test.rs`:
```rust
use netsimulator::packet::{parse_packet, PacketInfo};

#[test]
fn test_parse_ipv4_packet() {
    // Minimal IPv4 packet: version=4, IHL=5, total_length=20, TTL=64
    let packet = vec![
        0x45, 0x00, 0x00, 0x14, // Version, IHL, TOS, Total Length
        0x00, 0x00, 0x00, 0x00, // ID, Flags, Fragment Offset
        0x40, 0x00, 0x00, 0x00, // TTL=64, Protocol, Checksum
        0xC0, 0xA8, 0x01, 0x01, // Source IP: 192.168.1.1
        0xC0, 0xA8, 0x01, 0x02, // Dest IP: 192.168.1.2
    ];

    let result = parse_packet(&packet);
    assert!(result.is_ok());

    let info = result.unwrap();
    assert_eq!(info.ip_version(), 4);
    assert_eq!(info.ttl(), 64);
    assert_eq!(info.packet_len(), 20);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_parse_ipv4_packet
```

Expected: FAIL with "unresolved import"

**Step 3: Implement PacketError and PacketInfo**

Create `src/packet/parser.rs`:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PacketError {
    #[error("Packet too small: {0} bytes")]
    TooSmall(usize),

    #[error("Invalid IP version: {0}")]
    InvalidVersion(u8),

    #[error("Invalid packet: {0}")]
    Invalid(String),

    #[error("TTL exceeded")]
    TtlExceeded,
}

#[derive(Debug, Clone)]
pub enum PacketInfo {
    V4(Ipv4Info),
    V6(Ipv6Info),
}

impl PacketInfo {
    pub fn ip_version(&self) -> u8 {
        match self {
            PacketInfo::V4(_) => 4,
            PacketInfo::V6(_) => 6,
        }
    }

    pub fn ttl(&self) -> u8 {
        match self {
            PacketInfo::V4(info) => info.ttl,
            PacketInfo::V6(info) => info.hop_limit,
        }
    }

    pub fn packet_len(&self) -> usize {
        match self {
            PacketInfo::V4(info) => info.total_length as usize,
            PacketInfo::V6(info) => info.payload_length as usize + 40,
        }
    }

    pub fn source_ip(&self) -> String {
        match self {
            PacketInfo::V4(info) => format!("{}.{}.{}.{}",
                info.source[0], info.source[1], info.source[2], info.source[3]),
            PacketInfo::V6(info) => {
                // Simplified IPv6 display
                format!("{:02x}{:02x}:{:02x}{:02x}:...:{:02x}{:02x}",
                    info.source[0], info.source[1],
                    info.source[2], info.source[3],
                    info.source[14], info.source[15])
            }
        }
    }

    pub fn dest_ip(&self) -> String {
        match self {
            PacketInfo::V4(info) => format!("{}.{}.{}.{}",
                info.dest[0], info.dest[1], info.dest[2], info.dest[3]),
            PacketInfo::V6(info) => {
                format!("{:02x}{:02x}:{:02x}{:02x}:...:{:02x}{:02x}",
                    info.dest[0], info.dest[1],
                    info.dest[2], info.dest[3],
                    info.dest[14], info.dest[15])
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ipv4Info {
    pub version: u8,
    pub ihl: u8,
    pub total_length: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub source: [u8; 4],
    pub dest: [u8; 4],
}

#[derive(Debug, Clone)]
pub struct Ipv6Info {
    pub version: u8,
    pub payload_length: u16,
    pub next_header: u8,
    pub hop_limit: u8,
    pub source: [u8; 16],
    pub dest: [u8; 16],
}

pub fn parse_packet(packet: &[u8]) -> Result<PacketInfo, PacketError> {
    if packet.is_empty() {
        return Err(PacketError::TooSmall(0));
    }

    let version = packet[0] >> 4;

    match version {
        4 => parse_ipv4(packet),
        6 => parse_ipv6(packet),
        _ => Err(PacketError::InvalidVersion(version)),
    }
}

fn parse_ipv4(packet: &[u8]) -> Result<PacketInfo, PacketError> {
    if packet.len() < 20 {
        return Err(PacketError::TooSmall(packet.len()));
    }

    let version = packet[0] >> 4;
    let ihl = packet[0] & 0x0F;
    let total_length = u16::from_be_bytes([packet[2], packet[3]]);
    let ttl = packet[8];
    let protocol = packet[9];

    let mut source = [0u8; 4];
    let mut dest = [0u8; 4];
    source.copy_from_slice(&packet[12..16]);
    dest.copy_from_slice(&packet[16..20]);

    Ok(PacketInfo::V4(Ipv4Info {
        version,
        ihl,
        total_length,
        ttl,
        protocol,
        source,
        dest,
    }))
}

fn parse_ipv6(packet: &[u8]) -> Result<PacketInfo, PacketError> {
    if packet.len() < 40 {
        return Err(PacketError::TooSmall(packet.len()));
    }

    let version = packet[0] >> 4;
    let payload_length = u16::from_be_bytes([packet[4], packet[5]]);
    let next_header = packet[6];
    let hop_limit = packet[7];

    let mut source = [0u8; 16];
    let mut dest = [0u8; 16];
    source.copy_from_slice(&packet[8..24]);
    dest.copy_from_slice(&packet[24..40]);

    Ok(PacketInfo::V6(Ipv6Info {
        version,
        payload_length,
        next_header,
        hop_limit,
        source,
        dest,
    }))
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_parse_ipv4_packet
```

Expected: PASS

**Step 5: Write test for IPv6 parsing**

Add to `tests/packet_test.rs`:
```rust
#[test]
fn test_parse_ipv6_packet() {
    // Minimal IPv6 packet header (40 bytes)
    let mut packet = vec![0u8; 40];
    packet[0] = 0x60; // Version 6
    packet[4] = 0x00; // Payload length high
    packet[5] = 0x00; // Payload length low
    packet[6] = 0x3B; // Next header (no next header)
    packet[7] = 64;   // Hop limit

    let result = parse_packet(&packet);
    assert!(result.is_ok());

    let info = result.unwrap();
    assert_eq!(info.ip_version(), 6);
    assert_eq!(info.ttl(), 64);  // hop_limit
}
```

**Step 6: Run test**

Run:
```bash
cargo test test_parse_ipv6_packet
```

Expected: PASS

**Step 7: Commit packet parsing**

```bash
git add src/packet/parser.rs tests/packet_test.rs
git commit -m "feat: implement IPv4 and IPv6 packet parsing"
```

---

## Task 3: Implement TTL Decrement and Validation

**Files:**
- Create: `src/packet/ipv4.rs`
- Create: `src/packet/ipv6.rs`
- Modify: `tests/packet_test.rs`

**Step 1: Write test for TTL decrement**

Add to `tests/packet_test.rs`:
```rust
use netsimulator::packet::decrement_ttl;

#[test]
fn test_decrement_ttl_ipv4() {
    let mut packet = vec![
        0x45, 0x00, 0x00, 0x14,
        0x00, 0x00, 0x00, 0x00,
        0x40, 0x00, 0x00, 0x00, // TTL=64
        0xC0, 0xA8, 0x01, 0x01,
        0xC0, 0xA8, 0x01, 0x02,
    ];

    let result = decrement_ttl(&mut packet);
    assert!(result.is_ok());
    assert_eq!(packet[8], 63);  // TTL decremented
}

#[test]
fn test_decrement_ttl_expired() {
    let mut packet = vec![
        0x45, 0x00, 0x00, 0x14,
        0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, // TTL=1
        0xC0, 0xA8, 0x01, 0x01,
        0xC0, 0xA8, 0x01, 0x02,
    ];

    let result = decrement_ttl(&mut packet);
    assert!(result.is_err());  // Should fail with TTL exceeded
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_decrement_ttl_ipv4
```

Expected: FAIL with "unresolved import"

**Step 3: Implement TTL decrement for IPv4**

Create `src/packet/ipv4.rs`:
```rust
use crate::packet::PacketError;

/// Decrement TTL in IPv4 packet and update checksum
pub fn decrement_ttl_v4(packet: &mut [u8]) -> Result<(), PacketError> {
    if packet.len() < 20 {
        return Err(PacketError::TooSmall(packet.len()));
    }

    let ttl = packet[8];
    if ttl <= 1 {
        return Err(PacketError::TtlExceeded);
    }

    // Decrement TTL
    packet[8] = ttl - 1;

    // Update IPv4 header checksum
    // Zero out current checksum
    packet[10] = 0;
    packet[11] = 0;

    // Calculate new checksum
    let checksum = calculate_ipv4_checksum(packet);
    packet[10] = (checksum >> 8) as u8;
    packet[11] = (checksum & 0xFF) as u8;

    Ok(())
}

fn calculate_ipv4_checksum(packet: &[u8]) -> u16 {
    let header_len = ((packet[0] & 0x0F) * 4) as usize;
    let header = &packet[..header_len.min(packet.len())];

    let mut sum: u32 = 0;

    for i in (0..header.len()).step_by(2) {
        let word = if i + 1 < header.len() {
            u16::from_be_bytes([header[i], header[i + 1]])
        } else {
            u16::from_be_bytes([header[i], 0])
        };
        sum += word as u32;
    }

    // Add carries
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    !sum as u16
}
```

**Step 4: Implement TTL decrement for IPv6**

Create `src/packet/ipv6.rs`:
```rust
use crate::packet::PacketError;

/// Decrement hop limit in IPv6 packet (no checksum update needed)
pub fn decrement_ttl_v6(packet: &mut [u8]) -> Result<(), PacketError> {
    if packet.len() < 40 {
        return Err(PacketError::TooSmall(packet.len()));
    }

    let hop_limit = packet[7];
    if hop_limit <= 1 {
        return Err(PacketError::TtlExceeded);
    }

    // Decrement hop limit (no checksum in IPv6 header)
    packet[7] = hop_limit - 1;

    Ok(())
}
```

**Step 5: Add wrapper function in parser**

Add to `src/packet/parser.rs`:
```rust
use crate::packet::ipv4::decrement_ttl_v4;
use crate::packet::ipv6::decrement_ttl_v6;

/// Decrement TTL/hop limit in packet based on IP version
pub fn decrement_ttl(packet: &mut [u8]) -> Result<(), PacketError> {
    if packet.is_empty() {
        return Err(PacketError::TooSmall(0));
    }

    let version = packet[0] >> 4;

    match version {
        4 => decrement_ttl_v4(packet),
        6 => decrement_ttl_v6(packet),
        _ => Err(PacketError::InvalidVersion(version)),
    }
}
```

Update `src/packet/mod.rs`:
```rust
pub mod parser;
pub mod ipv4;
pub mod ipv6;

pub use parser::{PacketInfo, PacketError, parse_packet, decrement_ttl};
```

**Step 6: Run test to verify it passes**

Run:
```bash
cargo test test_decrement_ttl_ipv4
cargo test test_decrement_ttl_expired
```

Expected: PASS

**Step 7: Commit TTL handling**

```bash
git add src/packet/ipv4.rs src/packet/ipv6.rs src/packet/parser.rs src/packet/mod.rs tests/packet_test.rs
git commit -m "feat: implement TTL decrement with checksum update"
```

---

## Task 4: Create Forwarding Engine Structure

**Files:**
- Create: `src/forwarding/mod.rs`
- Create: `src/forwarding/engine.rs`
- Modify: `src/lib.rs`
- Create: `tests/forwarding_test.rs`

**Step 1: Write test for forwarding engine**

Create `tests/forwarding_test.rs`:
```rust
use netsimulator::forwarding::ForwardingEngine;
use netsimulator::topology::{Router, NetworkFabric};
use netsimulator::config::NetworkConfig;

#[test]
fn test_forwarding_engine_creation() {
    let toml_str = r#"
        [global]
        tun_a = "tunA"
        tun_b = "tunB"
        ingress_a = "Rx0y0"
        ingress_b = "Rx2y2"

        [Rx0y0_Rx0y1]
        mtu = 1500
    "#;

    let config = NetworkConfig::from_toml(toml_str).unwrap();
    let fabric = NetworkFabric::from_config(&config).unwrap();

    let engine = ForwardingEngine::new(fabric, config);
    assert!(engine.ingress_a().is_some());
    assert!(engine.ingress_b().is_some());
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_forwarding_engine_creation
```

Expected: FAIL with "module `forwarding` not found"

**Step 3: Create forwarding engine**

Create `src/forwarding/mod.rs`:
```rust
pub mod engine;

pub use engine::ForwardingEngine;
```

Update `src/lib.rs`:
```rust
pub mod config;
pub mod topology;
pub mod tun;
pub mod packet;
pub mod forwarding;
```

Create `src/forwarding/engine.rs`:
```rust
use crate::config::NetworkConfig;
use crate::topology::{NetworkFabric, Router};

/// Main forwarding engine that processes packets through the fabric
pub struct ForwardingEngine {
    fabric: NetworkFabric,
    ingress_a_name: String,
    ingress_b_name: String,
}

impl ForwardingEngine {
    pub fn new(fabric: NetworkFabric, config: NetworkConfig) -> Self {
        ForwardingEngine {
            fabric,
            ingress_a_name: config.global.ingress_a,
            ingress_b_name: config.global.ingress_b,
        }
    }

    pub fn ingress_a(&self) -> Option<&Router> {
        self.fabric.get_router_by_name(&self.ingress_a_name)
    }

    pub fn ingress_b(&self) -> Option<&Router> {
        self.fabric.get_router_by_name(&self.ingress_b_name)
    }

    pub fn fabric(&self) -> &NetworkFabric {
        &self.fabric
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_forwarding_engine_creation
```

Expected: PASS

**Step 5: Commit forwarding engine structure**

```bash
git add src/forwarding/ src/lib.rs tests/forwarding_test.rs
git commit -m "feat: add ForwardingEngine structure"
```

---

## Task 5: Implement Packet Forwarding Logic (Stub)

**Files:**
- Modify: `src/forwarding/engine.rs`
- Modify: `tests/forwarding_test.rs`

**Step 1: Write test for packet injection**

Add to `tests/forwarding_test.rs`:
```rust
use netsimulator::tun::PacketMessage;
use netsimulator::topology::VirtualCustomer;

#[test]
fn test_inject_packet_from_tun_a() {
    let toml_str = r#"
        [global]
        tun_a = "tunA"
        tun_b = "tunB"
        ingress_a = "Rx0y0"
        ingress_b = "Rx2y2"

        [Rx0y0_Rx0y1]
    "#;

    let config = NetworkConfig::from_toml(toml_str).unwrap();
    let fabric = NetworkFabric::from_config(&config).unwrap();
    let engine = ForwardingEngine::new(fabric, config);

    let packet = vec![0x45, 0x00, 0x00, 0x14];  // IPv4 header start
    let msg = PacketMessage::new(packet.clone(), "tunA");
    let customer = VirtualCustomer::new(0);

    let result = engine.identify_ingress_router(&msg);
    assert!(result.is_some());
    let (router, _target) = result.unwrap();
    assert_eq!(router.name(), "Rx0y0");
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_inject_packet_from_tun_a
```

Expected: FAIL with "no method named `identify_ingress_router`"

**Step 3: Implement ingress identification**

Add to `src/forwarding/engine.rs`:
```rust
use crate::tun::PacketMessage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetTun {
    TunA,
    TunB,
}

impl ForwardingEngine {
    /// Identify which ingress router to use based on source TUN interface
    /// Returns (ingress_router, target_tun)
    pub fn identify_ingress_router(&self, msg: &PacketMessage) -> Option<(&Router, TargetTun)> {
        let source = msg.source_interface();

        // Packet from tunA goes to ingress_a, target is tunB
        if source.contains("tunA") || source == &self.ingress_a_name {
            self.ingress_a().map(|r| (r, TargetTun::TunB))
        }
        // Packet from tunB goes to ingress_b, target is tunA
        else if source.contains("tunB") || source == &self.ingress_b_name {
            self.ingress_b().map(|r| (r, TargetTun::TunA))
        }
        else {
            None
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_inject_packet_from_tun_a
```

Expected: PASS

**Step 5: Commit ingress identification**

```bash
git add src/forwarding/engine.rs tests/forwarding_test.rs
git commit -m "feat: implement ingress router identification"
```

---

## Plan 4 Completion Checklist

Before moving to Plan 5, verify:

- [ ] All tests pass: `cargo test`
- [ ] IPv4 packets can be parsed
- [ ] IPv6 packets can be parsed
- [ ] TTL can be decremented
- [ ] TTL expiration is detected
- [ ] IPv4 checksum is updated correctly
- [ ] ForwardingEngine structure exists
- [ ] Ingress router identification works

Run full test suite:
```bash
cargo test
```

**Next:** Proceed to Plan 5 (Routing Table Computation)

**Note:** The actual packet forwarding between routers will be implemented in later tasks after routing tables are computed. This plan establishes the parsing and TTL handling foundations.
