# Plan 2: Core Data Structures and Router Model

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the fundamental data structures representing the virtual network fabric including routers, links, and the overall topology graph.

**Architecture:** Use a graph-based representation where routers are nodes and links are edges. Each router maintains references to its neighbors and has a unique coordinate-based identifier. The fabric maintains the global view of all routers and links.

**Tech Stack:** Rust standard library collections (HashMap, Vec), Arc/Mutex for shared state

---

## Task 1: Create Router Data Structure

**Files:**
- Create: `src/topology/mod.rs`
- Create: `src/topology/router.rs`
- Create: `tests/topology_test.rs`
- Modify: `src/lib.rs`

**Step 1: Write test for router creation**

Create `tests/topology_test.rs`:
```rust
use netsimulator::topology::Router;

#[test]
fn test_router_creation() {
    let router = Router::new(0, 0);
    assert_eq!(router.x(), 0);
    assert_eq!(router.y(), 0);
    assert_eq!(router.name(), "Rx0y0");
}

#[test]
fn test_router_name_formatting() {
    let r1 = Router::new(3, 4);
    assert_eq!(r1.name(), "Rx3y4");

    let r2 = Router::new(5, 5);
    assert_eq!(r2.name(), "Rx5y5");
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_router_creation
```

Expected: FAIL with "module `topology` not found"

**Step 3: Create topology module structure**

Create `src/topology/mod.rs`:
```rust
pub mod router;
pub mod link;
pub mod fabric;

pub use router::Router;
pub use link::Link;
pub use fabric::NetworkFabric;
```

Update `src/lib.rs`:
```rust
pub mod config;
pub mod topology;
```

**Step 4: Implement Router structure**

Create `src/topology/router.rs`:
```rust
use std::sync::Arc;

/// Represents a virtual router in the 6x6 fabric
#[derive(Debug, Clone)]
pub struct Router {
    x: u8,
    y: u8,
    name: String,
}

impl Router {
    /// Create a new router at coordinates (x, y)
    pub fn new(x: u8, y: u8) -> Self {
        assert!(x <= 5, "Router x coordinate must be 0-5");
        assert!(y <= 5, "Router y coordinate must be 0-5");

        let name = format!("Rx{}y{}", x, y);

        Router { x, y, name }
    }

    /// Create a router from a name string (e.g., "Rx0y0")
    pub fn from_name(name: &str) -> Option<Self> {
        let re = regex::Regex::new(r"^Rx([0-5])y([0-5])$").unwrap();
        let caps = re.captures(name)?;

        let x = caps.get(1)?.as_str().parse::<u8>().ok()?;
        let y = caps.get(2)?.as_str().parse::<u8>().ok()?;

        Some(Router::new(x, y))
    }

    pub fn x(&self) -> u8 {
        self.x
    }

    pub fn y(&self) -> u8 {
        self.y
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn coords(&self) -> (u8, u8) {
        (self.x, self.y)
    }
}

impl PartialEq for Router {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Router {}

impl std::hash::Hash for Router {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}
```

**Step 5: Run test to verify it passes**

Run:
```bash
cargo test test_router_creation
cargo test test_router_name_formatting
```

Expected: PASS

**Step 6: Write test for router from_name parsing**

Add to `tests/topology_test.rs`:
```rust
#[test]
fn test_router_from_name() {
    let router = Router::from_name("Rx2y3");
    assert!(router.is_some());
    let router = router.unwrap();
    assert_eq!(router.x(), 2);
    assert_eq!(router.y(), 3);
}

#[test]
fn test_router_from_invalid_name() {
    assert!(Router::from_name("InvalidName").is_none());
    assert!(Router::from_name("Rx6y0").is_none());  // x out of range
    assert!(Router::from_name("Rx0y7").is_none());  // y out of range
}
```

**Step 7: Run test**

Run:
```bash
cargo test test_router_from_name
```

Expected: PASS

**Step 8: Commit router implementation**

```bash
git add src/topology/ src/lib.rs tests/topology_test.rs
git commit -m "feat: add Router data structure with coordinate-based naming"
```

---

## Task 2: Create Link Data Structure

**Files:**
- Create: `src/topology/link.rs`
- Modify: `tests/topology_test.rs`

**Step 1: Write test for link creation**

Add to `tests/topology_test.rs`:
```rust
use netsimulator::topology::{Router, Link};

#[test]
fn test_link_creation() {
    let r1 = Router::new(0, 0);
    let r2 = Router::new(0, 1);

    let link = Link::new(r1.clone(), r2.clone(), 1500, 0.0, 0.0, 0.0, false);

    assert_eq!(link.mtu(), 1500);
    assert_eq!(link.delay_ms(), 0.0);
    assert_eq!(link.jitter_ms(), 0.0);
    assert_eq!(link.loss_percent(), 0.0);
    assert_eq!(link.per_packet_lb(), false);
}

#[test]
fn test_link_with_characteristics() {
    let r1 = Router::new(1, 1);
    let r2 = Router::new(1, 2);

    let link = Link::new(r1, r2, 1400, 5.0, 1.0, 0.5, true);

    assert_eq!(link.mtu(), 1400);
    assert_eq!(link.delay_ms(), 5.0);
    assert_eq!(link.jitter_ms(), 1.0);
    assert_eq!(link.loss_percent(), 0.5);
    assert_eq!(link.per_packet_lb(), true);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_link_creation
```

Expected: FAIL with "struct `Link` not found"

**Step 3: Implement Link structure**

Create `src/topology/link.rs`:
```rust
use crate::topology::Router;

/// Represents a bidirectional link between two routers
#[derive(Debug, Clone)]
pub struct Link {
    router_a: Router,
    router_b: Router,
    mtu: u32,
    delay_ms: f64,
    jitter_ms: f64,
    loss_percent: f64,
    per_packet_lb: bool,
}

impl Link {
    pub fn new(
        router_a: Router,
        router_b: Router,
        mtu: u32,
        delay_ms: f64,
        jitter_ms: f64,
        loss_percent: f64,
        per_packet_lb: bool,
    ) -> Self {
        Link {
            router_a,
            router_b,
            mtu,
            delay_ms,
            jitter_ms,
            loss_percent,
            per_packet_lb,
        }
    }

    pub fn router_a(&self) -> &Router {
        &self.router_a
    }

    pub fn router_b(&self) -> &Router {
        &self.router_b
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

    pub fn per_packet_lb(&self) -> bool {
        self.per_packet_lb
    }

    /// Get the other end of the link given one router
    pub fn other_end(&self, router: &Router) -> Option<&Router> {
        if router == &self.router_a {
            Some(&self.router_b)
        } else if router == &self.router_b {
            Some(&self.router_a)
        } else {
            None
        }
    }

    /// Check if this link connects two specific routers
    pub fn connects(&self, r1: &Router, r2: &Router) -> bool {
        (r1 == &self.router_a && r2 == &self.router_b)
            || (r1 == &self.router_b && r2 == &self.router_a)
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_link_creation
cargo test test_link_with_characteristics
```

Expected: PASS

**Step 5: Write test for link helper methods**

Add to `tests/topology_test.rs`:
```rust
#[test]
fn test_link_other_end() {
    let r1 = Router::new(0, 0);
    let r2 = Router::new(0, 1);
    let link = Link::new(r1.clone(), r2.clone(), 1500, 0.0, 0.0, 0.0, false);

    let other = link.other_end(&r1);
    assert!(other.is_some());
    assert_eq!(other.unwrap(), &r2);

    let other = link.other_end(&r2);
    assert!(other.is_some());
    assert_eq!(other.unwrap(), &r1);

    let r3 = Router::new(1, 1);
    assert!(link.other_end(&r3).is_none());
}

#[test]
fn test_link_connects() {
    let r1 = Router::new(0, 0);
    let r2 = Router::new(0, 1);
    let r3 = Router::new(1, 1);

    let link = Link::new(r1.clone(), r2.clone(), 1500, 0.0, 0.0, 0.0, false);

    assert!(link.connects(&r1, &r2));
    assert!(link.connects(&r2, &r1));  // bidirectional
    assert!(!link.connects(&r1, &r3));
    assert!(!link.connects(&r2, &r3));
}
```

**Step 6: Run test**

Run:
```bash
cargo test test_link_other_end
cargo test test_link_connects
```

Expected: PASS

**Step 7: Commit link implementation**

```bash
git add src/topology/link.rs tests/topology_test.rs
git commit -m "feat: add Link data structure with network characteristics"
```

---

## Task 3: Create Network Fabric Structure

**Files:**
- Create: `src/topology/fabric.rs`
- Modify: `tests/topology_test.rs`

**Step 1: Write test for fabric creation**

Add to `tests/topology_test.rs`:
```rust
use netsimulator::topology::NetworkFabric;

#[test]
fn test_fabric_creation() {
    let fabric = NetworkFabric::new();
    assert_eq!(fabric.router_count(), 0);
    assert_eq!(fabric.link_count(), 0);
}

#[test]
fn test_fabric_add_router() {
    let mut fabric = NetworkFabric::new();
    let router = Router::new(0, 0);

    fabric.add_router(router.clone());
    assert_eq!(fabric.router_count(), 1);
    assert!(fabric.has_router(&router));
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_fabric_creation
```

Expected: FAIL with "struct `NetworkFabric` not found"

**Step 3: Implement NetworkFabric structure**

Create `src/topology/fabric.rs`:
```rust
use crate::topology::{Router, Link};
use std::collections::{HashMap, HashSet};

/// Represents the entire network fabric with all routers and links
#[derive(Debug)]
pub struct NetworkFabric {
    routers: HashMap<(u8, u8), Router>,
    links: Vec<Link>,
    // Adjacency list: router coords -> list of (neighbor coords, link index)
    adjacency: HashMap<(u8, u8), Vec<((u8, u8), usize)>>,
}

impl NetworkFabric {
    pub fn new() -> Self {
        NetworkFabric {
            routers: HashMap::new(),
            links: Vec::new(),
            adjacency: HashMap::new(),
        }
    }

    pub fn add_router(&mut self, router: Router) {
        let coords = router.coords();
        self.routers.insert(coords, router);
        self.adjacency.entry(coords).or_insert_with(Vec::new);
    }

    pub fn add_link(&mut self, link: Link) {
        let idx = self.links.len();
        let a_coords = link.router_a().coords();
        let b_coords = link.router_b().coords();

        // Add to adjacency lists for both routers
        self.adjacency
            .entry(a_coords)
            .or_insert_with(Vec::new)
            .push((b_coords, idx));

        self.adjacency
            .entry(b_coords)
            .or_insert_with(Vec::new)
            .push((a_coords, idx));

        self.links.push(link);
    }

    pub fn router_count(&self) -> usize {
        self.routers.len()
    }

    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    pub fn has_router(&self, router: &Router) -> bool {
        self.routers.contains_key(&router.coords())
    }

    pub fn get_router(&self, x: u8, y: u8) -> Option<&Router> {
        self.routers.get(&(x, y))
    }

    pub fn get_router_by_name(&self, name: &str) -> Option<&Router> {
        let router = Router::from_name(name)?;
        self.get_router(router.x(), router.y())
    }

    pub fn get_links_for_router(&self, router: &Router) -> Vec<&Link> {
        let coords = router.coords();
        self.adjacency
            .get(&coords)
            .map(|neighbors| {
                neighbors
                    .iter()
                    .map(|(_, link_idx)| &self.links[*link_idx])
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_neighbors(&self, router: &Router) -> Vec<&Router> {
        let coords = router.coords();
        self.adjacency
            .get(&coords)
            .map(|neighbors| {
                neighbors
                    .iter()
                    .filter_map(|(neighbor_coords, _)| self.routers.get(neighbor_coords))
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for NetworkFabric {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_fabric_creation
cargo test test_fabric_add_router
```

Expected: PASS

**Step 5: Write test for fabric with links**

Add to `tests/topology_test.rs`:
```rust
#[test]
fn test_fabric_add_link() {
    let mut fabric = NetworkFabric::new();

    let r1 = Router::new(0, 0);
    let r2 = Router::new(0, 1);

    fabric.add_router(r1.clone());
    fabric.add_router(r2.clone());

    let link = Link::new(r1.clone(), r2.clone(), 1500, 0.0, 0.0, 0.0, false);
    fabric.add_link(link);

    assert_eq!(fabric.link_count(), 1);
    assert_eq!(fabric.get_links_for_router(&r1).len(), 1);
    assert_eq!(fabric.get_links_for_router(&r2).len(), 1);
}

#[test]
fn test_fabric_get_neighbors() {
    let mut fabric = NetworkFabric::new();

    let r1 = Router::new(0, 0);
    let r2 = Router::new(0, 1);
    let r3 = Router::new(1, 0);

    fabric.add_router(r1.clone());
    fabric.add_router(r2.clone());
    fabric.add_router(r3.clone());

    fabric.add_link(Link::new(r1.clone(), r2.clone(), 1500, 0.0, 0.0, 0.0, false));
    fabric.add_link(Link::new(r1.clone(), r3.clone(), 1500, 0.0, 0.0, 0.0, false));

    let neighbors = fabric.get_neighbors(&r1);
    assert_eq!(neighbors.len(), 2);

    let neighbors = fabric.get_neighbors(&r2);
    assert_eq!(neighbors.len(), 1);
}
```

**Step 6: Run test**

Run:
```bash
cargo test test_fabric_add_link
cargo test test_fabric_get_neighbors
```

Expected: PASS

**Step 7: Commit fabric implementation**

```bash
git add src/topology/fabric.rs tests/topology_test.rs
git commit -m "feat: add NetworkFabric structure with adjacency graph"
```

---

## Task 4: Build Fabric from Configuration

**Files:**
- Modify: `src/topology/fabric.rs`
- Modify: `tests/topology_test.rs`

**Step 1: Write test for building fabric from config**

Add to `tests/topology_test.rs`:
```rust
use netsimulator::config::NetworkConfig;

#[test]
fn test_build_fabric_from_config() {
    let toml_str = r#"
        [global]
        tun_a = "tunA"
        tun_b = "tunB"
        ingress_a = "Rx0y0"
        ingress_b = "Rx2y2"

        [Rx0y0_Rx0y1]
        mtu = 1500

        [Rx0y1_Rx1y1]
        mtu = 1500

        [Rx1y1_Rx2y2]
        mtu = 1400
        delay_ms = 5.0
    "#;

    let config = NetworkConfig::from_toml(toml_str).unwrap();
    let fabric = NetworkFabric::from_config(&config);

    assert!(fabric.is_ok());
    let fabric = fabric.unwrap();

    // Should have 4 routers mentioned in links
    assert_eq!(fabric.router_count(), 4);
    assert_eq!(fabric.link_count(), 3);

    // Verify specific routers exist
    assert!(fabric.get_router_by_name("Rx0y0").is_some());
    assert!(fabric.get_router_by_name("Rx2y2").is_some());
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_build_fabric_from_config
```

Expected: FAIL with "no method named `from_config`"

**Step 3: Implement from_config method**

Add to `src/topology/fabric.rs`:
```rust
use crate::config::NetworkConfig;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FabricError {
    #[error("Invalid router name in configuration: {0}")]
    InvalidRouterName(String),
}

impl NetworkFabric {
    pub fn from_config(config: &NetworkConfig) -> Result<Self, FabricError> {
        let mut fabric = NetworkFabric::new();
        let mut router_set = HashSet::new();

        // Extract all unique routers from link definitions
        for (link_name, link_config) in &config.links {
            let parts: Vec<&str> = link_name.split('_').collect();
            if parts.len() != 2 {
                return Err(FabricError::InvalidRouterName(link_name.clone()));
            }

            // Add both routers
            for router_name in parts {
                router_set.insert(router_name.to_string());
            }
        }

        // Add ingress routers (they might not be in links)
        router_set.insert(config.global.ingress_a.clone());
        router_set.insert(config.global.ingress_b.clone());

        // Create router objects
        for router_name in &router_set {
            let router = Router::from_name(router_name)
                .ok_or_else(|| FabricError::InvalidRouterName(router_name.clone()))?;
            fabric.add_router(router);
        }

        // Create links
        for (link_name, link_config) in &config.links {
            let parts: Vec<&str> = link_name.split('_').collect();
            let router_a = Router::from_name(parts[0])
                .ok_or_else(|| FabricError::InvalidRouterName(parts[0].to_string()))?;
            let router_b = Router::from_name(parts[1])
                .ok_or_else(|| FabricError::InvalidRouterName(parts[1].to_string()))?;

            let link = Link::new(
                router_a,
                router_b,
                link_config.mtu,
                link_config.delay_ms,
                link_config.jitter_ms,
                link_config.loss_percent,
                link_config.per_packet_lb,
            );

            fabric.add_link(link);
        }

        Ok(fabric)
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_build_fabric_from_config
```

Expected: PASS

**Step 5: Commit fabric building from config**

```bash
git add src/topology/fabric.rs tests/topology_test.rs
git commit -m "feat: build NetworkFabric from configuration"
```

---

## Task 5: Add Virtual Customer Number Support

**Files:**
- Create: `src/topology/customer.rs`
- Modify: `src/topology/mod.rs`
- Modify: `tests/topology_test.rs`

**Step 1: Write test for customer number**

Add to `tests/topology_test.rs`:
```rust
use netsimulator::topology::VirtualCustomer;

#[test]
fn test_virtual_customer_creation() {
    let customer = VirtualCustomer::new(1);
    assert_eq!(customer.id(), 1);
}

#[test]
fn test_virtual_customer_equality() {
    let c1 = VirtualCustomer::new(1);
    let c2 = VirtualCustomer::new(1);
    let c3 = VirtualCustomer::new(2);

    assert_eq!(c1, c2);
    assert_ne!(c1, c3);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_virtual_customer_creation
```

Expected: FAIL with "struct `VirtualCustomer` not found"

**Step 3: Implement VirtualCustomer structure**

Create `src/topology/customer.rs`:
```rust
/// Represents a virtual customer number for topology isolation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VirtualCustomer {
    id: u32,
}

impl VirtualCustomer {
    pub fn new(id: u32) -> Self {
        VirtualCustomer { id }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

impl Default for VirtualCustomer {
    fn default() -> Self {
        VirtualCustomer::new(0)
    }
}
```

Update `src/topology/mod.rs`:
```rust
pub mod router;
pub mod link;
pub mod fabric;
pub mod customer;

pub use router::Router;
pub use link::Link;
pub use fabric::NetworkFabric;
pub use customer::VirtualCustomer;
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_virtual_customer_creation
cargo test test_virtual_customer_equality
```

Expected: PASS

**Step 5: Commit virtual customer support**

```bash
git add src/topology/customer.rs src/topology/mod.rs tests/topology_test.rs
git commit -m "feat: add VirtualCustomer for topology isolation"
```

---

## Task 6: Add Fabric Visualization/Debug Output

**Files:**
- Modify: `src/topology/fabric.rs`
- Modify: `src/main.rs`

**Step 1: Implement Display trait for fabric**

Add to `src/topology/fabric.rs`:
```rust
use std::fmt;

impl NetworkFabric {
    pub fn print_summary(&self) {
        println!("Network Fabric Summary:");
        println!("  Routers: {}", self.router_count());
        println!("  Links: {}", self.link_count());
        println!();

        println!("Router Details:");
        let mut routers: Vec<_> = self.routers.values().collect();
        routers.sort_by_key(|r| r.coords());

        for router in routers {
            let neighbors = self.get_neighbors(router);
            print!("  {} -> neighbors: ", router.name());
            for (i, neighbor) in neighbors.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{}", neighbor.name());
            }
            println!();
        }
        println!();

        println!("Link Details:");
        for (i, link) in self.links.iter().enumerate() {
            println!(
                "  Link {}: {} <-> {} (MTU: {}, Delay: {}ms, Jitter: {}ms, Loss: {}%)",
                i,
                link.router_a().name(),
                link.router_b().name(),
                link.mtu(),
                link.delay_ms(),
                link.jitter_ms(),
                link.loss_percent()
            );
        }
    }
}
```

**Step 2: Update main.rs to print fabric**

Update `src/main.rs`:
```rust
use clap::Parser;
use netsimulator::config::NetworkConfig;
use netsimulator::topology::NetworkFabric;
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
    println!();

    println!("Building network fabric...");
    let fabric = NetworkFabric::from_config(&config)?;
    println!();

    fabric.print_summary();

    Ok(())
}
```

**Step 3: Test manually**

Run:
```bash
cargo build
./target/debug/netsimulator --config examples/simple_topology.toml
```

Expected: Detailed output showing routers and links

**Step 4: Commit visualization**

```bash
git add src/topology/fabric.rs src/main.rs
git commit -m "feat: add fabric visualization and debug output"
```

---

## Plan 2 Completion Checklist

Before moving to Plan 3, verify:

- [ ] All tests pass: `cargo test`
- [ ] Router structure works with coordinate naming
- [ ] Link structure stores network characteristics
- [ ] NetworkFabric maintains adjacency graph
- [ ] Fabric can be built from configuration
- [ ] Virtual customer numbers are supported
- [ ] Fabric visualization works
- [ ] Running with example config shows fabric details

Run full test suite:
```bash
cargo test
cargo run -- --config examples/simple_topology.toml
```

**Next:** Proceed to Plan 3 (TUN Interface Management)
