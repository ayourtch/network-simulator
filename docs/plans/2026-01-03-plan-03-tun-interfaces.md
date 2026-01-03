# Plan 3: TUN Interface Management

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement TUN interface creation, async reading, and async writing on Linux hosts to enable packet injection and extraction from the virtual network fabric.

**Architecture:** Use the `tun` crate to manage TUN devices. Wrap TUN operations in async interfaces using tokio. Implement separate read and write loops that can run concurrently. Handle errors gracefully with proper cleanup.

**Tech Stack:** tun crate, tokio (async I/O), bytes crate for buffer management

---

## Task 1: Add TUN Dependencies and Basic Structure

**Files:**
- Modify: `Cargo.toml`
- Create: `src/tun/mod.rs`
- Create: `src/tun/interface.rs`
- Modify: `src/lib.rs`

**Step 1: Add tun crate dependency**

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

[dev-dependencies]
tempfile = "3.8"
```

**Step 2: Create tun module structure**

Create `src/tun/mod.rs`:
```rust
pub mod interface;

pub use interface::{TunInterface, TunError};
```

Update `src/lib.rs`:
```rust
pub mod config;
pub mod topology;
pub mod tun;
```

**Step 3: Build to verify dependencies**

Run:
```bash
cargo build
```

Expected: Success

**Step 4: Commit dependency addition**

```bash
git add Cargo.toml src/tun/mod.rs src/lib.rs
git commit -m "feat: add tun crate dependency and module structure"
```

---

## Task 2: Create TUN Interface Error Types

**Files:**
- Create: `src/tun/interface.rs`
- Create: `tests/tun_test.rs`

**Step 1: Write test for TUN interface creation (will require root)**

Create `tests/tun_test.rs`:
```rust
// Note: These tests require root privileges on Linux
// Run with: sudo -E cargo test

#[cfg(target_os = "linux")]
#[test]
#[ignore]  // Ignored by default, run with --ignored flag
fn test_tun_interface_creation() {
    use netsimulator::tun::TunInterface;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let result = TunInterface::new("test_tun0").await;
        assert!(result.is_ok());

        let tun = result.unwrap();
        assert_eq!(tun.name(), "test_tun0");
    });
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_tun_interface_creation
```

Expected: FAIL with "struct `TunInterface` not found"

**Step 3: Implement TunError and basic TunInterface**

Create `src/tun/interface.rs`:
```rust
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Error)]
pub enum TunError {
    #[error("Failed to create TUN device: {0}")]
    CreateError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid packet: {0}")]
    InvalidPacket(String),
}

/// Represents a TUN interface for packet injection/extraction
pub struct TunInterface {
    name: String,
    device: tun::AsyncDevice,
}

impl TunInterface {
    /// Create a new TUN interface with the given name
    pub async fn new(name: &str) -> Result<Self, TunError> {
        let mut config = tun::Configuration::default();
        config.name(name);
        config.up();

        #[cfg(target_os = "linux")]
        config.platform(|config| {
            config.packet_information(false);
        });

        let device = tun::create_as_async(&config)
            .map_err(|e| TunError::CreateError(format!("{}", e)))?;

        Ok(TunInterface {
            name: name.to_string(),
            device,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
```

**Step 4: Run test (note: requires root)**

Run:
```bash
# This test requires root privileges
sudo -E cargo test test_tun_interface_creation -- --ignored
```

Expected: PASS (if running as root on Linux), or skip if not on Linux

**Step 5: Commit TUN interface structure**

```bash
git add src/tun/interface.rs tests/tun_test.rs
git commit -m "feat: add TUN interface creation with error types"
```

---

## Task 3: Implement TUN Read Operations

**Files:**
- Modify: `src/tun/interface.rs`
- Modify: `tests/tun_test.rs`

**Step 1: Write test for TUN read (mock-based)**

Add to `tests/tun_test.rs`:
```rust
#[test]
fn test_packet_buffer_creation() {
    use netsimulator::tun::TunInterface;

    let buffer = TunInterface::create_read_buffer();
    assert_eq!(buffer.len(), 65536);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_packet_buffer_creation
```

Expected: FAIL with "no method named `create_read_buffer`"

**Step 3: Implement read buffer and read operation**

Add to `src/tun/interface.rs`:
```rust
const MAX_PACKET_SIZE: usize = 65536;

impl TunInterface {
    /// Create a buffer for reading packets
    pub fn create_read_buffer() -> Vec<u8> {
        vec![0u8; MAX_PACKET_SIZE]
    }

    /// Read a packet from the TUN interface
    /// Returns the number of bytes read
    pub async fn read_packet(&mut self, buffer: &mut [u8]) -> Result<usize, TunError> {
        let n = self.device.read(buffer).await?;
        Ok(n)
    }

    /// Read a packet and return it as a Vec<u8>
    pub async fn read_packet_owned(&mut self) -> Result<Vec<u8>, TunError> {
        let mut buffer = Self::create_read_buffer();
        let n = self.read_packet(&mut buffer).await?;
        buffer.truncate(n);
        Ok(buffer)
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_packet_buffer_creation
```

Expected: PASS

**Step 5: Write integration test for read (requires root)**

Add to `tests/tun_test.rs`:
```rust
#[cfg(target_os = "linux")]
#[test]
#[ignore]
fn test_tun_read_timeout() {
    use netsimulator::tun::TunInterface;
    use tokio::runtime::Runtime;
    use tokio::time::{timeout, Duration};

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut tun = TunInterface::new("test_tun1").await.unwrap();

        // Try to read with timeout (should timeout since no packets)
        let result = timeout(
            Duration::from_millis(100),
            tun.read_packet_owned()
        ).await;

        assert!(result.is_err());  // Should timeout
    });
}
```

**Step 6: Run integration test (requires root)**

Run:
```bash
sudo -E cargo test test_tun_read_timeout -- --ignored
```

Expected: PASS (timeouts as expected)

**Step 7: Commit TUN read operations**

```bash
git add src/tun/interface.rs tests/tun_test.rs
git commit -m "feat: implement TUN read operations with async support"
```

---

## Task 4: Implement TUN Write Operations

**Files:**
- Modify: `src/tun/interface.rs`
- Modify: `tests/tun_test.rs`

**Step 1: Write test for TUN write**

Add to `tests/tun_test.rs`:
```rust
#[test]
fn test_validate_packet_size() {
    use netsimulator::tun::TunInterface;

    // Too large packet should be detected
    let large_packet = vec![0u8; 70000];
    assert!(TunInterface::validate_packet_size(&large_packet).is_err());

    // Normal packet should be ok
    let normal_packet = vec![0u8; 1500];
    assert!(TunInterface::validate_packet_size(&normal_packet).is_ok());

    // Empty packet should fail
    let empty_packet = vec![];
    assert!(TunInterface::validate_packet_size(&empty_packet).is_err());
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_validate_packet_size
```

Expected: FAIL with "no method named `validate_packet_size`"

**Step 3: Implement write operation with validation**

Add to `src/tun/interface.rs`:
```rust
impl TunInterface {
    /// Validate packet size before writing
    pub fn validate_packet_size(packet: &[u8]) -> Result<(), TunError> {
        if packet.is_empty() {
            return Err(TunError::InvalidPacket("Packet is empty".to_string()));
        }
        if packet.len() > MAX_PACKET_SIZE {
            return Err(TunError::InvalidPacket(
                format!("Packet too large: {} bytes", packet.len())
            ));
        }
        Ok(())
    }

    /// Write a packet to the TUN interface
    pub async fn write_packet(&mut self, packet: &[u8]) -> Result<usize, TunError> {
        Self::validate_packet_size(packet)?;
        let n = self.device.write(packet).await?;
        Ok(n)
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_validate_packet_size
```

Expected: PASS

**Step 5: Commit TUN write operations**

```bash
git add src/tun/interface.rs tests/tun_test.rs
git commit -m "feat: implement TUN write operations with validation"
```

---

## Task 5: Create TUN Manager for Multiple Interfaces

**Files:**
- Create: `src/tun/manager.rs`
- Modify: `src/tun/mod.rs`
- Modify: `tests/tun_test.rs`

**Step 1: Write test for TUN manager**

Add to `tests/tun_test.rs`:
```rust
#[cfg(target_os = "linux")]
#[test]
#[ignore]
fn test_tun_manager_creation() {
    use netsimulator::tun::TunManager;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let result = TunManager::new("tun_a_test", "tun_b_test").await;
        assert!(result.is_ok());

        let manager = result.unwrap();
        assert_eq!(manager.tun_a_name(), "tun_a_test");
        assert_eq!(manager.tun_b_name(), "tun_b_test");
    });
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_tun_manager_creation
```

Expected: FAIL with "struct `TunManager` not found"

**Step 3: Implement TunManager**

Create `src/tun/manager.rs`:
```rust
use crate::tun::{TunInterface, TunError};
use tokio::sync::mpsc;

/// Manages both TUN interfaces (tunA and tunB)
pub struct TunManager {
    tun_a: TunInterface,
    tun_b: TunInterface,
}

impl TunManager {
    /// Create a new TunManager with two TUN interfaces
    pub async fn new(tun_a_name: &str, tun_b_name: &str) -> Result<Self, TunError> {
        let tun_a = TunInterface::new(tun_a_name).await?;
        let tun_b = TunInterface::new(tun_b_name).await?;

        Ok(TunManager { tun_a, tun_b })
    }

    pub fn tun_a_name(&self) -> &str {
        self.tun_a.name()
    }

    pub fn tun_b_name(&self) -> &str {
        self.tun_b.name()
    }

    /// Split into separate TUN interfaces for concurrent access
    pub fn split(self) -> (TunInterface, TunInterface) {
        (self.tun_a, self.tun_b)
    }
}
```

Update `src/tun/mod.rs`:
```rust
pub mod interface;
pub mod manager;

pub use interface::{TunInterface, TunError};
pub use manager::TunManager;
```

**Step 4: Run test (requires root)**

Run:
```bash
sudo -E cargo test test_tun_manager_creation -- --ignored
```

Expected: PASS

**Step 5: Commit TUN manager**

```bash
git add src/tun/manager.rs src/tun/mod.rs tests/tun_test.rs
git commit -m "feat: add TunManager for managing both TUN interfaces"
```

---

## Task 6: Create Packet Reader Task

**Files:**
- Create: `src/tun/reader.rs`
- Modify: `src/tun/mod.rs`
- Modify: `tests/tun_test.rs`

**Step 1: Write test for packet reader**

Add to `tests/tun_test.rs`:
```rust
use tokio::sync::mpsc;

#[test]
fn test_packet_message_creation() {
    use netsimulator::tun::PacketMessage;

    let packet = vec![0x45, 0x00, 0x00, 0x54];  // IPv4 header start
    let msg = PacketMessage::new(packet.clone(), "tunA");

    assert_eq!(msg.packet(), &packet);
    assert_eq!(msg.source_interface(), "tunA");
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_packet_message_creation
```

Expected: FAIL with "struct `PacketMessage` not found"

**Step 3: Implement PacketMessage and reader**

Create `src/tun/reader.rs`:
```rust
use crate::tun::{TunInterface, TunError};
use tokio::sync::mpsc;

/// Represents a packet read from a TUN interface
#[derive(Debug, Clone)]
pub struct PacketMessage {
    packet: Vec<u8>,
    source_interface: String,
}

impl PacketMessage {
    pub fn new(packet: Vec<u8>, source_interface: &str) -> Self {
        PacketMessage {
            packet,
            source_interface: source_interface.to_string(),
        }
    }

    pub fn packet(&self) -> &[u8] {
        &self.packet
    }

    pub fn source_interface(&self) -> &str {
        &self.source_interface
    }

    pub fn into_packet(self) -> Vec<u8> {
        self.packet
    }
}

/// Continuously read packets from a TUN interface and send to channel
pub async fn read_loop(
    mut tun: TunInterface,
    tx: mpsc::UnboundedSender<PacketMessage>,
) -> Result<(), TunError> {
    let tun_name = tun.name().to_string();

    loop {
        match tun.read_packet_owned().await {
            Ok(packet) => {
                let msg = PacketMessage::new(packet, &tun_name);
                if tx.send(msg).is_err() {
                    // Channel closed, stop reading
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading from {}: {}", tun_name, e);
                return Err(e);
            }
        }
    }

    Ok(())
}
```

Update `src/tun/mod.rs`:
```rust
pub mod interface;
pub mod manager;
pub mod reader;

pub use interface::{TunInterface, TunError};
pub use manager::TunManager;
pub use reader::{PacketMessage, read_loop};
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_packet_message_creation
```

Expected: PASS

**Step 5: Write integration test for read loop**

Add to `tests/tun_test.rs`:
```rust
#[cfg(target_os = "linux")]
#[test]
#[ignore]
fn test_read_loop_channel() {
    use netsimulator::tun::{TunInterface, read_loop};
    use tokio::runtime::Runtime;
    use tokio::sync::mpsc;
    use tokio::time::{timeout, Duration};

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let tun = TunInterface::new("test_tun2").await.unwrap();
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Spawn read loop in background
        let read_task = tokio::spawn(read_loop(tun, tx));

        // Try to receive with timeout (should timeout since no packets)
        let result = timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(result.is_err());  // Should timeout

        // Clean up
        drop(rx);  // Close channel to stop read loop
    });
}
```

**Step 6: Commit packet reader**

```bash
git add src/tun/reader.rs src/tun/mod.rs tests/tun_test.rs
git commit -m "feat: add async packet reader with channel support"
```

---

## Task 7: Integrate TUN into Main Application

**Files:**
- Modify: `src/main.rs`

**Step 1: Update main to create TUN interfaces**

Update `src/main.rs`:
```rust
use clap::Parser;
use netsimulator::config::NetworkConfig;
use netsimulator::topology::NetworkFabric;
use netsimulator::tun::TunManager;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "netsimulator")]
#[command(about = "Network simulator with virtual router fabric", long_about = None)]
struct Args {
    /// Path to the TOML configuration file
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,

    /// Skip TUN interface creation (for testing)
    #[arg(long)]
    no_tun: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("Loading configuration from: {}", args.config.display());
    let config = NetworkConfig::from_file(&args.config)?;

    println!("Configuration loaded successfully!");
    println!("  TUN A: {} (ingress: {})", config.global.tun_a, config.global.ingress_a);
    println!("  TUN B: {} (ingress: {})", config.global.tun_b, config.global.ingress_b);
    println!("  Links defined: {}", config.links.len());
    println!();

    println!("Building network fabric...");
    let fabric = NetworkFabric::from_config(&config)?;
    println!();

    fabric.print_summary();

    if !args.no_tun {
        println!("Creating TUN interfaces...");
        match TunManager::new(&config.global.tun_a, &config.global.tun_b).await {
            Ok(tun_manager) => {
                println!("TUN interfaces created successfully:");
                println!("  {}", tun_manager.tun_a_name());
                println!("  {}", tun_manager.tun_b_name());
                println!();
                println!("Network simulator is ready!");
                println!("Press Ctrl+C to stop...");

                // Keep running until interrupted
                tokio::signal::ctrl_c().await?;
                println!("\nShutting down...");
            }
            Err(e) => {
                eprintln!("Failed to create TUN interfaces: {}", e);
                eprintln!("Note: TUN interface creation requires root privileges.");
                eprintln!("Run with --no-tun to skip TUN creation for testing.");
                return Err(e.into());
            }
        }
    } else {
        println!("Skipping TUN interface creation (--no-tun flag)");
    }

    Ok(())
}
```

**Step 2: Test without TUN**

Run:
```bash
cargo run -- --config examples/simple_topology.toml --no-tun
```

Expected: Success without creating TUN interfaces

**Step 3: Test with TUN (requires root)**

Run:
```bash
sudo -E cargo run -- --config examples/simple_topology.toml
```

Expected: Creates TUN interfaces and waits for Ctrl+C

**Step 4: Commit main integration**

```bash
git add src/main.rs
git commit -m "feat: integrate TUN interface creation into main application"
```

---

## Task 8: Add TUN Documentation

**Files:**
- Create: `docs/tun_interfaces.md`

**Step 1: Write TUN interface documentation**

Create `docs/tun_interfaces.md`:
```markdown
# TUN Interface Management

## Overview

The network simulator uses Linux TUN (network TUNnel) devices to inject and extract packets from the virtual router fabric. Two TUN interfaces are created: tunA and tunB.

## Requirements

### Linux Only
TUN interfaces are Linux-specific. This simulator requires:
- Linux kernel 2.4 or later
- TUN/TAP kernel module enabled
- Root privileges to create TUN devices

### Verifying TUN Support

Check if TUN is available:
```bash
# Check for TUN module
lsmod | grep tun

# If not loaded, load it
sudo modprobe tun
```

## Architecture

### TunInterface
- Wraps a single TUN device
- Provides async read/write operations
- Validates packet sizes
- Handles errors gracefully

### TunManager
- Manages both tunA and tunB interfaces
- Creates interfaces on startup
- Provides access to individual interfaces

### PacketMessage
- Encapsulates packets read from TUN
- Tracks source interface (tunA or tunB)
- Used for routing packets into the fabric

### Read Loop
- Async task that continuously reads from TUN
- Sends packets to processing pipeline via channel
- Runs one per TUN interface

## Running the Simulator

### With TUN Interfaces (requires root)
```bash
sudo -E cargo run -- --config config.toml
```

The `-E` flag preserves environment variables (useful for Rust toolchain).

### Without TUN Interfaces (testing)
```bash
cargo run -- --config config.toml --no-tun
```

## Packet Flow

1. External application writes packet to tunA
2. Read loop receives packet
3. Packet wrapped in PacketMessage
4. Sent to ingress router (specified in config)
5. Routed through virtual fabric
6. Delivered to egress router
7. Written to tunB
8. External application reads from tunB

## Testing

Most TUN tests are marked `#[ignore]` because they require root privileges.

Run TUN tests:
```bash
sudo -E cargo test -- --ignored
```

## Limitations

- Linux only (no macOS/Windows support in v1)
- Requires root privileges
- No IPv6-specific handling yet (treats as opaque packets)
- Single topology per simulator instance in v1

## Troubleshooting

### Permission Denied
Error: "Failed to create TUN device: Permission denied"

Solution: Run with sudo or give binary CAP_NET_ADMIN capability:
```bash
sudo setcap cap_net_admin+ep target/debug/netsimulator
```

### TUN Module Not Available
Error: "Failed to create TUN device: No such device"

Solution: Load TUN module:
```bash
sudo modprobe tun
```

### Interface Already Exists
Error: "Failed to create TUN device: File exists"

Solution: Remove existing interface:
```bash
sudo ip link delete tunA
sudo ip link delete tunB
```
```

**Step 2: Commit documentation**

```bash
git add docs/tun_interfaces.md
git commit -m "docs: add TUN interface documentation"
```

---

## Plan 3 Completion Checklist

Before moving to Plan 4, verify:

- [ ] All unit tests pass: `cargo test`
- [ ] TUN interfaces can be created (requires root)
- [ ] Packets can be read from TUN (integration test)
- [ ] Packets can be written to TUN (integration test)
- [ ] TunManager manages both interfaces
- [ ] Read loop works with channels
- [ ] Main application creates TUN interfaces
- [ ] Documentation is complete

Run full test suite:
```bash
cargo test
sudo -E cargo test -- --ignored  # TUN-specific tests
cargo run -- --config examples/simple_topology.toml --no-tun
sudo -E cargo run -- --config examples/simple_topology.toml  # Ctrl+C to stop
```

**Next:** Proceed to Plan 4 (Packet Processing and Forwarding Engine)
