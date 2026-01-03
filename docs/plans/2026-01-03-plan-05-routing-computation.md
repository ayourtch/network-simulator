# Plan 5: Routing Table Computation

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Compute simplified routing tables for each router with entries for tunA and tunB destinations using shortest path algorithms.

**Architecture:** Implement Dijkstra's algorithm to find shortest paths from each router to ingress routers. Each router gets two routing table entries: one for tunA destination and one for tunB destination. Support multiple equal-cost paths for load balancing.

**Tech Stack:** Rust standard library, binary heap for priority queue, HashMap for graph representation

---

## Task 1: Create Routing Module Structure

**Files:**
- Create: `src/routing/mod.rs`
- Create: `src/routing/table.rs`
- Create: `src/routing/dijkstra.rs`
- Modify: `src/lib.rs`
- Create: `tests/routing_test.rs`

**Step 1: Create routing module**

Create `src/routing/mod.rs`:
```rust
pub mod table;
pub mod dijkstra;

pub use table::{RoutingTable, RoutingEntry, NextHop};
pub use dijkstra::compute_shortest_paths;
```

Update `src/lib.rs`:
```rust
pub mod config;
pub mod topology;
pub mod tun;
pub mod packet;
pub mod forwarding;
pub mod routing;
```

**Step 2: Build to verify structure**

Run:
```bash
cargo build
```

Expected: Compilation errors (modules not created yet)

**Step 3: Create empty module files**

Create `src/routing/table.rs`:
```rust
// Routing table implementation
```

Create `src/routing/dijkstra.rs`:
```rust
// Dijkstra's shortest path algorithm
```

**Step 4: Build again**

Run:
```bash
cargo build
```

Expected: Success

**Step 5: Commit routing module structure**

```bash
git add src/routing/ src/lib.rs
git commit -m "feat: add routing module structure"
```

---

## Task 2: Implement Routing Table Data Structures

**Files:**
- Modify: `src/routing/table.rs`
- Create: `tests/routing_test.rs`

**Step 1: Write test for routing table**

Create `tests/routing_test.rs`:
```rust
use netsimulator::routing::{RoutingTable, RoutingEntry, NextHop};
use netsimulator::topology::Router;

#[test]
fn test_routing_table_creation() {
    let router = Router::new(0, 0);
    let table = RoutingTable::new(router.clone());

    assert_eq!(table.router().name(), "Rx0y0");
    assert!(table.get_next_hops_to_tun_a().is_empty());
    assert!(table.get_next_hops_to_tun_b().is_empty());
}

#[test]
fn test_add_routing_entry() {
    let router = Router::new(0, 0);
    let mut table = RoutingTable::new(router.clone());

    let next_hop = Router::new(0, 1);
    table.add_tun_a_route(next_hop.clone(), 10);

    let routes = table.get_next_hops_to_tun_a();
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].next_hop().name(), "Rx0y1");
    assert_eq!(routes[0].cost(), 10);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_routing_table_creation
```

Expected: FAIL with "struct `RoutingTable` not found"

**Step 3: Implement routing table structures**

Update `src/routing/table.rs`:
```rust
use crate::topology::Router;

/// Represents a next hop in the routing table
#[derive(Debug, Clone)]
pub struct NextHop {
    next_hop_router: Router,
    cost: u32,
}

impl NextHop {
    pub fn new(next_hop_router: Router, cost: u32) -> Self {
        NextHop {
            next_hop_router,
            cost,
        }
    }

    pub fn next_hop(&self) -> &Router {
        &self.next_hop_router
    }

    pub fn cost(&self) -> u32 {
        self.cost
    }
}

/// Routing entry (collection of next hops for a destination)
#[derive(Debug, Clone)]
pub struct RoutingEntry {
    next_hops: Vec<NextHop>,
}

impl RoutingEntry {
    pub fn new() -> Self {
        RoutingEntry {
            next_hops: Vec::new(),
        }
    }

    pub fn add_next_hop(&mut self, next_hop: NextHop) {
        self.next_hops.push(next_hop);
    }

    pub fn next_hops(&self) -> &[NextHop] {
        &self.next_hops
    }

    pub fn is_empty(&self) -> bool {
        self.next_hops.is_empty()
    }
}

impl Default for RoutingEntry {
    fn default() -> Self {
        Self::new()
    }
}

/// Routing table for a single router
/// Contains entries for tunA and tunB destinations
#[derive(Debug, Clone)]
pub struct RoutingTable {
    router: Router,
    to_tun_a: RoutingEntry,
    to_tun_b: RoutingEntry,
}

impl RoutingTable {
    pub fn new(router: Router) -> Self {
        RoutingTable {
            router,
            to_tun_a: RoutingEntry::new(),
            to_tun_b: RoutingEntry::new(),
        }
    }

    pub fn router(&self) -> &Router {
        &self.router
    }

    pub fn add_tun_a_route(&mut self, next_hop_router: Router, cost: u32) {
        self.to_tun_a.add_next_hop(NextHop::new(next_hop_router, cost));
    }

    pub fn add_tun_b_route(&mut self, next_hop_router: Router, cost: u32) {
        self.to_tun_b.add_next_hop(NextHop::new(next_hop_router, cost));
    }

    pub fn get_next_hops_to_tun_a(&self) -> &[NextHop] {
        self.to_tun_a.next_hops()
    }

    pub fn get_next_hops_to_tun_b(&self) -> &[NextHop] {
        self.to_tun_b.next_hops()
    }
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_routing_table_creation
cargo test test_add_routing_entry
```

Expected: PASS

**Step 5: Commit routing table structures**

```bash
git add src/routing/table.rs tests/routing_test.rs
git commit -m "feat: implement routing table data structures"
```

---

## Task 3: Implement Dijkstra's Algorithm

**Files:**
- Modify: `src/routing/dijkstra.rs`
- Modify: `tests/routing_test.rs`

**Step 1: Write test for Dijkstra**

Add to `tests/routing_test.rs`:
```rust
use netsimulator::routing::compute_shortest_paths;
use netsimulator::topology::{NetworkFabric, Router, Link};
use std::collections::HashMap;

#[test]
fn test_dijkstra_simple_path() {
    // Create simple 3-router path: R0 -> R1 -> R2
    let mut fabric = NetworkFabric::new();

    let r0 = Router::new(0, 0);
    let r1 = Router::new(0, 1);
    let r2 = Router::new(0, 2);

    fabric.add_router(r0.clone());
    fabric.add_router(r1.clone());
    fabric.add_router(r2.clone());

    fabric.add_link(Link::new(r0.clone(), r1.clone(), 1500, 1.0, 0.0, 0.0, false));
    fabric.add_link(Link::new(r1.clone(), r2.clone(), 1500, 1.0, 0.0, 0.0, false));

    // Compute shortest paths from r0 to r2
    let paths = compute_shortest_paths(&fabric, &r0, &r2);

    assert_eq!(paths.len(), 1);  // One path
    assert_eq!(paths[0].0.name(), "Rx0y1");  // Next hop is R1
    assert_eq!(paths[0].1, 2);  // Cost is 2
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_dijkstra_simple_path
```

Expected: FAIL with "function `compute_shortest_paths` not found"

**Step 3: Implement Dijkstra's algorithm**

Update `src/routing/dijkstra.rs`:
```rust
use crate::topology::{NetworkFabric, Router};
use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;

#[derive(Debug, Clone, Eq, PartialEq)]
struct State {
    cost: u32,
    coords: (u8, u8),
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap
        other.cost.cmp(&self.cost)
            .then_with(|| self.coords.cmp(&other.coords))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Compute shortest paths from source to destination
/// Returns vec of (next_hop_router, cost) for all equal-cost paths
pub fn compute_shortest_paths(
    fabric: &NetworkFabric,
    source: &Router,
    destination: &Router,
) -> Vec<(Router, u32)> {
    let source_coords = source.coords();
    let dest_coords = destination.coords();

    // If source == destination, no next hop needed
    if source_coords == dest_coords {
        return vec![];
    }

    // Distance map: coords -> cost
    let mut dist: HashMap<(u8, u8), u32> = HashMap::new();
    // Previous map: coords -> vec of (previous_coords, cost)
    let mut prev: HashMap<(u8, u8), Vec<(u8, u8)>> = HashMap::new();

    let mut heap = BinaryHeap::new();

    dist.insert(source_coords, 0);
    heap.push(State { cost: 0, coords: source_coords });

    while let Some(State { cost, coords }) = heap.pop() {
        // If we've reached destination, stop
        if coords == dest_coords {
            break;
        }

        // Skip if we've found a better path already
        if cost > *dist.get(&coords).unwrap_or(&u32::MAX) {
            continue;
        }

        // Get router at current coords
        let current_router = match fabric.get_router(coords.0, coords.1) {
            Some(r) => r,
            None => continue,
        };

        // Check all neighbors
        let neighbors = fabric.get_neighbors(current_router);
        for neighbor in neighbors {
            let neighbor_coords = neighbor.coords();

            // Link cost is 1 for simplicity (could use delay or other metric)
            let link_cost = 1;
            let new_cost = cost + link_cost;

            let current_dist = *dist.get(&neighbor_coords).unwrap_or(&u32::MAX);

            if new_cost < current_dist {
                // Found a better path
                dist.insert(neighbor_coords, new_cost);
                prev.insert(neighbor_coords, vec![coords]);
                heap.push(State { cost: new_cost, coords: neighbor_coords });
            } else if new_cost == current_dist {
                // Found an equal-cost path
                prev.entry(neighbor_coords)
                    .or_insert_with(Vec::new)
                    .push(coords);
            }
        }
    }

    // Backtrack from destination to find next hops from source
    find_next_hops(fabric, source_coords, dest_coords, &prev, &dist)
}

fn find_next_hops(
    fabric: &NetworkFabric,
    source: (u8, u8),
    destination: (u8, u8),
    prev: &HashMap<(u8, u8), Vec<(u8, u8)>>,
    dist: &HashMap<(u8, u8), u32>,
) -> Vec<(Router, u32)> {
    let mut result = Vec::new();

    // Get cost to destination
    let total_cost = match dist.get(&destination) {
        Some(&c) => c,
        None => return vec![],  // No path exists
    };

    // Find all next hops from source
    let source_router = match fabric.get_router(source.0, source.1) {
        Some(r) => r,
        None => return vec![],
    };

    let neighbors = fabric.get_neighbors(source_router);
    for neighbor in neighbors {
        let neighbor_coords = neighbor.coords();
        if let Some(&neighbor_cost) = dist.get(&neighbor_coords) {
            // Check if this neighbor is on a shortest path to destination
            if neighbor_cost == 1 && neighbor_cost + get_dist_to_dest(neighbor_coords, destination, dist) == total_cost {
                result.push((neighbor.clone(), total_cost));
            }
        }
    }

    result
}

fn get_dist_to_dest(from: (u8, u8), to: (u8, u8), dist: &HashMap<(u8, u8), u32>) -> u32 {
    if from == to {
        return 0;
    }
    *dist.get(&to).unwrap_or(&u32::MAX)
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_dijkstra_simple_path
```

Expected: PASS

**Step 5: Write test for multiple equal-cost paths**

Add to `tests/routing_test.rs`:
```rust
#[test]
fn test_dijkstra_multiple_paths() {
    // Create diamond topology:
    //     R1
    //    /  \
    //  R0    R3
    //    \  /
    //     R2
    let mut fabric = NetworkFabric::new();

    let r0 = Router::new(0, 0);
    let r1 = Router::new(0, 1);
    let r2 = Router::new(1, 0);
    let r3 = Router::new(1, 1);

    fabric.add_router(r0.clone());
    fabric.add_router(r1.clone());
    fabric.add_router(r2.clone());
    fabric.add_router(r3.clone());

    // Two equal-cost paths from R0 to R3
    fabric.add_link(Link::new(r0.clone(), r1.clone(), 1500, 0.0, 0.0, 0.0, false));
    fabric.add_link(Link::new(r1.clone(), r3.clone(), 1500, 0.0, 0.0, 0.0, false));
    fabric.add_link(Link::new(r0.clone(), r2.clone(), 1500, 0.0, 0.0, 0.0, false));
    fabric.add_link(Link::new(r2.clone(), r3.clone(), 1500, 0.0, 0.0, 0.0, false));

    let paths = compute_shortest_paths(&fabric, &r0, &r3);

    assert_eq!(paths.len(), 2);  // Two equal-cost paths
    assert_eq!(paths[0].1, 2);   // Both have cost 2
    assert_eq!(paths[1].1, 2);
}
```

**Step 6: Run test**

Run:
```bash
cargo test test_dijkstra_multiple_paths
```

Expected: PASS (may need to fix implementation)

**Step 7: Commit Dijkstra implementation**

```bash
git add src/routing/dijkstra.rs tests/routing_test.rs
git commit -m "feat: implement Dijkstra's shortest path algorithm"
```

---

## Task 4: Compute Routing Tables for All Routers

**Files:**
- Create: `src/routing/builder.rs`
- Modify: `src/routing/mod.rs`
- Modify: `tests/routing_test.rs`

**Step 1: Write test for routing table builder**

Add to `tests/routing_test.rs`:
```rust
use netsimulator::routing::build_all_routing_tables;
use netsimulator::config::NetworkConfig;

#[test]
fn test_build_routing_tables() {
    let toml_str = r#"
        [global]
        tun_a = "tunA"
        tun_b = "tunB"
        ingress_a = "Rx0y0"
        ingress_b = "Rx0y2"

        [Rx0y0_Rx0y1]
        [Rx0y1_Rx0y2]
    "#;

    let config = NetworkConfig::from_toml(toml_str).unwrap();
    let fabric = NetworkFabric::from_config(&config).unwrap();

    let tables = build_all_routing_tables(&fabric, &config);

    // Should have tables for all 3 routers
    assert_eq!(tables.len(), 3);

    // Router Rx0y1 should have routes to both tunA and tunB
    let r1_table = tables.iter()
        .find(|t| t.router().name() == "Rx0y1")
        .unwrap();

    assert!(!r1_table.get_next_hops_to_tun_a().is_empty());
    assert!(!r1_table.get_next_hops_to_tun_b().is_empty());
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_build_routing_tables
```

Expected: FAIL with "function `build_all_routing_tables` not found"

**Step 3: Implement routing table builder**

Create `src/routing/builder.rs`:
```rust
use crate::config::NetworkConfig;
use crate::topology::{NetworkFabric, Router};
use crate::routing::{RoutingTable, compute_shortest_paths};

/// Build routing tables for all routers in the fabric
pub fn build_all_routing_tables(
    fabric: &NetworkFabric,
    config: &NetworkConfig,
) -> Vec<RoutingTable> {
    let mut tables = Vec::new();

    // Get ingress routers
    let ingress_a = match fabric.get_router_by_name(&config.global.ingress_a) {
        Some(r) => r.clone(),
        None => return tables,
    };

    let ingress_b = match fabric.get_router_by_name(&config.global.ingress_b) {
        Some(r) => r.clone(),
        None => return tables,
    };

    // Build routing table for each router
    for x in 0..=5 {
        for y in 0..=5 {
            if let Some(router) = fabric.get_router(x, y) {
                let mut table = RoutingTable::new(router.clone());

                // Compute paths to ingress_a (tunA)
                let paths_to_a = compute_shortest_paths(fabric, router, &ingress_a);
                for (next_hop, cost) in paths_to_a {
                    table.add_tun_a_route(next_hop, cost);
                }

                // Compute paths to ingress_b (tunB)
                let paths_to_b = compute_shortest_paths(fabric, router, &ingress_b);
                for (next_hop, cost) in paths_to_b {
                    table.add_tun_b_route(next_hop, cost);
                }

                tables.push(table);
            }
        }
    }

    tables
}
```

Update `src/routing/mod.rs`:
```rust
pub mod table;
pub mod dijkstra;
pub mod builder;

pub use table::{RoutingTable, RoutingEntry, NextHop};
pub use dijkstra::compute_shortest_paths;
pub use builder::build_all_routing_tables;
```

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test test_build_routing_tables
```

Expected: PASS

**Step 5: Commit routing table builder**

```bash
git add src/routing/builder.rs src/routing/mod.rs tests/routing_test.rs
git commit -m "feat: implement routing table builder for all routers"
```

---

## Task 5: Integrate Routing into Forwarding Engine

**Files:**
- Modify: `src/forwarding/engine.rs`
- Modify: `tests/forwarding_test.rs`

**Step 1: Write test for routing integration**

Add to `tests/forwarding_test.rs`:
```rust
use netsimulator::routing::RoutingTable;

#[test]
fn test_forwarding_engine_with_routing() {
    let toml_str = r#"
        [global]
        tun_a = "tunA"
        tun_b = "tunB"
        ingress_a = "Rx0y0"
        ingress_b = "Rx0y2"

        [Rx0y0_Rx0y1]
        [Rx0y1_Rx0y2]
    "#;

    let config = NetworkConfig::from_toml(toml_str).unwrap();
    let fabric = NetworkFabric::from_config(&config).unwrap();
    let tables = netsimulator::routing::build_all_routing_tables(&fabric, &config);

    let engine = ForwardingEngine::with_routing_tables(fabric, config, tables);

    let r1 = Router::new(0, 1);
    let table = engine.get_routing_table(&r1);
    assert!(table.is_some());
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test test_forwarding_engine_with_routing
```

Expected: FAIL with "no method named `with_routing_tables`"

**Step 3: Update ForwardingEngine to store routing tables**

Update `src/forwarding/engine.rs`:
```rust
use crate::config::NetworkConfig;
use crate::topology::{NetworkFabric, Router};
use crate::tun::PacketMessage;
use crate::routing::RoutingTable;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetTun {
    TunA,
    TunB,
}

/// Main forwarding engine that processes packets through the fabric
pub struct ForwardingEngine {
    fabric: NetworkFabric,
    ingress_a_name: String,
    ingress_b_name: String,
    routing_tables: HashMap<(u8, u8), RoutingTable>,
}

impl ForwardingEngine {
    pub fn new(fabric: NetworkFabric, config: NetworkConfig) -> Self {
        ForwardingEngine {
            fabric,
            ingress_a_name: config.global.ingress_a,
            ingress_b_name: config.global.ingress_b,
            routing_tables: HashMap::new(),
        }
    }

    pub fn with_routing_tables(
        fabric: NetworkFabric,
        config: NetworkConfig,
        tables: Vec<RoutingTable>,
    ) -> Self {
        let mut routing_tables = HashMap::new();

        for table in tables {
            routing_tables.insert(table.router().coords(), table);
        }

        ForwardingEngine {
            fabric,
            ingress_a_name: config.global.ingress_a,
            ingress_b_name: config.global.ingress_b,
            routing_tables,
        }
    }

    pub fn get_routing_table(&self, router: &Router) -> Option<&RoutingTable> {
        self.routing_tables.get(&router.coords())
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
cargo test test_forwarding_engine_with_routing
```

Expected: PASS

**Step 5: Commit routing integration**

```bash
git add src/forwarding/engine.rs tests/forwarding_test.rs
git commit -m "feat: integrate routing tables into ForwardingEngine"
```

---

## Plan 5 Completion Checklist

Before moving to Plan 6, verify:

- [ ] All tests pass: `cargo test`
- [ ] RoutingTable structure works
- [ ] Dijkstra's algorithm computes shortest paths
- [ ] Multiple equal-cost paths are found
- [ ] Routing tables built for all routers
- [ ] ForwardingEngine stores routing tables
- [ ] Can look up routing table by router

Run full test suite:
```bash
cargo test
```

**Next:** Proceed to Plan 6 (Link Simulation - MTU, Delay, Jitter, Loss)
