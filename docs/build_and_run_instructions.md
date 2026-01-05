# Build & Run Instructions

This file explains how to compile, configure, and execute the network simulator on a Linux host.

---

## Prerequisites

| Item | Minimum Version | Why |
|------|-----------------|-----|
| **Rust** | `1.73` (or latest stable) | Cargo, async/await, `std::net` APIs |
| **Cargo** | bundled with Rust | Build system |
| **Linux kernel** | `5.4` (any modern distribution) | Required for TUN/TAP device creation |
| **Capabilities** | `CAP_NET_ADMIN` (or run as root) | Creating TUN interfaces requires elevated privileges |
| **Clang / libclang** | `>= 10` | Required by the `pnet` crate for packet parsing |

Install Rust via `rustup` if you don't have it:
```bash
curl https://sh.rustup.rs -sSf | sh -s -- -y
source $HOME/.cargo/env
rustup update stable
```

## Clone the Repository
```bash
git clone <repo-url> network-simulator
cd network-simulator
```

## Cargo Configuration
The project uses a few third‑party crates. The exact versions are pinned in `Cargo.toml`. If you need to add or update dependencies, run:
```bash
cargo update
```

## Building the Binary
```bash
cargo build --release
```
The executable will be located at `target/release/network-simulator`.

## Preparing the TUN Device
The simulator now supports **dual** TUN interfaces (`real_tun_a` and `real_tun_b`). Each can be created manually (requires root) or the binary will attempt to create them if they do not exist.

### Manual creation (recommended for debugging)
```bash
sudo ip tuntap add dev tunA mode tun
sudo ip link set tunA up
sudo ip tuntap add dev tunB mode tun
sudo ip link set tunB up
```
Assign IP addresses if you want to ping through them:
```bash
sudo ip addr add 10.0.0.1/24 dev tunA
sudo ip addr add 10.0.0.2/24 dev tunB
```

### Automatic creation
If the configuration file does **not** contain existing devices, the simulator will attempt to create them using the `tun` crate. In that case the process must run with the necessary capabilities (e.g., via `sudo` or setcap):
```bash
sudo setcap cap_net_admin=eip $(realpath $(which network-simulator))
```
Then you can run the binary as a normal user.

## Running the Simulator
```bash
# Basic run with the example configuration
cargo run --release -- -c config.toml
```
The command‑line interface accepts the following flags (future extensions may add more):
- `-c, --config <FILE>` – Path to the TOML configuration file (default: `config.toml`).
- `-v, --verbose` – Enable verbose logging (see Logging section).
- `--seed <NUM>` – Override the RNG seed defined in the configuration.

You should see log output indicating the creation of the fabric, routing table computation, and that the TUN interfaces are being listened on.

---

# Logging & Observability

The simulator uses the `tracing` crate with `tracing_subscriber` for structured, async‑compatible logs.

## Log Levels
| Level | Intended Use |
|-------|--------------|
| **ERROR** | Fatal failures (e.g., cannot create TUN, configuration invalid). The process will exit.
| **WARN** | Non‑fatal issues (e.g., a link configuration is missing optional fields, using defaults).
| **INFO** | High‑level lifecycle events (fabric built, routing tables computed, start/stop of TUN readers).
| **DEBUG** | Per‑packet events (packet received, dropped due to loss, delayed, forwarded). Verbose mode enables this.
| **TRACE** | Extremely fine‑grained data (e.g., hash values for multipath, exact timer expirations).

## Configuring the Subscriber
At startup we initialise a subscriber that respects the `RUST_LOG` environment variable:
```rust
use tracing_subscriber::{EnvFilter, fmt};

fn init_logging() {
    let env = EnvFilter::from_default_env()
        .add_directive("network_simulator=info".parse().unwrap());
    fmt::Subscriber::builder()
        .with_env_filter(env)
        .init();
}
```
Run with `RUST_LOG=debug cargo run …` to see debug logs.

## Metrics (Future Extension)
The current version does not expose Prometheus metrics, but the logging infrastructure makes it easy to add counters (e.g., packets processed per router) later.

---

# Test Outline (documented in `docs/testing_strategy.md`)

Below is a concise outline of the test plan; the actual test files will live under `tests/` and `src/` unit‑test modules.

1. **Configuration Parsing**
   - Valid TOML loads correctly, all routers/links present.
   - Invalid router IDs cause an error.
   - Duplicate link definitions are rejected.
   - Bidirectional consistency validation.

2. **Core Data Structures**
   - `RouterId::validate` accepts correct patterns, rejects others.
   - `LinkId::new` normalises order (A_B == B_A).
   - `Fabric::add_router`/`add_link` correctly updates internal maps.
   - `Link.counter` increments atomically.

3. **Routing Computation**
   - Simple topology (two routers + tunA/tunB) yields expected next‑hop entries.
   - Multi‑path topology with equal‑cost paths produces deterministic next hop (e.g., lexicographic order).
   - Verify that `total_cost` matches sum of link delays.

4. **Link Simulation**
   - When `loss_percent` = 100, packet is dropped.
   - When `delay_ms` is set, packet is delayed by at least that duration (use `tokio::time::Instant::now`).
   - Jitter adds random variance within bounds.
   - `load_balance` flag toggles counter usage for hash.

5. **Packet Processing**
   - Parsing raw IPv4/IPv6 packets yields correct `PacketMeta` fields.
   - TTL/Hop‑Limit decrement works and generates ICMP Time‑Exceeded when reaching zero.
   - MTU enforcement: packet larger than link MTU triggers ICMP Destination Unreachable (Fragmentation Needed).

6. **ICMP Generation**
   - TTL‑exceeded generates correct ICMP Type/Code and includes original IP header + first 8 bytes of payload.
   - Fragmentation Needed includes the MTU field.

7. **Multi‑path Routing**
   - For a topology with two equal‑cost paths, the hash of a 5‑tuple consistently selects the same path across packets of the same flow.
   - When `load_balance = true` on a link, the per‑packet counter changes the hash outcome, spreading traffic across both paths.

8. **Integration / End‑to‑End** (Plan 9)
   - Spin up the binary in a PTY, send ICMP ping through `tunA` → `tunB` and verify round‑trip.
   - Measure observed delay vs configured link delay (allow small tolerance).
   - Verify packet loss statistics match configured loss percentages over many packets.
   - Test IPv6 ping similarly.
   - Validate that the simulator can be cleanly stopped (SIGINT) without leaking resources.

All tests should be runnable with `cargo test`. Integration tests that need root/TUN access are placed under `tests/integration/` and are marked with `#[ignore]` by default; they can be run manually with `cargo test -- --ignored` after the necessary privileges are granted.

---

# Summary
You now have:
- **Configuration schema** (`docs/configuration_schema.md`)
- **Core data‑structure definitions** (`docs/core_data_structures.md`)
- **Build & run guide + logging plan** (this file)
- **Testing strategy outline** (`docs/testing_strategy.md`)

With these artifacts, the implementation can proceed in the order defined by the master plan, and each step can be verified against concrete tests and documentation.
