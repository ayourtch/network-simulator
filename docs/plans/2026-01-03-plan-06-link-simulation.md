# Plan 6: Link Simulation (MTU, Delay, Jitter, Loss)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement realistic network link characteristics including MTU enforcement, delay, jitter (delay variation), and packet loss simulation.

**Architecture:** Create a link simulator that wraps packet transmission between routers. Use tokio::time for async delays. Use rand crate for jitter and loss simulation. Validate MTU before transmission.

**Tech Stack:** tokio::time::sleep for delays, rand crate for randomization, statistics for jitter distribution

---

## Task 1: Add Simulation Dependencies

**Files:**
- Modify: `Cargo.toml`
- Create: `src/simulation/mod.rs`
- Create: `src/simulation/link.rs`
- Modify: `src/lib.rs`

**Step 1: Add rand dependency**

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
rand = "0.8"

[dev-dependencies]
tempfile = "3.8"
```

**Step 2: Create simulation module**

Create `src/simulation/mod.rs`:
```rust
pub mod link;

pub use link::{LinkSimulator, SimulationError};
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
```

**Step 3: Build to verify**

Run:
```bash
cargo build
```

Expected: Compilation error (link.rs not created)

**Step 4: Create empty link simulator**

Create `src/simulation/link.rs`:
```rust
// Link simulation implementation
```

**Step 5: Build again**

Run:
```bash
cargo build
```

Expected: Success

**Step 6: Commit simulation module structure**

```bash
git add Cargo.toml src/simulation/ src/lib.rs
git commit -m "feat: add simulation module structure with rand dependency"
```

---

## Task 2: Implement MTU Validation

**Files:**
- Modify: `src/simulation/link.rs`
- Create: `tests/simulation_test.rs`

**Step 1: Write test for MTU validation**

Create `tests/simulation_test.rs`:
```rust
use netsimulator::simulation::{LinkSimulator, SimulationError};

#[test]
fn test_mtu_validation() {
    let simulator = LinkSimulator::new(1500, 0.0, 0.0, 0.0);

    // Packet within MTU should pass
    let small_packet = vec![0u8; 1400];
    assert!(simulator.validate_mtu(&small_packet).is_ok());

    // Packet exactly at MTU should pass
    let exact_packet = vec![0u8; 1500];
    assert!(simulator.validate_mtu(&exact_packet).is_ok());

    // Packet exceeding MTU should fail
    let large_packet = vec![0u8; 1501];
    assert!(simulator.validate_mtu(&large_packet).is_err());
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_mtu_validation
```

Expected: FAIL with "struct `LinkSimulator` not found"

**Step 3: Implement LinkSimulator with MTU validation**

Update `src/simulation/link.rs`:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SimulationError {
    #[error("Packet size {0} exceeds MTU {1}")]
    MtuExceeded(usize, u32),

    #[error("Packet dropped due to simulated loss")]
    PacketDropped,
}

/// Simulates network link characteristics
#[derive(Debug, Clone)]
pub struct LinkSimulator {
    mtu: u32,
    delay_ms: f64,
    jitter_ms: f64,
    loss_percent: f64,
}

impl LinkSimulator {
    pub fn new(mtu: u32, delay_ms: f64, jitter_ms: f64, loss_percent: f64) -> Self {
        LinkSimulator {
            mtu,
            delay_ms,
            jitter_ms,
            loss_percent,
        }
    }

    /// Validate that packet size does not exceed MTU
    pub fn validate_mtu(&self, packet: &[u8]) -> Result<(), SimulationError> {
        if packet.len() > self.mtu as usize {
            return Err(SimulationError::MtuExceeded(packet.len(), self.mtu));
        }
        Ok(())
    }

    pub fn mtu(&self) -> u32 {
        self.mtu
    }

    pub fn delay_ms(&self) -> f64 {
        self.delay_ms
    }

    pub fn jitter_ms(&self) -> f64 {
        self.jitter_ms
    }

    pub fn loss_percent(&self) -> f64 {
        self.loss_percent
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_mtu_validation
```

Expected: PASS

**Step 5: Commit MTU validation**

```bash
git add src/simulation/link.rs tests/simulation_test.rs
git commit -m "feat: implement MTU validation in LinkSimulator"
```

---

## Task 3: Implement Packet Loss Simulation

**Files:**
- Modify: `src/simulation/link.rs`
- Modify: `tests/simulation_test.rs`

**Step 1: Write test for packet loss**

Add to `tests/simulation_test.rs`:
```rust
#[test]
fn test_packet_loss_simulation() {
    // 100% loss - all packets should be dropped
    let simulator = LinkSimulator::new(1500, 0.0, 0.0, 100.0);

    let mut dropped_count = 0;
    for _ in 0..100 {
        if simulator.should_drop_packet() {
            dropped_count += 1;
        }
    }

    assert_eq!(dropped_count, 100);

    // 0% loss - no packets should be dropped
    let simulator = LinkSimulator::new(1500, 0.0, 0.0, 0.0);

    dropped_count = 0;
    for _ in 0..100 {
        if simulator.should_drop_packet() {
            dropped_count += 1;
        }
    }

    assert_eq!(dropped_count, 0);
}

#[test]
fn test_packet_loss_approximate() {
    // 50% loss - approximately half should be dropped
    let simulator = LinkSimulator::new(1500, 0.0, 0.0, 50.0);

    let mut dropped_count = 0;
    let iterations = 1000;

    for _ in 0..iterations {
        if simulator.should_drop_packet() {
            dropped_count += 1;
        }
    }

    // Allow 10% margin of error (should be between 450-550 drops)
    assert!(dropped_count > 450 && dropped_count < 550,
            "Expected ~500 drops, got {}", dropped_count);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_packet_loss_simulation
```

Expected: FAIL with "no method named `should_drop_packet`"

**Step 3: Implement packet loss simulation**

Add to `src/simulation/link.rs`:
```rust
use rand::Rng;

impl LinkSimulator {
    /// Check if packet should be dropped based on loss percentage
    pub fn should_drop_packet(&self) -> bool {
        if self.loss_percent <= 0.0 {
            return false;
        }
        if self.loss_percent >= 100.0 {
            return true;
        }

        let mut rng = rand::thread_rng();
        let roll: f64 = rng.gen_range(0.0..100.0);
        roll < self.loss_percent
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_packet_loss_simulation
cargo test test_packet_loss_approximate
```

Expected: PASS

**Step 5: Commit packet loss simulation**

```bash
git add src/simulation/link.rs tests/simulation_test.rs
git commit -m "feat: implement packet loss simulation with randomization"
```

---

## Task 4: Implement Delay Simulation

**Files:**
- Modify: `src/simulation/link.rs`
- Modify: `tests/simulation_test.rs`

**Step 1: Write test for delay calculation**

Add to `tests/simulation_test.rs`:
```rust
#[test]
fn test_delay_calculation() {
    // Fixed delay with no jitter
    let simulator = LinkSimulator::new(1500, 10.0, 0.0, 0.0);

    for _ in 0..10 {
        let delay = simulator.calculate_delay();
        assert_eq!(delay, 10.0);
    }
}

#[test]
fn test_delay_with_jitter() {
    // Delay with jitter should vary
    let simulator = LinkSimulator::new(1500, 10.0, 5.0, 0.0);

    let mut delays = Vec::new();
    for _ in 0..100 {
        delays.push(simulator.calculate_delay());
    }

    // Check that we have variance
    let min_delay = delays.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_delay = delays.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    // With jitter of 5ms, delays should vary
    assert!(max_delay > min_delay);

    // Average should be close to base delay
    let avg_delay: f64 = delays.iter().sum::<f64>() / delays.len() as f64;
    assert!((avg_delay - 10.0).abs() < 2.0);  // Within 2ms of expected
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_delay_calculation
```

Expected: FAIL with "no method named `calculate_delay`"

**Step 3: Implement delay calculation with jitter**

Add to `src/simulation/link.rs`:
```rust
impl LinkSimulator {
    /// Calculate delay with jitter applied
    /// Jitter is uniformly distributed: delay +/- jitter
    pub fn calculate_delay(&self) -> f64 {
        if self.jitter_ms <= 0.0 {
            return self.delay_ms;
        }

        let mut rng = rand::thread_rng();
        let jitter_delta = rng.gen_range(-self.jitter_ms..=self.jitter_ms);
        (self.delay_ms + jitter_delta).max(0.0)  // Ensure non-negative
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_delay_calculation
cargo test test_delay_with_jitter
```

Expected: PASS

**Step 5: Write test for async delay application**

Add to `tests/simulation_test.rs`:
```rust
use tokio::time::{Instant, Duration};

#[tokio::test]
async fn test_apply_delay_async() {
    let simulator = LinkSimulator::new(1500, 50.0, 0.0, 0.0);

    let start = Instant::now();
    simulator.apply_delay().await;
    let elapsed = start.elapsed();

    // Should be approximately 50ms (allow some tolerance)
    assert!(elapsed >= Duration::from_millis(45));
    assert!(elapsed <= Duration::from_millis(55));
}
```

**Step 6: Implement async delay application**

Add to `src/simulation/link.rs`:
```rust
use tokio::time::{sleep, Duration};

impl LinkSimulator {
    /// Apply delay asynchronously
    pub async fn apply_delay(&self) {
        let delay = self.calculate_delay();
        if delay > 0.0 {
            sleep(Duration::from_micros((delay * 1000.0) as u64)).await;
        }
    }
}
```

**Step 7: Run test**

Run:
```bash
cargo test test_apply_delay_async
```

Expected: PASS

**Step 8: Commit delay simulation**

```bash
git add src/simulation/link.rs tests/simulation_test.rs
git commit -m "feat: implement delay and jitter simulation with async support"
```

---

## Task 5: Implement Complete Link Simulation

**Files:**
- Modify: `src/simulation/link.rs`
- Modify: `tests/simulation_test.rs`

**Step 1: Write test for complete simulation**

Add to `tests/simulation_test.rs`:
```rust
#[tokio::test]
async fn test_simulate_link_traversal_success() {
    let simulator = LinkSimulator::new(1500, 10.0, 0.0, 0.0);
    let packet = vec![0u8; 1000];

    let start = Instant::now();
    let result = simulator.simulate_link_traversal(&packet).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    assert!(elapsed >= Duration::from_millis(9));
}

#[tokio::test]
async fn test_simulate_link_traversal_mtu_exceeded() {
    let simulator = LinkSimulator::new(1500, 10.0, 0.0, 0.0);
    let packet = vec![0u8; 2000];  // Exceeds MTU

    let result = simulator.simulate_link_traversal(&packet).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SimulationError::MtuExceeded(_, _)));
}

#[tokio::test]
async fn test_simulate_link_traversal_packet_loss() {
    let simulator = LinkSimulator::new(1500, 1.0, 0.0, 100.0);  // 100% loss
    let packet = vec![0u8; 1000];

    let result = simulator.simulate_link_traversal(&packet).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SimulationError::PacketDropped));
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_simulate_link_traversal_success
```

Expected: FAIL with "no method named `simulate_link_traversal`"

**Step 3: Implement complete link simulation**

Add to `src/simulation/link.rs`:
```rust
impl LinkSimulator {
    /// Simulate complete link traversal
    /// Returns Ok(()) if packet successfully traverses link
    /// Returns Err if packet is dropped or MTU exceeded
    pub async fn simulate_link_traversal(&self, packet: &[u8]) -> Result<(), SimulationError> {
        // Check MTU
        self.validate_mtu(packet)?;

        // Check packet loss
        if self.should_drop_packet() {
            return Err(SimulationError::PacketDropped);
        }

        // Apply delay
        self.apply_delay().await;

        Ok(())
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_simulate_link_traversal_success
cargo test test_simulate_link_traversal_mtu_exceeded
cargo test test_simulate_link_traversal_packet_loss
```

Expected: PASS

**Step 5: Commit complete link simulation**

```bash
git add src/simulation/link.rs tests/simulation_test.rs
git commit -m "feat: implement complete link traversal simulation"
```

---

## Task 6: Create Link Simulator from Configuration

**Files:**
- Modify: `src/simulation/link.rs`
- Modify: `src/topology/link.rs`
- Modify: `tests/simulation_test.rs`

**Step 1: Write test for creating simulator from link config**

Add to `tests/simulation_test.rs`:
```rust
use netsimulator::topology::{Router, Link};

#[test]
fn test_link_simulator_from_link() {
    let r1 = Router::new(0, 0);
    let r2 = Router::new(0, 1);

    let link = Link::new(r1, r2, 1400, 5.0, 1.0, 0.5, false);
    let simulator = LinkSimulator::from_link(&link);

    assert_eq!(simulator.mtu(), 1400);
    assert_eq!(simulator.delay_ms(), 5.0);
    assert_eq!(simulator.jitter_ms(), 1.0);
    assert_eq!(simulator.loss_percent(), 0.5);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_link_simulator_from_link
```

Expected: FAIL with "no function named `from_link`"

**Step 3: Implement from_link constructor**

Add to `src/simulation/link.rs`:
```rust
use crate::topology::Link;

impl LinkSimulator {
    /// Create a LinkSimulator from a topology Link
    pub fn from_link(link: &Link) -> Self {
        LinkSimulator::new(
            link.mtu(),
            link.delay_ms(),
            link.jitter_ms(),
            link.loss_percent(),
        )
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_link_simulator_from_link
```

Expected: PASS

**Step 5: Commit from_link constructor**

```bash
git add src/simulation/link.rs tests/simulation_test.rs
git commit -m "feat: add LinkSimulator::from_link constructor"
```

---

## Task 7: Add Simulation Statistics (Optional Enhancement)

**Files:**
- Modify: `src/simulation/link.rs`
- Create: `src/simulation/stats.rs`
- Modify: `src/simulation/mod.rs`

**Step 1: Write test for statistics tracking**

Add to `tests/simulation_test.rs`:
```rust
use netsimulator::simulation::LinkStats;

#[test]
fn test_link_statistics() {
    let mut stats = LinkStats::new();

    stats.record_packet_sent(1000);
    stats.record_packet_sent(1500);
    stats.record_packet_dropped();

    assert_eq!(stats.packets_sent(), 2);
    assert_eq!(stats.packets_dropped(), 1);
    assert_eq!(stats.bytes_sent(), 2500);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_link_statistics
```

Expected: FAIL with "struct `LinkStats` not found"

**Step 3: Implement statistics tracking**

Create `src/simulation/stats.rs`:
```rust
/// Statistics for a simulated link
#[derive(Debug, Clone, Default)]
pub struct LinkStats {
    packets_sent: u64,
    packets_dropped: u64,
    bytes_sent: u64,
}

impl LinkStats {
    pub fn new() -> Self {
        LinkStats::default()
    }

    pub fn record_packet_sent(&mut self, size: usize) {
        self.packets_sent += 1;
        self.bytes_sent += size as u64;
    }

    pub fn record_packet_dropped(&mut self) {
        self.packets_dropped += 1;
    }

    pub fn packets_sent(&self) -> u64 {
        self.packets_sent
    }

    pub fn packets_dropped(&self) -> u64 {
        self.packets_dropped
    }

    pub fn bytes_sent(&self) -> u64 {
        self.bytes_sent
    }

    pub fn loss_rate(&self) -> f64 {
        let total = self.packets_sent + self.packets_dropped;
        if total == 0 {
            0.0
        } else {
            (self.packets_dropped as f64 / total as f64) * 100.0
        }
    }
}
```

Update `src/simulation/mod.rs`:
```rust
pub mod link;
pub mod stats;

pub use link::{LinkSimulator, SimulationError};
pub use stats::LinkStats;
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_link_statistics
```

Expected: PASS

**Step 5: Commit statistics tracking**

```bash
git add src/simulation/stats.rs src/simulation/mod.rs tests/simulation_test.rs
git commit -m "feat: add link statistics tracking"
```

---

## Plan 6 Completion Checklist

Before moving to Plan 7, verify:

- [ ] All tests pass: `cargo test`
- [ ] MTU validation works correctly
- [ ] Packet loss simulation is probabilistically correct
- [ ] Delay simulation with jitter works
- [ ] Async delay application works
- [ ] Complete link traversal simulation works
- [ ] Can create LinkSimulator from topology Link
- [ ] Statistics tracking works (optional)

Run full test suite:
```bash
cargo test
```

**Next:** Proceed to Plan 7 (ICMP Error Generation)
