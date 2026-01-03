# Network Simulator - Master Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust-based network simulator with a 6x6 virtual router fabric that forwards traffic between two TUN interfaces with realistic network characteristics (delay, jitter, loss, MTU).

**Architecture:** Event-driven architecture using async Rust (tokio). TUN interfaces feed packets into designated ingress routers. Packets traverse the virtual fabric based on computed routing tables, respecting topology, link characteristics, and TTL. ICMP errors are generated as needed. Multi-path routing uses 5-tuple hashing for path selection.

**Tech Stack:**
- Rust (async/tokio for concurrency)
- tun/tap crate for TUN interface management
- toml crate for configuration parsing
- pnet or smoltcp for packet parsing/generation
- IPv4 and IPv6 support

---

## Implementation Plan Overview

This project is divided into 9 sequential implementation plans. Each plan builds on the previous ones and should be implemented in order.

### Plan 1: Project Setup and Configuration Parsing
**File:** `2026-01-03-plan-01-project-setup.md`

**Purpose:** Initialize Rust project structure, set up dependencies, and implement TOML configuration file parsing.

**Deliverables:**
- Cargo project with all dependencies
- Configuration data structures
- TOML parser that reads topology configuration
- Validation for bidirectional link detection and conflicts

**Prerequisites:** None

---

### Plan 2: Core Data Structures and Router Model
**File:** `2026-01-03-plan-02-core-data-structures.md`

**Purpose:** Implement the fundamental data structures representing the virtual network fabric.

**Deliverables:**
- Router representation (Rx0y0 to Rx5y5)
- Link representation with properties (MTU, delay, jitter, loss, per-packet load balancing flag)
- Network fabric topology graph
- Virtual customer number tracking

**Prerequisites:** Plan 1 completed

---

### Plan 3: TUN Interface Management
**File:** `2026-01-03-plan-03-tun-interfaces.md`

**Purpose:** Implement TUN interface creation, reading, and writing on Linux hosts.

**Deliverables:**
- TUN interface creation for tunA and tunB
- Async packet reading from TUN interfaces
- Async packet writing to TUN interfaces
- Error handling for TUN operations

**Prerequisites:** Plan 2 completed

---

### Plan 4: Packet Processing and Forwarding Engine
**File:** `2026-01-03-plan-04-packet-processing.md`

**Purpose:** Implement core packet processing logic including parsing, TTL handling, and basic forwarding.

**Deliverables:**
- IPv4 and IPv6 packet parsing
- TTL/hop limit decrement and validation
- Packet forwarding between routers
- Packet injection from TUN interfaces to designated ingress routers
- Packet delivery to TUN interfaces from egress routers

**Prerequisites:** Plan 3 completed

---

### Plan 5: Routing Table Computation
**File:** `2026-01-03-plan-05-routing-computation.md`

**Purpose:** Compute simplified routing tables for each router with entries for tunA and tunB destinations.

**Deliverables:**
- Shortest path algorithm (Dijkstra or similar)
- Per-router routing table with tunA and tunB entries
- Support for multiple equal-cost paths
- Routing table updates when topology changes (framework only, not dynamic in v1)

**Prerequisites:** Plan 4 completed

---

### Plan 6: Link Simulation (MTU, Delay, Jitter, Loss)
**File:** `2026-01-03-plan-06-link-simulation.md`

**Purpose:** Implement realistic network link characteristics.

**Deliverables:**
- MTU enforcement with fragmentation detection
- Link delay simulation
- Jitter simulation (random delay variation)
- Packet loss simulation (random and configurable)
- Async delay mechanisms using tokio timers

**Prerequisites:** Plan 5 completed

---

### Plan 7: ICMP Error Generation
**File:** `2026-01-03-plan-07-icmp-errors.md`

**Purpose:** Generate appropriate ICMP error messages for various failure scenarios.

**Deliverables:**
- TTL exceeded (Time Exceeded) ICMP messages
- Fragmentation needed (Destination Unreachable) ICMP messages
- ICMPv4 and ICMPv6 packet generation
- Proper source address selection for ICMP errors
- ICMP error routing back to source using reverse routing table

**Prerequisites:** Plan 6 completed

---

### Plan 8: Multi-path Routing and Load Balancing
**File:** `2026-01-03-plan-08-multipath-routing.md`

**Purpose:** Implement multi-path routing with 5-tuple hashing and optional per-packet load balancing.

**Deliverables:**
- 5-tuple extraction (src IP, dst IP, src port, dst port, protocol)
- Hash function combining 5-tuple and router hostname
- Path selection based on hash
- Per-packet load balancing mode using counter
- Consistent path selection for flows

**Prerequisites:** Plan 7 completed

---

### Plan 9: Integration and End-to-End Testing
**File:** `2026-01-03-plan-09-integration-testing.md`

**Purpose:** Comprehensive testing of the entire system with realistic scenarios.

**Deliverables:**
- End-to-end ping tests (IPv4 and IPv6)
- MTU verification tests
- Delay and jitter measurement tests
- Packet loss verification tests
- Multi-path routing verification
- ICMP error generation tests
- Performance benchmarking
- Example TOML configuration files

**Prerequisites:** Plan 8 completed

---

## Implementation Guidelines

**For each plan:**

1. **Follow TDD** - Write tests first, see them fail, implement minimal code to pass
2. **Commit frequently** - After each passing test or logical unit of work
3. **Keep it simple** - YAGNI (You Aren't Gonna Need It) - only build what's specified
4. **No over-engineering** - Don't add features not in the requirements
5. **Document as you go** - Add doc comments to public functions and structs
6. **Handle errors properly** - Use Result types, don't unwrap in production code
7. **Use async/await** - This is an I/O bound system, async is essential

**Testing strategy:**
- Unit tests for individual components
- Integration tests for component interactions
- End-to-end tests in Plan 9
- Use mock/fake implementations for TUN interfaces in early tests

**Code organization:**
```
src/
├── main.rs              # Entry point, CLI parsing
├── config/              # Configuration parsing (Plan 1)
├── topology/            # Router, Link, Fabric structures (Plan 2)
├── tun/                 # TUN interface management (Plan 3)
├── packet/              # Packet parsing and generation (Plan 4)
├── routing/             # Routing table computation (Plan 5)
├── simulation/          # Link simulation characteristics (Plan 6)
├── icmp/                # ICMP error generation (Plan 7)
├── forwarding/          # Multi-path forwarding logic (Plan 8)
└── lib.rs               # Library re-exports
```

---

## Getting Started

**To begin implementation:**

1. Start with Plan 1 (Project Setup and Configuration Parsing)
2. Follow each plan sequentially
3. Do not skip plans or implement out of order
4. Each plan includes detailed step-by-step instructions
5. Run tests frequently to catch issues early
6. Commit after each completed task

**Estimated effort:**
- Each plan: 4-8 hours (varies by complexity)
- Total project: 40-60 hours for experienced Rust developer
- Add 50-100% more time for someone learning Rust

**Success criteria:**
- All tests pass
- Can ping between tunA and tunB through the fabric
- Observable delay, jitter, and loss match configuration
- ICMP errors generated correctly for TTL exceeded and MTU issues
- Multi-path routing distributes traffic across multiple paths

---

## Notes for Implementation

**Platform compatibility:**
- TUN interfaces are Linux-specific in this version
- Test on Linux (Ubuntu 20.04+ recommended)
- macOS/Windows support would require significant changes

**Performance considerations:**
- Start with correctness, optimize later
- 6x6 = 36 routers should handle easily
- Async I/O prevents blocking on slow links
- Consider ring buffers for high packet rates (optimization, not v1)

**Future enhancements (NOT in v1):**
- Dynamic topology updates
- Multiple virtual customers with isolated topologies
- Statistics collection and reporting
- Web UI for visualization
- Larger fabric sizes
- Support for other protocols (MPLS, etc.)

**Remember:** Build what's specified, no more, no less. The plans are detailed enough for someone with basic Rust knowledge to implement successfully.
