# Plan 8: Multi-path Routing and Load Balancing

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement multi-path routing with 5-tuple hashing for consistent path selection and optional per-packet load balancing.

**Architecture:** Extract 5-tuple (src IP, dst IP, src port, dst port, protocol) from packets. Combine with router hostname in hash function to select path. For per-packet load balancing links, add a counter to the hash. Ensure flows take consistent paths for connection stability.

**Tech Stack:** Hash functions from std library, packet parsing for port extraction

---

## Task 1: Create Multi-path Module

**Files:**
- Create: `src/forwarding/multipath.rs`
- Modify: `src/forwarding/mod.rs`
- Create: `tests/multipath_test.rs`

**Step 1: Create multipath module**

Create `src/forwarding/multipath.rs`:
```rust
// Multi-path routing implementation
```

Update `src/forwarding/mod.rs`:
```rust
pub mod engine;
pub mod multipath;

pub use engine::{ForwardingEngine, TargetTun};
pub use multipath::{FiveTuple, PathSelector};
```

**Step 2: Build to verify**

Run:
```bash
cargo build
```

Expected: Success

**Step 3: Commit module structure**

```bash
git add src/forwarding/multipath.rs src/forwarding/mod.rs
git commit -m "feat: add multipath routing module structure"
```

---

## Task 2: Implement 5-Tuple Extraction

**Files:**
- Modify: `src/forwarding/multipath.rs`
- Create: `tests/multipath_test.rs`

**Step 1: Write test for 5-tuple extraction**

Create `tests/multipath_test.rs`:
```rust
use netsimulator::forwarding::FiveTuple;

#[test]
fn test_extract_five_tuple_ipv4_tcp() {
    // IPv4 TCP packet
    let packet = vec![
        // IP header
        0x45, 0x00, 0x00, 0x28,  // Version, IHL, length
        0x00, 0x00, 0x00, 0x00,
        0x40, 0x06, 0x00, 0x00,  // TTL, Protocol=6 (TCP)
        192, 168, 1, 1,          // Source IP
        192, 168, 1, 2,          // Dest IP
        // TCP header
        0x04, 0xD2,              // Source port = 1234
        0x00, 0x50,              // Dest port = 80
    ];

    let tuple = FiveTuple::extract(&packet);
    assert!(tuple.is_ok());

    let tuple = tuple.unwrap();
    assert_eq!(tuple.protocol(), 6);
    assert_eq!(tuple.src_port(), Some(1234));
    assert_eq!(tuple.dst_port(), Some(80));
}

#[test]
fn test_extract_five_tuple_ipv4_udp() {
    let packet = vec![
        0x45, 0x00, 0x00, 0x28,
        0x00, 0x00, 0x00, 0x00,
        0x40, 0x11, 0x00, 0x00,  // Protocol=17 (UDP)
        192, 168, 1, 1,
        192, 168, 1, 2,
        // UDP header
        0x1F, 0x90,              // Source port = 8080
        0x00, 0x35,              // Dest port = 53 (DNS)
    ];

    let tuple = FiveTuple::extract(&packet).unwrap();
    assert_eq!(tuple.protocol(), 17);
    assert_eq!(tuple.src_port(), Some(8080));
    assert_eq!(tuple.dst_port(), Some(53));
}

#[test]
fn test_extract_five_tuple_ipv4_icmp() {
    let packet = vec![
        0x45, 0x00, 0x00, 0x28,
        0x00, 0x00, 0x00, 0x00,
        0x40, 0x01, 0x00, 0x00,  // Protocol=1 (ICMP)
        192, 168, 1, 1,
        192, 168, 1, 2,
    ];

    let tuple = FiveTuple::extract(&packet).unwrap();
    assert_eq!(tuple.protocol(), 1);
    assert_eq!(tuple.src_port(), None);  // ICMP has no ports
    assert_eq!(tuple.dst_port(), None);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_extract_five_tuple_ipv4_tcp
```

Expected: FAIL with "struct `FiveTuple` not found"

**Step 3: Implement 5-tuple structure and extraction**

Update `src/forwarding/multipath.rs`:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MultipathError {
    #[error("Invalid packet: {0}")]
    InvalidPacket(String),
}

/// Represents a 5-tuple for flow identification
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FiveTuple {
    src_ip: Vec<u8>,
    dst_ip: Vec<u8>,
    protocol: u8,
    src_port: Option<u16>,
    dst_port: Option<u16>,
}

impl FiveTuple {
    /// Extract 5-tuple from packet
    pub fn extract(packet: &[u8]) -> Result<Self, MultipathError> {
        if packet.is_empty() {
            return Err(MultipathError::InvalidPacket("Empty packet".to_string()));
        }

        let version = packet[0] >> 4;

        match version {
            4 => Self::extract_ipv4(packet),
            6 => Self::extract_ipv6(packet),
            _ => Err(MultipathError::InvalidPacket(
                format!("Invalid IP version: {}", version)
            )),
        }
    }

    fn extract_ipv4(packet: &[u8]) -> Result<Self, MultipathError> {
        if packet.len() < 20 {
            return Err(MultipathError::InvalidPacket("Packet too small".to_string()));
        }

        let protocol = packet[9];
        let src_ip = packet[12..16].to_vec();
        let dst_ip = packet[16..20].to_vec();

        let (src_port, dst_port) = Self::extract_ports(packet, 20, protocol)?;

        Ok(FiveTuple {
            src_ip,
            dst_ip,
            protocol,
            src_port,
            dst_port,
        })
    }

    fn extract_ipv6(packet: &[u8]) -> Result<Self, MultipathError> {
        if packet.len() < 40 {
            return Err(MultipathError::InvalidPacket("Packet too small".to_string()));
        }

        let protocol = packet[6];  // Next header
        let src_ip = packet[8..24].to_vec();
        let dst_ip = packet[24..40].to_vec();

        let (src_port, dst_port) = Self::extract_ports(packet, 40, protocol)?;

        Ok(FiveTuple {
            src_ip,
            dst_ip,
            protocol,
            src_port,
            dst_port,
        })
    }

    fn extract_ports(
        packet: &[u8],
        offset: usize,
        protocol: u8,
    ) -> Result<(Option<u16>, Option<u16>), MultipathError> {
        // TCP (6) and UDP (17) have ports at same location
        if (protocol == 6 || protocol == 17) && packet.len() >= offset + 4 {
            let src_port = u16::from_be_bytes([packet[offset], packet[offset + 1]]);
            let dst_port = u16::from_be_bytes([packet[offset + 2], packet[offset + 3]]);
            Ok((Some(src_port), Some(dst_port)))
        } else {
            // Other protocols (ICMP, etc.) don't have ports
            Ok((None, None))
        }
    }

    pub fn src_ip(&self) -> &[u8] {
        &self.src_ip
    }

    pub fn dst_ip(&self) -> &[u8] {
        &self.dst_ip
    }

    pub fn protocol(&self) -> u8 {
        self.protocol
    }

    pub fn src_port(&self) -> Option<u16> {
        self.src_port
    }

    pub fn dst_port(&self) -> Option<u16> {
        self.dst_port
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_extract_five_tuple_ipv4_tcp
cargo test test_extract_five_tuple_ipv4_udp
cargo test test_extract_five_tuple_ipv4_icmp
```

Expected: PASS

**Step 5: Commit 5-tuple extraction**

```bash
git add src/forwarding/multipath.rs tests/multipath_test.rs
git commit -m "feat: implement 5-tuple extraction from packets"
```

---

## Task 3: Implement Path Selection with Hashing

**Files:**
- Modify: `src/forwarding/multipath.rs`
- Modify: `tests/multipath_test.rs`

**Step 1: Write test for path selection**

Add to `tests/multipath_test.rs`:
```rust
use netsimulator::forwarding::PathSelector;
use netsimulator::topology::Router;

#[test]
fn test_path_selection_consistency() {
    let router = Router::new(0, 0);
    let selector = PathSelector::new(router.name().to_string());

    let packet1 = vec![
        0x45, 0x00, 0x00, 0x28,
        0x00, 0x00, 0x00, 0x00,
        0x40, 0x06, 0x00, 0x00,
        192, 168, 1, 1,
        192, 168, 1, 2,
        0x04, 0xD2,  // Source port
        0x00, 0x50,  // Dest port
    ];

    let packet2 = packet1.clone();

    // Same packet should always select same path
    let paths = vec![
        Router::new(0, 1),
        Router::new(1, 0),
        Router::new(1, 1),
    ];

    let idx1 = selector.select_path(&packet1, &paths, false, 0).unwrap();
    let idx2 = selector.select_path(&packet2, &paths, false, 0).unwrap();

    assert_eq!(idx1, idx2);
}

#[test]
fn test_path_selection_distribution() {
    let router = Router::new(0, 0);
    let selector = PathSelector::new(router.name().to_string());

    let paths = vec![
        Router::new(0, 1),
        Router::new(1, 0),
    ];

    let mut path_counts = vec![0, 0];

    // Generate different packets by varying source port
    for src_port in 0..100 {
        let mut packet = vec![
            0x45, 0x00, 0x00, 0x28,
            0x00, 0x00, 0x00, 0x00,
            0x40, 0x06, 0x00, 0x00,
            192, 168, 1, 1,
            192, 168, 1, 2,
            (src_port >> 8) as u8,
            (src_port & 0xFF) as u8,
            0x00, 0x50,
        ];

        let idx = selector.select_path(&packet, &paths, false, 0).unwrap();
        path_counts[idx] += 1;
    }

    // Both paths should be used (not necessarily evenly, but both should be non-zero)
    assert!(path_counts[0] > 0);
    assert!(path_counts[1] > 0);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_path_selection_consistency
```

Expected: FAIL with "struct `PathSelector` not found"

**Step 3: Implement path selector**

Add to `src/forwarding/multipath.rs`:
```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use crate::topology::Router;

/// Selects paths for multi-path routing
pub struct PathSelector {
    router_name: String,
}

impl PathSelector {
    pub fn new(router_name: String) -> Self {
        PathSelector { router_name }
    }

    /// Select a path from available paths based on 5-tuple hash
    /// If per_packet_lb is true, uses counter for per-packet load balancing
    pub fn select_path(
        &self,
        packet: &[u8],
        paths: &[Router],
        per_packet_lb: bool,
        counter: u64,
    ) -> Result<usize, MultipathError> {
        if paths.is_empty() {
            return Err(MultipathError::InvalidPacket("No paths available".to_string()));
        }

        if paths.len() == 1 {
            return Ok(0);
        }

        let hash = if per_packet_lb {
            // Use counter for per-packet load balancing
            self.hash_with_counter(packet, counter)
        } else {
            // Use 5-tuple for flow-based load balancing
            self.hash_packet(packet)?
        };

        let idx = (hash % paths.len() as u64) as usize;
        Ok(idx)
    }

    fn hash_packet(&self, packet: &[u8]) -> Result<u64, MultipathError> {
        let tuple = FiveTuple::extract(packet)?;
        let mut hasher = DefaultHasher::new();

        // Hash 5-tuple
        tuple.hash(&mut hasher);

        // Hash router name for diversity across routers
        self.router_name.hash(&mut hasher);

        Ok(hasher.finish())
    }

    fn hash_with_counter(&self, packet: &[u8], counter: u64) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash packet data (or subset)
        packet.hash(&mut hasher);

        // Hash router name
        self.router_name.hash(&mut hasher);

        // Hash counter for per-packet variation
        counter.hash(&mut hasher);

        hasher.finish()
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_path_selection_consistency
cargo test test_path_selection_distribution
```

Expected: PASS

**Step 5: Commit path selection**

```bash
git add src/forwarding/multipath.rs tests/multipath_test.rs
git commit -m "feat: implement path selection with 5-tuple hashing"
```

---

## Task 4: Implement Per-Packet Load Balancing

**Files:**
- Modify: `src/forwarding/multipath.rs`
- Modify: `tests/multipath_test.rs`

**Step 1: Write test for per-packet load balancing**

Add to `tests/multipath_test.rs`:
```rust
#[test]
fn test_per_packet_load_balancing() {
    let router = Router::new(0, 0);
    let selector = PathSelector::new(router.name().to_string());

    let packet = vec![
        0x45, 0x00, 0x00, 0x28,
        0x00, 0x00, 0x00, 0x00,
        0x40, 0x06, 0x00, 0x00,
        192, 168, 1, 1,
        192, 168, 1, 2,
        0x04, 0xD2,
        0x00, 0x50,
    ];

    let paths = vec![
        Router::new(0, 1),
        Router::new(1, 0),
    ];

    let mut path_counts = vec![0, 0];

    // With per-packet LB enabled, same packet + different counter = different paths possible
    for counter in 0..100 {
        let idx = selector.select_path(&packet, &paths, true, counter).unwrap();
        path_counts[idx] += 1;
    }

    // Both paths should be used
    assert!(path_counts[0] > 0);
    assert!(path_counts[1] > 0);
}

#[test]
fn test_flow_based_vs_per_packet() {
    let router = Router::new(0, 0);
    let selector = PathSelector::new(router.name().to_string());

    let packet = vec![
        0x45, 0x00, 0x00, 0x28,
        0x00, 0x00, 0x00, 0x00,
        0x40, 0x06, 0x00, 0x00,
        192, 168, 1, 1,
        192, 168, 1, 2,
        0x04, 0xD2,
        0x00, 0x50,
    ];

    let paths = vec![
        Router::new(0, 1),
        Router::new(1, 0),
        Router::new(1, 1),
    ];

    // Flow-based: same packet always gives same path
    let idx1 = selector.select_path(&packet, &paths, false, 0).unwrap();
    let idx2 = selector.select_path(&packet, &paths, false, 1).unwrap();
    assert_eq!(idx1, idx2);  // Counter ignored

    // Per-packet: counter affects selection
    let mut indices = Vec::new();
    for counter in 0..10 {
        indices.push(selector.select_path(&packet, &paths, true, counter).unwrap());
    }

    // At least some variation in selected paths
    let unique_count = indices.iter().collect::<std::collections::HashSet<_>>().len();
    assert!(unique_count > 1);
}
```

**Step 2: Run test to verify it passes**

Run:
```bash
cargo test test_per_packet_load_balancing
cargo test test_flow_based_vs_per_packet
```

Expected: PASS (implementation already supports this)

**Step 3: Commit per-packet load balancing test**

```bash
git add tests/multipath_test.rs
git commit -m "test: add per-packet load balancing tests"
```

---

## Task 5: Integrate Multi-path into Forwarding Engine

**Files:**
- Modify: `src/forwarding/engine.rs`
- Modify: `tests/forwarding_test.rs`

**Step 1: Write test for multipath integration**

Add to `tests/forwarding_test.rs`:
```rust
use netsimulator::forwarding::PathSelector;

#[test]
fn test_forwarding_engine_multipath_selection() {
    let toml_str = r#"
        [global]
        tun_a = "tunA"
        tun_b = "tunB"
        ingress_a = "Rx0y0"
        ingress_b = "Rx2y2"

        [Rx0y0_Rx0y1]
        [Rx0y0_Rx1y0]
        [Rx0y1_Rx0y2]
        [Rx1y0_Rx1y1]
        [Rx1y1_Rx2y2]
    "#;

    let config = NetworkConfig::from_toml(toml_str).unwrap();
    let fabric = NetworkFabric::from_config(&config).unwrap();
    let tables = netsimulator::routing::build_all_routing_tables(&fabric, &config);
    let engine = ForwardingEngine::with_routing_tables(fabric, config, tables);

    // Router Rx0y0 has multiple next hops
    let r0 = Router::new(0, 0);
    let selector = PathSelector::new(r0.name().to_string());

    // Verify selector can be created
    assert_eq!(selector.new("Rx0y0".to_string()).router_name, "Rx0y0");
}
```

**Step 2: Update PathSelector to expose router_name for testing**

Add to `src/forwarding/multipath.rs`:
```rust
impl PathSelector {
    pub fn router_name(&self) -> &str {
        &self.router_name
    }
}
```

**Step 3: Run test**

Run:
```bash
cargo test test_forwarding_engine_multipath_selection
```

Expected: PASS (or fix test as needed)

**Step 4: Commit multipath integration**

```bash
git add src/forwarding/multipath.rs tests/forwarding_test.rs
git commit -m "feat: expose PathSelector for integration with ForwardingEngine"
```

---

## Task 6: Add Counter Management for Per-Packet LB

**Files:**
- Create: `src/forwarding/counter.rs`
- Modify: `src/forwarding/mod.rs`
- Modify: `tests/multipath_test.rs`

**Step 1: Write test for packet counter**

Add to `tests/multipath_test.rs`:
```rust
use netsimulator::forwarding::PacketCounter;

#[test]
fn test_packet_counter() {
    let mut counter = PacketCounter::new();

    assert_eq!(counter.next(), 0);
    assert_eq!(counter.next(), 1);
    assert_eq!(counter.next(), 2);

    // Counter should wrap around eventually (but test with small numbers)
    for _ in 0..100 {
        counter.next();
    }
    assert_eq!(counter.next(), 103);
}
```

**Step 2: Implement packet counter**

Create `src/forwarding/counter.rs`:
```rust
use std::sync::atomic::{AtomicU64, Ordering};

/// Thread-safe packet counter for per-packet load balancing
pub struct PacketCounter {
    counter: AtomicU64,
}

impl PacketCounter {
    pub fn new() -> Self {
        PacketCounter {
            counter: AtomicU64::new(0),
        }
    }

    /// Get next counter value
    pub fn next(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }

    /// Get current counter value without incrementing
    pub fn current(&self) -> u64 {
        self.counter.load(Ordering::Relaxed)
    }
}

impl Default for PacketCounter {
    fn default() -> Self {
        Self::new()
    }
}
```

Update `src/forwarding/mod.rs`:
```rust
pub mod engine;
pub mod multipath;
pub mod counter;

pub use engine::{ForwardingEngine, TargetTun};
pub use multipath::{FiveTuple, PathSelector};
pub use counter::PacketCounter;
```

**Step 3: Run test to verify it passes**

Run:
```bash
cargo test test_packet_counter
```

Expected: PASS

**Step 4: Commit packet counter**

```bash
git add src/forwarding/counter.rs src/forwarding/mod.rs tests/multipath_test.rs
git commit -m "feat: add thread-safe packet counter for per-packet LB"
```

---

## Plan 8 Completion Checklist

Before moving to Plan 9, verify:

- [ ] All tests pass: `cargo test`
- [ ] 5-tuple extraction works for TCP, UDP, ICMP
- [ ] Path selection is consistent for same flow
- [ ] Path selection distributes across available paths
- [ ] Per-packet load balancing varies with counter
- [ ] Flow-based routing is sticky for same 5-tuple
- [ ] Packet counter is thread-safe

Run full test suite:
```bash
cargo test
```

**Next:** Proceed to Plan 9 (Integration and End-to-End Testing)

**Note:** The actual integration of multipath routing into the complete forwarding pipeline will be completed in Plan 9 when all components are brought together.
