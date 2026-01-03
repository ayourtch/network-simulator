# Plan 1: Project Setup and Configuration Parsing

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Initialize the Rust project structure with all required dependencies and implement TOML configuration file parsing with validation.

**Architecture:** Use serde and toml crates for configuration deserialization. Implement validation logic to detect bidirectional link conflicts and ensure configuration consistency.

**Tech Stack:** Rust, cargo, serde, toml, thiserror (for error handling)

---

## Task 1: Initialize Rust Project

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`

**Step 1: Create new Rust project**

Run in terminal:
```bash
cargo init --name netsimulator
```

Expected output: "Created binary (application) package"

**Step 2: Update Cargo.toml with dependencies**

Edit `Cargo.toml`:
```toml
[package]
name = "netsimulator"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
thiserror = "1.0"
anyhow = "1.0"

[dev-dependencies]
tempfile = "3.8"
```

**Step 3: Verify project builds**

Run:
```bash
cargo build
```

Expected: "Finished dev [unoptimized + debuginfo] target(s)"

**Step 4: Commit initial project**

```bash
git init
git add .
git commit -m "feat: initialize Rust project with dependencies"
```

---

## Task 2: Create Configuration Data Structures

**Files:**
- Create: `src/config/mod.rs`
- Create: `src/config/types.rs`
- Create: `tests/config_test.rs`

**Step 1: Write test for basic config parsing**

Create `tests/config_test.rs`:
```rust
use netsimulator::config::NetworkConfig;

#[test]
fn test_parse_empty_config() {
    let toml_str = r#"
        [global]
        tun_a = "tunA"
        tun_b = "tunB"
        ingress_a = "Rx0y0"
        ingress_b = "Rx5y5"
    "#;

    let config = NetworkConfig::from_toml(toml_str);
    assert!(config.is_ok());
    let config = config.unwrap();
    assert_eq!(config.global.tun_a, "tunA");
    assert_eq!(config.global.tun_b, "tunB");
    assert_eq!(config.global.ingress_a, "Rx0y0");
    assert_eq!(config.global.ingress_b, "Rx5y5");
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_parse_empty_config
```

Expected: FAIL with "module `config` not found" or similar

**Step 3: Create config module structure**

Create `src/config/mod.rs`:
```rust
pub mod types;

pub use types::NetworkConfig;
```

Update `src/lib.rs`:
```rust
pub mod config;
```

**Step 4: Implement basic config types**

Create `src/config/types.rs`:
```rust
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("TOML parse error: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Deserialize, Clone)]
pub struct NetworkConfig {
    pub global: GlobalConfig,
    #[serde(flatten)]
    pub links: HashMap<String, LinkConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GlobalConfig {
    pub tun_a: String,
    pub tun_b: String,
    pub ingress_a: String,
    pub ingress_b: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LinkConfig {
    #[serde(default = "default_mtu")]
    pub mtu: u32,

    #[serde(default)]
    pub delay_ms: f64,

    #[serde(default)]
    pub jitter_ms: f64,

    #[serde(default)]
    pub loss_percent: f64,

    #[serde(default)]
    pub per_packet_lb: bool,
}

fn default_mtu() -> u32 {
    1500
}

impl NetworkConfig {
    pub fn from_toml(toml_str: &str) -> Result<Self, ConfigError> {
        let config: NetworkConfig = toml::from_str(toml_str)?;
        Ok(config)
    }
}
```

**Step 5: Run test to verify it passes**

Run:
```bash
cargo test test_parse_empty_config
```

Expected: PASS

**Step 6: Commit config parsing**

```bash
git add src/config/ src/lib.rs tests/config_test.rs
git commit -m "feat: add basic TOML config parsing"
```

---

## Task 3: Parse Link Configurations

**Files:**
- Modify: `tests/config_test.rs`
- Modify: `src/config/types.rs`

**Step 1: Write test for link parsing**

Add to `tests/config_test.rs`:
```rust
#[test]
fn test_parse_link_config() {
    let toml_str = r#"
        [global]
        tun_a = "tunA"
        tun_b = "tunB"
        ingress_a = "Rx0y0"
        ingress_b = "Rx5y5"

        [Rx0y0_Rx0y1]
        mtu = 1400
        delay_ms = 5.0
        jitter_ms = 1.0
        loss_percent = 0.1
        per_packet_lb = false
    "#;

    let config = NetworkConfig::from_toml(toml_str).unwrap();
    assert_eq!(config.links.len(), 1);

    let link = config.links.get("Rx0y0_Rx0y1").unwrap();
    assert_eq!(link.mtu, 1400);
    assert_eq!(link.delay_ms, 5.0);
    assert_eq!(link.jitter_ms, 1.0);
    assert_eq!(link.loss_percent, 0.1);
    assert_eq!(link.per_packet_lb, false);
}
```

**Step 2: Run test**

Run:
```bash
cargo test test_parse_link_config
```

Expected: PASS (should already work with current implementation)

**Step 3: Write test for default values**

Add to `tests/config_test.rs`:
```rust
#[test]
fn test_link_defaults() {
    let toml_str = r#"
        [global]
        tun_a = "tunA"
        tun_b = "tunB"
        ingress_a = "Rx0y0"
        ingress_b = "Rx5y5"

        [Rx0y0_Rx0y1]
    "#;

    let config = NetworkConfig::from_toml(toml_str).unwrap();
    let link = config.links.get("Rx0y0_Rx0y1").unwrap();

    assert_eq!(link.mtu, 1500);  // default
    assert_eq!(link.delay_ms, 0.0);  // default
    assert_eq!(link.jitter_ms, 0.0);  // default
    assert_eq!(link.loss_percent, 0.0);  // default
    assert_eq!(link.per_packet_lb, false);  // default
}
```

**Step 4: Run test**

Run:
```bash
cargo test test_link_defaults
```

Expected: PASS

**Step 5: Commit link parsing**

```bash
git add tests/config_test.rs
git commit -m "test: add link configuration parsing tests"
```

---

## Task 4: Implement Bidirectional Link Detection

**Files:**
- Modify: `src/config/types.rs`
- Modify: `tests/config_test.rs`

**Step 1: Write test for duplicate link detection**

Add to `tests/config_test.rs`:
```rust
#[test]
fn test_detect_bidirectional_links() {
    let toml_str = r#"
        [global]
        tun_a = "tunA"
        tun_b = "tunB"
        ingress_a = "Rx0y0"
        ingress_b = "Rx5y5"

        [Rx0y0_Rx0y1]
        mtu = 1400

        [Rx0y1_Rx0y0]
        mtu = 1500
    "#;

    let result = NetworkConfig::from_toml(toml_str);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, netsimulator::config::ConfigError::ValidationError(_)));
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_detect_bidirectional_links
```

Expected: FAIL (currently doesn't detect duplicates)

**Step 3: Implement bidirectional link validation**

Modify `src/config/types.rs`, update the `from_toml` method:
```rust
impl NetworkConfig {
    pub fn from_toml(toml_str: &str) -> Result<Self, ConfigError> {
        let config: NetworkConfig = toml::from_str(toml_str)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        self.validate_bidirectional_links()?;
        Ok(())
    }

    fn validate_bidirectional_links(&self) -> Result<(), ConfigError> {
        use std::collections::HashSet;

        let mut seen_links: HashSet<(String, String)> = HashSet::new();

        for link_name in self.links.keys() {
            let parts: Vec<&str> = link_name.split('_').collect();
            if parts.len() != 2 {
                return Err(ConfigError::ValidationError(
                    format!("Invalid link name format: {}. Expected 'RouterA_RouterB'", link_name)
                ));
            }

            let router_a = parts[0].to_string();
            let router_b = parts[1].to_string();

            // Create both directions
            let forward = (router_a.clone(), router_b.clone());
            let reverse = (router_b, router_a);

            // Check if we've seen the reverse direction
            if seen_links.contains(&reverse) {
                return Err(ConfigError::ValidationError(
                    format!("Duplicate bidirectional link detected: {} (conflicts with reverse direction)", link_name)
                ));
            }

            seen_links.insert(forward);
        }

        Ok(())
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_detect_bidirectional_links
```

Expected: PASS

**Step 5: Write test for valid bidirectional scenario**

Add to `tests/config_test.rs`:
```rust
#[test]
fn test_valid_multiple_links() {
    let toml_str = r#"
        [global]
        tun_a = "tunA"
        tun_b = "tunB"
        ingress_a = "Rx0y0"
        ingress_b = "Rx5y5"

        [Rx0y0_Rx0y1]
        mtu = 1400

        [Rx0y1_Rx0y2]
        mtu = 1500

        [Rx1y0_Rx1y1]
        mtu = 1450
    "#;

    let config = NetworkConfig::from_toml(toml_str);
    assert!(config.is_ok());
    assert_eq!(config.unwrap().links.len(), 3);
}
```

**Step 6: Run test**

Run:
```bash
cargo test test_valid_multiple_links
```

Expected: PASS

**Step 7: Commit bidirectional link validation**

```bash
git add src/config/types.rs tests/config_test.rs
git commit -m "feat: add bidirectional link duplicate detection"
```

---

## Task 5: Add Configuration File Loading

**Files:**
- Modify: `src/config/types.rs`
- Create: `tests/fixtures/example_config.toml`
- Modify: `tests/config_test.rs`

**Step 1: Create example configuration file**

Create `tests/fixtures/example_config.toml`:
```toml
[global]
tun_a = "tunA"
tun_b = "tunB"
ingress_a = "Rx0y0"
ingress_b = "Rx5y5"

# Links in the 6x6 fabric
# Row 0 connections
[Rx0y0_Rx0y1]
mtu = 1500
delay_ms = 2.0
jitter_ms = 0.5
loss_percent = 0.0

[Rx0y0_Rx1y0]
mtu = 1500
delay_ms = 2.0
jitter_ms = 0.5

# Add more links as needed for testing
```

**Step 2: Write test for file loading**

Add to `tests/config_test.rs`:
```rust
use std::path::Path;

#[test]
fn test_load_from_file() {
    let path = Path::new("tests/fixtures/example_config.toml");
    let config = NetworkConfig::from_file(path);
    assert!(config.is_ok());

    let config = config.unwrap();
    assert_eq!(config.global.tun_a, "tunA");
    assert_eq!(config.links.len(), 2);
}
```

**Step 3: Run test to verify it fails**

Run:
```bash
cargo test test_load_from_file
```

Expected: FAIL with "no method named `from_file`"

**Step 4: Implement file loading**

Add to `src/config/types.rs`:
```rust
use std::path::Path;
use std::fs;

impl NetworkConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)
            .map_err(|e| ConfigError::ValidationError(format!("Failed to read file: {}", e)))?;
        Self::from_toml(&contents)
    }
}
```

**Step 5: Run test to verify it passes**

Run:
```bash
cargo test test_load_from_file
```

Expected: PASS

**Step 6: Commit file loading**

```bash
mkdir -p tests/fixtures
git add src/config/types.rs tests/config_test.rs tests/fixtures/example_config.toml
git commit -m "feat: add config file loading from disk"
```

---

## Task 6: Validate Router Names

**Files:**
- Modify: `src/config/types.rs`
- Modify: `tests/config_test.rs`

**Step 1: Write test for router name validation**

Add to `tests/config_test.rs`:
```rust
#[test]
fn test_validate_router_names() {
    let toml_str = r#"
        [global]
        tun_a = "tunA"
        tun_b = "tunB"
        ingress_a = "InvalidRouter"
        ingress_b = "Rx5y5"
    "#;

    let result = NetworkConfig::from_toml(toml_str);
    assert!(result.is_err());
}

#[test]
fn test_valid_router_names() {
    let toml_str = r#"
        [global]
        tun_a = "tunA"
        tun_b = "tunB"
        ingress_a = "Rx0y0"
        ingress_b = "Rx5y5"
    "#;

    let result = NetworkConfig::from_toml(toml_str);
    assert!(result.is_ok());
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_validate_router_names
```

Expected: FAIL (currently no validation)

**Step 3: Implement router name validation**

Add to `src/config/types.rs`:
```rust
impl NetworkConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        self.validate_bidirectional_links()?;
        self.validate_router_names()?;
        Ok(())
    }

    fn validate_router_names(&self) -> Result<(), ConfigError> {
        self.validate_router_name(&self.global.ingress_a, "ingress_a")?;
        self.validate_router_name(&self.global.ingress_b, "ingress_b")?;

        // Validate router names in link definitions
        for link_name in self.links.keys() {
            let parts: Vec<&str> = link_name.split('_').collect();
            if parts.len() == 2 {
                self.validate_router_name(parts[0], &format!("link {}", link_name))?;
                self.validate_router_name(parts[1], &format!("link {}", link_name))?;
            }
        }

        Ok(())
    }

    fn validate_router_name(&self, name: &str, context: &str) -> Result<(), ConfigError> {
        // Router names must match pattern: Rx[0-5]y[0-5]
        let re = regex::Regex::new(r"^Rx[0-5]y[0-5]$").unwrap();
        if !re.is_match(name) {
            return Err(ConfigError::ValidationError(
                format!("Invalid router name '{}' in {}: must match pattern Rx[0-5]y[0-5]", name, context)
            ));
        }
        Ok(())
    }
}
```

**Step 4: Add regex dependency**

Update `Cargo.toml` dependencies section:
```toml
[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
thiserror = "1.0"
anyhow = "1.0"
regex = "1.10"
```

**Step 5: Run test to verify it passes**

Run:
```bash
cargo test test_validate_router_names
```

Expected: PASS

**Step 6: Commit router name validation**

```bash
git add Cargo.toml src/config/types.rs tests/config_test.rs
git commit -m "feat: add router name validation for 6x6 grid"
```

---

## Task 7: Add CLI Argument Parsing

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs`

**Step 1: Add clap dependency**

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
```

**Step 2: Implement CLI parsing**

Update `src/main.rs`:
```rust
use clap::Parser;
use netsimulator::config::NetworkConfig;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "netsimulator")]
#[command(about = "Network simulator with virtual router fabric", long_about = None)]
struct Args {
    /// Path to the TOML configuration file
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,
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

    Ok(())
}
```

**Step 3: Test CLI manually**

Run:
```bash
cargo build
./target/debug/netsimulator --config tests/fixtures/example_config.toml
```

Expected output:
```
Loading configuration from: tests/fixtures/example_config.toml
Configuration loaded successfully!
  TUN A: tunA (ingress: Rx0y0)
  TUN B: tunB (ingress: Rx5y5)
  Links defined: 2
```

**Step 4: Test help message**

Run:
```bash
./target/debug/netsimulator --help
```

Expected: Help message with --config option described

**Step 5: Commit CLI parsing**

```bash
git add Cargo.toml src/main.rs
git commit -m "feat: add CLI argument parsing for config file"
```

---

## Task 8: Create Complete Example Configuration

**Files:**
- Create: `examples/full_fabric.toml`
- Create: `examples/simple_topology.toml`

**Step 1: Create full 6x6 fabric example**

Create `examples/full_fabric.toml`:
```toml
[global]
tun_a = "tunA"
tun_b = "tunB"
ingress_a = "Rx0y0"
ingress_b = "Rx5y5"

# This example shows a fully connected 6x6 mesh
# Each router connects to its neighbors (up, down, left, right)

# Row 0 - horizontal links
[Rx0y0_Rx0y1]
mtu = 1500
delay_ms = 2.0
jitter_ms = 0.5

[Rx0y1_Rx0y2]
mtu = 1500
delay_ms = 2.0

[Rx0y2_Rx0y3]
mtu = 1500
delay_ms = 2.0

[Rx0y3_Rx0y4]
mtu = 1500
delay_ms = 2.0

[Rx0y4_Rx0y5]
mtu = 1500
delay_ms = 2.0

# Row 0 - vertical links
[Rx0y0_Rx1y0]
mtu = 1500
delay_ms = 2.0

[Rx0y1_Rx1y1]
mtu = 1500
delay_ms = 2.0

[Rx0y2_Rx1y2]
mtu = 1500
delay_ms = 2.0

[Rx0y3_Rx1y3]
mtu = 1500
delay_ms = 2.0

[Rx0y4_Rx1y4]
mtu = 1500
delay_ms = 2.0

[Rx0y5_Rx1y5]
mtu = 1500
delay_ms = 2.0

# Continue pattern for rows 1-5...
# (Add all remaining links following the same pattern)
# For brevity, showing structure only

# Example of link with packet loss
[Rx2y2_Rx2y3]
mtu = 1400
delay_ms = 10.0
jitter_ms = 2.0
loss_percent = 0.5

# Example of per-packet load balancing
[Rx3y3_Rx3y4]
mtu = 1500
delay_ms = 1.0
per_packet_lb = true
```

**Step 2: Create simple topology example**

Create `examples/simple_topology.toml`:
```toml
[global]
tun_a = "tunA"
tun_b = "tunB"
ingress_a = "Rx0y0"
ingress_b = "Rx2y2"

# Simple linear topology for testing
# tunA -> Rx0y0 -> Rx0y1 -> Rx0y2 -> tunB
#                           |
#                         Rx1y2 -> Rx2y2

[Rx0y0_Rx0y1]
mtu = 1500
delay_ms = 5.0

[Rx0y1_Rx0y2]
mtu = 1500
delay_ms = 5.0

[Rx0y2_Rx1y2]
mtu = 1500
delay_ms = 5.0

[Rx1y2_Rx2y2]
mtu = 1500
delay_ms = 5.0
```

**Step 3: Test loading example configs**

Run:
```bash
cargo run -- --config examples/simple_topology.toml
```

Expected: Success message

Run:
```bash
cargo run -- --config examples/full_fabric.toml
```

Expected: Success message

**Step 4: Commit examples**

```bash
git add examples/
git commit -m "docs: add example configuration files"
```

---

## Task 9: Add Configuration Documentation

**Files:**
- Create: `docs/configuration.md`

**Step 1: Write configuration documentation**

Create `docs/configuration.md`:
```markdown
# Network Simulator Configuration Guide

## Configuration File Format

The network simulator uses TOML format for configuration. The configuration file defines:
- Global settings (TUN interface names, ingress routers)
- Link definitions with network characteristics

## File Structure

### Global Section

```toml
[global]
tun_a = "tunA"           # Name for first TUN interface
tun_b = "tunB"           # Name for second TUN interface
ingress_a = "Rx0y0"      # Router where tunA packets enter fabric
ingress_b = "Rx5y5"      # Router where tunB packets enter fabric
```

### Link Sections

Links are defined with section names combining two router names with underscore:

```toml
[RouterA_RouterB]
mtu = 1500              # Maximum transmission unit (bytes)
delay_ms = 2.0          # Link delay in milliseconds
jitter_ms = 0.5         # Delay variation in milliseconds
loss_percent = 0.1      # Packet loss percentage (0.0 - 100.0)
per_packet_lb = false   # Enable per-packet load balancing
```

**Bidirectional Links:** Links are bidirectional by default. Defining both `[Rx0y0_Rx0y1]` and `[Rx0y1_Rx0y0]` will cause a validation error.

**Default Values:**
- `mtu`: 1500
- `delay_ms`: 0.0
- `jitter_ms`: 0.0
- `loss_percent`: 0.0
- `per_packet_lb`: false

## Router Naming

Routers must follow the pattern: `Rx[0-5]y[0-5]`

This represents a 6x6 grid where:
- `x` is the row (0-5)
- `y` is the column (0-5)

Examples: `Rx0y0`, `Rx2y3`, `Rx5y5`

## Example Configurations

See `examples/simple_topology.toml` for a minimal setup.
See `examples/full_fabric.toml` for a complete 6x6 mesh.

## Validation

The configuration is validated on load:
1. Router names must match Rx[0-5]y[0-5] pattern
2. No duplicate bidirectional links
3. Link names must contain exactly two routers
4. All referenced routers must be valid names

## Running the Simulator

```bash
netsimulator --config path/to/config.toml
```
```

**Step 2: Commit documentation**

```bash
git add docs/configuration.md
git commit -m "docs: add configuration file documentation"
```

---

## Plan 1 Completion Checklist

Before moving to Plan 2, verify:

- [ ] Cargo project builds without errors
- [ ] All tests pass: `cargo test`
- [ ] Configuration can be loaded from TOML string
- [ ] Configuration can be loaded from file
- [ ] Bidirectional link validation works
- [ ] Router name validation works
- [ ] CLI accepts --config argument
- [ ] Example configurations load successfully
- [ ] Documentation is complete

Run full test suite:
```bash
cargo test
cargo build --release
cargo run -- --config examples/simple_topology.toml
```

**Next:** Proceed to Plan 2 (Core Data Structures and Router Model)
