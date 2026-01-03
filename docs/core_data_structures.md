# Core Data Structures Design

This document defines the primary Rust types that model the virtual network fabric, links, packets, and routing information. The design follows the module layout described in the master plan.

---

## 1. Top‑Level Types (module `topology`)

```rust
/// Unique identifier for a router in the fabric.
/// Expected format: "Rx{X}y{Y}" where X,Y ∈ [0,5] for the default 6×6 grid.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RouterId(pub String);

impl RouterId {
    /// Validate the identifier format.
    pub fn validate(&self) -> Result<(), String> {
        let re = regex::Regex::new(r"^Rx\d+y\d+$").unwrap();
        if re.is_match(&self.0) {
            Ok(())
        } else {
            Err(format!("Invalid router id '{}', expected RxXyY", self.0))
        }
    }
}

/// Representation of a virtual router.
#[derive(Debug, Clone)]
pub struct Router {
    /// Identifier (e.g., Rx0y0)
    pub id: RouterId,
    /// Computed routing table – two entries (to tunA and tunB)
    pub routing: RoutingTable,
    /// Optional per‑router state (e.g., statistics)
    pub stats: RouterStats,
}

/// Simple per‑router statistics.
#[derive(Debug, Default, Clone)]
pub struct RouterStats {
    pub packets_received: u64,
    pub packets_forwarded: u64,
    pub icmp_generated: u64,
}
```

## 2. Link Model (module `topology`)

```rust
/// Identifier for a bidirectional link between two routers.
/// The order of the two router ids does **not** matter.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinkId {
    pub a: RouterId,
    pub b: RouterId,
}

impl LinkId {
    /// Normalise the ordering (lexicographic) so that (A,B) == (B,A).
    pub fn new(r1: RouterId, r2: RouterId) -> Self {
        if r1.0 <= r2.0 {
            Self { a: r1, b: r2 }
        } else {
            Self { a: r2, b: r1 }
        }
    }
}

/// Configuration of a link – parsed from the TOML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkConfig {
    #[serde(default)]
    pub mtu: Option<u32>,          // Overrides global mtu when Some
    #[serde(default = "default_delay")]
    pub delay_ms: u32,
    #[serde(default = "default_jitter")]
    pub jitter_ms: u32,
    #[serde(default = "default_loss")]
    pub loss_percent: f32,        // 0.0 .. 100.0
    #[serde(default)]
    pub load_balance: bool,
}

fn default_delay() -> u32 { 0 }
fn default_jitter() -> u32 { 0 }
fn default_loss() -> f32 { 0.0 }

/// Runtime representation of a link, including a per‑packet counter for load‑balancing.
#[derive(Debug, Clone)]
pub struct Link {
    pub id: LinkId,
    pub cfg: LinkConfig,
    /// Counter incremented for each packet traversing the link – used when `load_balance` is true.
    pub counter: AtomicU64,
}
```

## 3. Fabric Graph (module `topology`)

The whole network is stored as an **undirected graph** using `petgraph` for easy path‑finding.

```rust
use petgraph::graph::{NodeIndex, UnGraph};
use std::collections::HashMap;

/// The complete virtual network fabric.
#[derive(Debug)]
pub struct Fabric {
    /// Underlying undirected graph – nodes are routers, edges are links.
    pub graph: UnGraph<Router, Link>,
    /// Mapping from `RouterId` to the node index in `graph`.
    pub router_index: HashMap<RouterId, NodeIndex>,
    /// Mapping from `LinkId` to the edge index (optional – petgraph can retrieve it via the nodes).
    pub link_index: HashMap<LinkId, petgraph::graph::EdgeIndex>,
}

impl Fabric {
    /// Create an empty fabric.
    pub fn new() -> Self {
        Self {
            graph: UnGraph::new_undirected(),
            router_index: HashMap::new(),
            link_index: HashMap::new(),
        }
    }

    /// Insert a router – panics if the id already exists.
    pub fn add_router(&mut self, router: Router) {
        let idx = self.graph.add_node(router.clone());
        self.router_index.insert(router.id.clone(), idx);
    }

    /// Insert a link between two existing routers.
    pub fn add_link(&mut self, a: &RouterId, b: &RouterId, cfg: LinkConfig) {
        let id = LinkId::new(a.clone(), b.clone());
        // Ensure both routers exist
        let a_idx = self.router_index.get(a).expect("router A missing");
        let b_idx = self.router_index.get(b).expect("router B missing");
        let link = Link { id: id.clone(), cfg, counter: AtomicU64::new(0) };
        let edge_idx = self.graph.add_edge(*a_idx, *b_idx, link);
        self.link_index.insert(id, edge_idx);
    }
}
```

## 4. Routing Table (module `routing`)

Each router only needs two entries: **to tunA** and **to tunB**. The entry stores the *next hop* router id and the *total cost* (used for debugging).

```rust
/// Destination identifier – only two possible values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Destination {
    TunA,
    TunB,
}

/// Single routing entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    pub next_hop: RouterId,
    pub total_cost: u32, // e.g., sum of link delays – optional
}

/// Routing table for a router – exactly two entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingTable {
    pub tun_a: RouteEntry,
    pub tun_b: RouteEntry,
}
```

### Routing Computation (module `routing`)

The computation is performed once after the topology is built (dynamic updates are out of scope for v1).

```rust
use petgraph::algo::dijkstra;

/// Compute routing tables for all routers in the fabric.
/// Returns a map `RouterId -> RoutingTable`.
pub fn compute_routing(fabric: &Fabric) -> HashMap<RouterId, RoutingTable> {
    let mut result = HashMap::new();

    // Determine the virtual "router" that represents each TUN interface.
    // In the configuration they are called "tunA" and "tunB" – we treat them as normal routers.
    let tun_a_id = RouterId("tunA".to_string());
    let tun_b_id = RouterId("tunB".to_string());

    // Ensure they exist in the graph (they will be added during topology creation).
    let tun_a_idx = fabric.router_index.get(&tun_a_id).expect("tunA missing");
    let tun_b_idx = fabric.router_index.get(&tun_b_id).expect("tunB missing");

    for (router_id, &node_idx) in &fabric.router_index {
        // Skip the special TUN nodes – they don't need routing entries.
        if router_id.0 == "tunA" || router_id.0 == "tunB" { continue; }

        // Dijkstra from current router to tunA and tunB.
        let costs = dijkstra(&fabric.graph, node_idx, None, |e| {
            // Edge weight = link delay (ms). If you want a different metric, replace here.
            let link = e.weight();
            link.cfg.delay_ms as u32
        });

        // Retrieve the next hop by walking back from destination to source.
        let next_hop_to_a = find_next_hop(&fabric.graph, node_idx, *tun_a_idx, &costs);
        let next_hop_to_b = find_next_hop(&fabric.graph, node_idx, *tun_b_idx, &costs);

        let table = RoutingTable {
            tun_a: RouteEntry { next_hop: next_hop_to_a, total_cost: *costs.get(&tun_a_idx).unwrap_or(&u32::MAX) },
            tun_b: RouteEntry { next_hop: next_hop_to_b, total_cost: *costs.get(&tun_b_idx).unwrap_or(&u32::MAX) },
        };
        result.insert(router_id.clone(), table);
    }
    result
}

/// Helper: walk back from destination to source using the `costs` map to find the immediate neighbor.
fn find_next_hop(
    graph: &UnGraph<Router, Link>,
    src: NodeIndex,
    dst: NodeIndex,
    costs: &HashMap<NodeIndex, u32>,
) -> RouterId {
    // Starting at dst, move to a neighbor with a strictly lower cost until we reach src.
    let mut current = dst;
    while current != src {
        let cur_cost = costs[&current];
        let mut found = false;
        for edge in graph.edges(current) {
            let neighbor = edge.target();
            if let Some(&nbr_cost) = costs.get(&neighbor) {
                if nbr_cost < cur_cost {
                    current = neighbor;
                    found = true;
                    break;
                }
            }
        }
        if !found {
            panic!("Failed to backtrack routing path from {:?} to {:?}", src, dst);
        }
    }
    // `current` is now the node directly adjacent to src.
    graph[current]
        .id
        .clone()
}
```

## 5. Packet Metadata (module `packet`)

Only a subset of packet fields are required for routing decisions.

```rust
#[derive(Debug, Clone)]
pub struct PacketMeta {
    pub src_ip: std::net::IpAddr,
    pub dst_ip: std::net::IpAddr,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8, // e.g., TCP=6, UDP=17, ICMP=1, ICMPv6=58
    pub ttl: u8,      // For IPv6 this is the Hop Limit field
    /// Virtual customer number – assigned when the packet first enters the simulator.
    pub customer_id: u32,
}
```

The raw packet bytes are handled by the `pnet`/`smoltcp` crates; they are parsed into `PacketMeta` for routing and ICMP generation.

---

## 6. Interaction with Other Modules

* **`tun`** – reads raw bytes from the TUN interfaces, parses them into `PacketMeta`, assigns a `customer_id`, and injects the packet into the fabric at the ingress router.
* **`simulation`** – when a packet traverses a `Link`, the link simulation layer consults `Link.cfg` (delay, jitter, loss) and updates the `Link.counter` if `load_balance` is true.
* **`icmp`** – builds ICMP error packets using the original `PacketMeta` and routes them back via the opposite routing table entry.
* **`forwarding`** – selects the egress link based on the router’s routing table and, if multiple equal‑cost paths exist, uses the 5‑tuple hash combined with the optional per‑link counter.

---

## 7. Serialization / Persistence

All structures that are part of the configuration (`LinkConfig`, `RouterId`, etc.) derive `Serialize`/`Deserialize` via `serde` so that the entire topology can be saved/loaded as JSON or TOML for debugging.

---

### Next Steps
1. Add unit tests for each struct’s validation logic (`RouterId::validate`, `LinkId::new`, etc.).
2. Implement the `topology::builder` that reads `config.toml` and creates a `Fabric`.
3. Wire the `routing::compute_routing` function into the startup flow.

These data‑structure definitions provide a clear contract for the rest of the codebase and enable incremental development of the simulator.
