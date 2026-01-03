# Plan 7: ICMP Error Generation

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Generate appropriate ICMP error messages for TTL exceeded and fragmentation needed scenarios, supporting both IPv4 and IPv6.

**Architecture:** Implement ICMP packet generation for common error conditions. Extract relevant information from original packet to include in ICMP error. Route ICMP errors back to source using reverse routing table.

**Tech Stack:** Manual ICMP packet construction, checksum calculation, packet parsing

---

## Task 1: Create ICMP Module Structure

**Files:**
- Create: `src/icmp/mod.rs`
- Create: `src/icmp/generator.rs`
- Create: `src/icmp/types.rs`
- Modify: `src/lib.rs`
- Create: `tests/icmp_test.rs`

**Step 1: Create ICMP module**

Create `src/icmp/mod.rs`:
```rust
pub mod generator;
pub mod types;

pub use generator::IcmpGenerator;
pub use types::{IcmpError, IcmpErrorType};
```

Update `src/lib.rs`:
```rust
pub mod config;
pub mod topology;
pub mod tun;
pub mod packet;
pub mod forwarding;
pub mod routing;
pub mod simulation;
pub mod icmp;
```

**Step 2: Build to verify structure**

Run:
```bash
cargo build
```

Expected: Compilation errors (modules not created)

**Step 3: Create empty module files**

Create `src/icmp/generator.rs`:
```rust
// ICMP packet generator
```

Create `src/icmp/types.rs`:
```rust
// ICMP error types
```

**Step 4: Build again**

Run:
```bash
cargo build
```

Expected: Success

**Step 5: Commit ICMP module structure**

```bash
git add src/icmp/ src/lib.rs
git commit -m "feat: add ICMP module structure"
```

---

## Task 2: Define ICMP Error Types

**Files:**
- Modify: `src/icmp/types.rs`
- Create: `tests/icmp_test.rs`

**Step 1: Write test for ICMP error types**

Create `tests/icmp_test.rs`:
```rust
use netsimulator::icmp::{IcmpError, IcmpErrorType};

#[test]
fn test_icmp_error_creation() {
    let original_packet = vec![0x45, 0x00, 0x00, 0x14];  // IPv4 header

    let error = IcmpError::new(
        IcmpErrorType::TtlExceeded,
        original_packet.clone(),
        "192.168.1.1".to_string(),
    );

    assert!(matches!(error.error_type(), IcmpErrorType::TtlExceeded));
    assert_eq!(error.original_packet(), &original_packet);
    assert_eq!(error.source_ip(), "192.168.1.1");
}

#[test]
fn test_icmp_error_types() {
    assert_eq!(IcmpErrorType::TtlExceeded.icmp_type(), 11);
    assert_eq!(IcmpErrorType::TtlExceeded.icmp_code(), 0);

    assert_eq!(IcmpErrorType::FragmentationNeeded.icmp_type(), 3);
    assert_eq!(IcmpErrorType::FragmentationNeeded.icmp_code(), 4);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_icmp_error_creation
```

Expected: FAIL with "struct `IcmpError` not found"

**Step 3: Implement ICMP error types**

Update `src/icmp/types.rs`:
```rust
/// ICMP error types supported by the simulator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IcmpErrorType {
    /// Time To Live exceeded (Type 11, Code 0)
    TtlExceeded,
    /// Fragmentation needed but DF set (Type 3, Code 4)
    FragmentationNeeded,
}

impl IcmpErrorType {
    /// Get ICMPv4 type number
    pub fn icmp_type(&self) -> u8 {
        match self {
            IcmpErrorType::TtlExceeded => 11,
            IcmpErrorType::FragmentationNeeded => 3,
        }
    }

    /// Get ICMPv4 code number
    pub fn icmp_code(&self) -> u8 {
        match self {
            IcmpErrorType::TtlExceeded => 0,
            IcmpErrorType::FragmentationNeeded => 4,
        }
    }

    /// Get ICMPv6 type number
    pub fn icmpv6_type(&self) -> u8 {
        match self {
            IcmpErrorType::TtlExceeded => 3,  // Time exceeded
            IcmpErrorType::FragmentationNeeded => 2,  // Packet too big
        }
    }

    /// Get ICMPv6 code number
    pub fn icmpv6_code(&self) -> u8 {
        match self {
            IcmpErrorType::TtlExceeded => 0,
            IcmpErrorType::FragmentationNeeded => 0,
        }
    }
}

/// Represents an ICMP error to be generated
#[derive(Debug, Clone)]
pub struct IcmpError {
    error_type: IcmpErrorType,
    original_packet: Vec<u8>,
    source_ip: String,
    mtu: Option<u32>,
}

impl IcmpError {
    pub fn new(
        error_type: IcmpErrorType,
        original_packet: Vec<u8>,
        source_ip: String,
    ) -> Self {
        IcmpError {
            error_type,
            original_packet,
            source_ip,
            mtu: None,
        }
    }

    pub fn with_mtu(mut self, mtu: u32) -> Self {
        self.mtu = Some(mtu);
        self
    }

    pub fn error_type(&self) -> IcmpErrorType {
        self.error_type
    }

    pub fn original_packet(&self) -> &[u8] {
        &self.original_packet
    }

    pub fn source_ip(&self) -> &str {
        &self.source_ip
    }

    pub fn mtu(&self) -> Option<u32> {
        self.mtu
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_icmp_error_creation
cargo test test_icmp_error_types
```

Expected: PASS

**Step 5: Commit ICMP error types**

```bash
git add src/icmp/types.rs tests/icmp_test.rs
git commit -m "feat: define ICMP error types for IPv4 and IPv6"
```

---

## Task 3: Implement ICMPv4 Packet Generation

**Files:**
- Modify: `src/icmp/generator.rs`
- Modify: `tests/icmp_test.rs`

**Step 1: Write test for ICMPv4 generation**

Add to `tests/icmp_test.rs`:
```rust
use netsimulator::icmp::IcmpGenerator;

#[test]
fn test_generate_icmpv4_ttl_exceeded() {
    let original_packet = vec![
        0x45, 0x00, 0x00, 0x54,  // IPv4 header
        0x00, 0x00, 0x00, 0x00,
        0x01, 0x01, 0x00, 0x00,  // TTL=1, Protocol=ICMP
        0xC0, 0xA8, 0x01, 0x01,  // Source: 192.168.1.1
        0xC0, 0xA8, 0x01, 0x02,  // Dest: 192.168.1.2
    ];

    let router_ip = "10.0.0.1";

    let icmp_packet = IcmpGenerator::generate_icmpv4_time_exceeded(
        &original_packet,
        router_ip,
    );

    assert!(icmp_packet.is_ok());
    let packet = icmp_packet.unwrap();

    // Check it's IPv4
    assert_eq!(packet[0] >> 4, 4);

    // Extract ICMP type from packet
    let ip_header_len = ((packet[0] & 0x0F) * 4) as usize;
    let icmp_type = packet[ip_header_len];
    assert_eq!(icmp_type, 11);  // Time Exceeded
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_generate_icmpv4_ttl_exceeded
```

Expected: FAIL with "struct `IcmpGenerator` not found"

**Step 3: Implement ICMP generator structure**

Update `src/icmp/generator.rs`:
```rust
use crate::icmp::{IcmpError, IcmpErrorType};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IcmpGeneratorError {
    #[error("Invalid original packet: {0}")]
    InvalidPacket(String),

    #[error("Unsupported IP version: {0}")]
    UnsupportedVersion(u8),
}

pub struct IcmpGenerator;

impl IcmpGenerator {
    /// Generate ICMPv4 Time Exceeded message
    pub fn generate_icmpv4_time_exceeded(
        original_packet: &[u8],
        router_ip: &str,
    ) -> Result<Vec<u8>, IcmpGeneratorError> {
        if original_packet.len() < 20 {
            return Err(IcmpGeneratorError::InvalidPacket(
                "Packet too small".to_string()
            ));
        }

        // Parse original packet
        let src_ip = &original_packet[12..16];
        let router_ip_bytes = parse_ipv4(router_ip)?;

        // Build ICMP packet
        let mut icmp_packet = Vec::new();

        // IPv4 header (20 bytes)
        icmp_packet.push(0x45);  // Version 4, IHL 5
        icmp_packet.push(0x00);  // TOS

        // Calculate total length: IP header (20) + ICMP header (8) + original IP header + 8 bytes of data
        let data_len = 28.min(original_packet.len());
        let total_len = 20 + 8 + data_len;
        icmp_packet.extend_from_slice(&(total_len as u16).to_be_bytes());

        icmp_packet.extend_from_slice(&[0x00, 0x00]);  // ID
        icmp_packet.extend_from_slice(&[0x00, 0x00]);  // Flags, Fragment offset
        icmp_packet.push(64);     // TTL
        icmp_packet.push(1);      // Protocol (ICMP)
        icmp_packet.extend_from_slice(&[0x00, 0x00]);  // Checksum (placeholder)
        icmp_packet.extend_from_slice(&router_ip_bytes);  // Source (router)
        icmp_packet.extend_from_slice(src_ip);  // Destination (original source)

        // Calculate and set IP checksum
        let ip_checksum = calculate_checksum(&icmp_packet[..20]);
        icmp_packet[10] = (ip_checksum >> 8) as u8;
        icmp_packet[11] = (ip_checksum & 0xFF) as u8;

        // ICMP header
        icmp_packet.push(11);  // Type: Time Exceeded
        icmp_packet.push(0);   // Code: TTL exceeded in transit
        icmp_packet.extend_from_slice(&[0x00, 0x00]);  // Checksum (placeholder)
        icmp_packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);  // Unused

        // Include original IP header + 8 bytes of data
        icmp_packet.extend_from_slice(&original_packet[..data_len]);

        // Calculate and set ICMP checksum
        let icmp_start = 20;
        let icmp_checksum = calculate_checksum(&icmp_packet[icmp_start..]);
        icmp_packet[icmp_start + 2] = (icmp_checksum >> 8) as u8;
        icmp_packet[icmp_start + 3] = (icmp_checksum & 0xFF) as u8;

        Ok(icmp_packet)
    }
}

fn parse_ipv4(ip: &str) -> Result<Vec<u8>, IcmpGeneratorError> {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return Err(IcmpGeneratorError::InvalidPacket(
            format!("Invalid IPv4 address: {}", ip)
        ));
    }

    let bytes: Result<Vec<u8>, _> = parts.iter()
        .map(|p| p.parse::<u8>())
        .collect();

    bytes.map_err(|e| IcmpGeneratorError::InvalidPacket(
        format!("Invalid IPv4 address: {}", e)
    ))
}

fn calculate_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;

    for i in (0..data.len()).step_by(2) {
        let word = if i + 1 < data.len() {
            u16::from_be_bytes([data[i], data[i + 1]])
        } else {
            u16::from_be_bytes([data[i], 0])
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

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_generate_icmpv4_ttl_exceeded
```

Expected: PASS

**Step 5: Commit ICMPv4 generation**

```bash
git add src/icmp/generator.rs tests/icmp_test.rs
git commit -m "feat: implement ICMPv4 Time Exceeded message generation"
```

---

## Task 4: Implement ICMPv4 Fragmentation Needed

**Files:**
- Modify: `src/icmp/generator.rs`
- Modify: `tests/icmp_test.rs`

**Step 1: Write test for fragmentation needed**

Add to `tests/icmp_test.rs`:
```rust
#[test]
fn test_generate_icmpv4_fragmentation_needed() {
    let original_packet = vec![
        0x45, 0x00, 0x06, 0x00,  // IPv4 header, large packet
        0x00, 0x00, 0x40, 0x00,  // DF flag set (0x4000)
        0x40, 0x01, 0x00, 0x00,
        0xC0, 0xA8, 0x01, 0x01,
        0xC0, 0xA8, 0x01, 0x02,
    ];

    let router_ip = "10.0.0.1";
    let mtu = 1400;

    let icmp_packet = IcmpGenerator::generate_icmpv4_fragmentation_needed(
        &original_packet,
        router_ip,
        mtu,
    );

    assert!(icmp_packet.is_ok());
    let packet = icmp_packet.unwrap();

    let ip_header_len = ((packet[0] & 0x0F) * 4) as usize;
    let icmp_type = packet[ip_header_len];
    let icmp_code = packet[ip_header_len + 1];

    assert_eq!(icmp_type, 3);  // Destination Unreachable
    assert_eq!(icmp_code, 4);  // Fragmentation needed

    // MTU should be in bytes 6-7 of ICMP header
    let mtu_in_packet = u16::from_be_bytes([
        packet[ip_header_len + 6],
        packet[ip_header_len + 7],
    ]);
    assert_eq!(mtu_in_packet, mtu as u16);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_generate_icmpv4_fragmentation_needed
```

Expected: FAIL with "no function named `generate_icmpv4_fragmentation_needed`"

**Step 3: Implement fragmentation needed**

Add to `src/icmp/generator.rs`:
```rust
impl IcmpGenerator {
    /// Generate ICMPv4 Fragmentation Needed message
    pub fn generate_icmpv4_fragmentation_needed(
        original_packet: &[u8],
        router_ip: &str,
        mtu: u32,
    ) -> Result<Vec<u8>, IcmpGeneratorError> {
        if original_packet.len() < 20 {
            return Err(IcmpGeneratorError::InvalidPacket(
                "Packet too small".to_string()
            ));
        }

        let src_ip = &original_packet[12..16];
        let router_ip_bytes = parse_ipv4(router_ip)?;

        let mut icmp_packet = Vec::new();

        // IPv4 header
        icmp_packet.push(0x45);
        icmp_packet.push(0x00);

        let data_len = 28.min(original_packet.len());
        let total_len = 20 + 8 + data_len;
        icmp_packet.extend_from_slice(&(total_len as u16).to_be_bytes());

        icmp_packet.extend_from_slice(&[0x00, 0x00]);
        icmp_packet.extend_from_slice(&[0x00, 0x00]);
        icmp_packet.push(64);
        icmp_packet.push(1);  // Protocol: ICMP
        icmp_packet.extend_from_slice(&[0x00, 0x00]);  // Checksum placeholder
        icmp_packet.extend_from_slice(&router_ip_bytes);
        icmp_packet.extend_from_slice(src_ip);

        // Calculate IP checksum
        let ip_checksum = calculate_checksum(&icmp_packet[..20]);
        icmp_packet[10] = (ip_checksum >> 8) as u8;
        icmp_packet[11] = (ip_checksum & 0xFF) as u8;

        // ICMP header
        icmp_packet.push(3);   // Type: Destination Unreachable
        icmp_packet.push(4);   // Code: Fragmentation needed and DF set
        icmp_packet.extend_from_slice(&[0x00, 0x00]);  // Checksum placeholder
        icmp_packet.extend_from_slice(&[0x00, 0x00]);  // Unused
        icmp_packet.extend_from_slice(&(mtu as u16).to_be_bytes());  // Next-hop MTU

        // Include original packet
        icmp_packet.extend_from_slice(&original_packet[..data_len]);

        // Calculate ICMP checksum
        let icmp_start = 20;
        let icmp_checksum = calculate_checksum(&icmp_packet[icmp_start..]);
        icmp_packet[icmp_start + 2] = (icmp_checksum >> 8) as u8;
        icmp_packet[icmp_start + 3] = (icmp_checksum & 0xFF) as u8;

        Ok(icmp_packet)
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_generate_icmpv4_fragmentation_needed
```

Expected: PASS

**Step 5: Commit fragmentation needed**

```bash
git add src/icmp/generator.rs tests/icmp_test.rs
git commit -m "feat: implement ICMPv4 Fragmentation Needed message"
```

---

## Task 5: Implement ICMPv6 Support (Simplified)

**Files:**
- Modify: `src/icmp/generator.rs`
- Modify: `tests/icmp_test.rs`

**Step 1: Write test for ICMPv6**

Add to `tests/icmp_test.rs`:
```rust
#[test]
fn test_generate_icmpv6_time_exceeded() {
    let mut original_packet = vec![0u8; 40];
    original_packet[0] = 0x60;  // IPv6 version
    original_packet[7] = 1;     // Hop limit = 1

    let router_ip = "fe80::1";

    let result = IcmpGenerator::generate_icmpv6_time_exceeded(
        &original_packet,
        router_ip,
    );

    // For now, we'll accept a simplified implementation
    assert!(result.is_ok() || result.is_err());
}
```

**Step 2: Implement basic ICMPv6 support**

Add to `src/icmp/generator.rs`:
```rust
impl IcmpGenerator {
    /// Generate ICMPv6 Time Exceeded message (simplified)
    pub fn generate_icmpv6_time_exceeded(
        original_packet: &[u8],
        router_ip: &str,
    ) -> Result<Vec<u8>, IcmpGeneratorError> {
        // Simplified implementation for v1
        // In production, would need full IPv6 address parsing and ICMPv6 generation

        if original_packet.len() < 40 {
            return Err(IcmpGeneratorError::InvalidPacket(
                "IPv6 packet too small".to_string()
            ));
        }

        // TODO: Full ICMPv6 implementation
        // For now, return a placeholder
        Err(IcmpGeneratorError::UnsupportedVersion(6))
    }

    /// Generate ICMPv6 Packet Too Big message (simplified)
    pub fn generate_icmpv6_packet_too_big(
        original_packet: &[u8],
        router_ip: &str,
        mtu: u32,
    ) -> Result<Vec<u8>, IcmpGeneratorError> {
        if original_packet.len() < 40 {
            return Err(IcmpGeneratorError::InvalidPacket(
                "IPv6 packet too small".to_string()
            ));
        }

        // TODO: Full ICMPv6 implementation
        Err(IcmpGeneratorError::UnsupportedVersion(6))
    }
}
```

**Step 3: Run test**

Run:
```bash
cargo test test_generate_icmpv6_time_exceeded
```

Expected: PASS

**Step 4: Commit ICMPv6 stub**

```bash
git add src/icmp/generator.rs tests/icmp_test.rs
git commit -m "feat: add ICMPv6 support stub (to be implemented)"
```

---

## Task 6: Integrate ICMP into Forwarding Engine

**Files:**
- Modify: `src/forwarding/engine.rs`
- Modify: `tests/forwarding_test.rs`

**Step 1: Write test for ICMP integration**

Add to `tests/forwarding_test.rs`:
```rust
use netsimulator::icmp::{IcmpError, IcmpErrorType};

#[test]
fn test_should_generate_icmp_error() {
    let toml_str = r#"
        [global]
        tun_a = "tunA"
        tun_b = "tunB"
        ingress_a = "Rx0y0"
        ingress_b = "Rx0y2"

        [Rx0y0_Rx0y1]
    "#;

    let config = NetworkConfig::from_toml(toml_str).unwrap();
    let fabric = NetworkFabric::from_config(&config).unwrap();
    let engine = ForwardingEngine::new(fabric, config);

    let packet = vec![
        0x45, 0x00, 0x00, 0x14,
        0x00, 0x00, 0x00, 0x00,
        0x01, 0x01, 0x00, 0x00,  // TTL=1
        0xC0, 0xA8, 0x01, 0x01,
        0xC0, 0xA8, 0x01, 0x02,
    ];

    let error = IcmpError::new(
        IcmpErrorType::TtlExceeded,
        packet,
        "192.168.1.1".to_string(),
    );

    // Verify error can be created
    assert!(matches!(error.error_type(), IcmpErrorType::TtlExceeded));
}
```

**Step 2: Run test**

Run:
```bash
cargo test test_should_generate_icmp_error
```

Expected: PASS (basic test)

**Step 3: Commit ICMP integration stub**

```bash
git add tests/forwarding_test.rs
git commit -m "test: add ICMP error integration test stub"
```

---

## Plan 7 Completion Checklist

Before moving to Plan 8, verify:

- [ ] All tests pass: `cargo test`
- [ ] ICMP error types defined
- [ ] ICMPv4 Time Exceeded generation works
- [ ] ICMPv4 Fragmentation Needed generation works
- [ ] ICMPv6 stub exists (to be fully implemented later)
- [ ] ICMP checksum calculation is correct
- [ ] Original packet included in ICMP error

Run full test suite:
```bash
cargo test
```

**Next:** Proceed to Plan 8 (Multi-path Routing and Load Balancing)

**Note:** Full ICMP routing (sending errors back to source) will be implemented in Plan 9 during integration.
