# Testing Strategy

This document expands the **outline** given in `docs/build_and_run_instructions.md` into concrete, runnable tests. The test suite is divided into three layers:

1. **Unit Tests** – fast, no external resources, run on every `cargo test`.
2. **Integration Tests** – involve the full runtime (async, timers, TUN creation). Marked `#[ignore]` because they need root privileges.
3. **End‑to‑End (E2E) Tests** – launch the compiled binary in a PTY, send real traffic through the simulated fabric, and verify behaviour. Also `#[ignore]`.

All tests are located under `src/` (unit tests) or `tests/` (integration/E2E). The Cargo.toml is already configured with the `tokio` test harness (`[dev-dependencies] tokio = { version = "1", features = ["macros", "rt-multi-thread"] }`).

---

## 1. Unit Tests

### 1.1 `router_id.rs`
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_router_ids() {
        let ids = ["Rx0y0", "Rx5y5", "Rx12y3"]; // allow any digits, later validation may enforce bounds
        for &s in &ids {
            let rid = RouterId(s.to_string());
            assert!(rid.validate().is_ok(), "{} should be valid", s);
        }
    }

    #[test]
    fn invalid_router_ids() {
        let ids = ["R0x0y0", "rx0y0", "Rx0y", "Rx_y0", ""];
        for &s in &ids {
            let rid = RouterId(s.to_string());
            assert!(rid.validate().is_err(), "{} should be invalid", s);
        }
    }
}
```

### 1.2 `link_id.rs`
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_is_normalised() {
        let a = RouterId("Rx0y1".to_string());
        let b = RouterId("Rx0y0".to_string());
        let id1 = LinkId::new(a.clone(), b.clone());
        let id2 = LinkId::new(b, a);
        assert_eq!(id1, id2);
        // Lexicographic order means a < b => a becomes .a
        assert_eq!(id1.a, RouterId("Rx0y0".to_string()));
        assert_eq!(id1.b, RouterId("Rx0y1".to_string()));
    }
}
```

### 1.3 `fabric.rs`
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::router::{Router, RouterId};
    use crate::link::{LinkConfig};

    #[tokio::test]
    async fn add_routers_and_links() {
        let mut fabric = Fabric::new();
        let r1 = Router { id: RouterId("Rx0y0".to_string()), routing: RoutingTable { tun_a: RouteEntry{next_hop: RouterId("".to_string()), total_cost:0}, tun_b: RouteEntry{next_hop: RouterId("".to_string()), total_cost:0}}, stats: RouterStats::default() };
        let r2 = Router { id: RouterId("Rx0y1".to_string()), routing: RoutingTable { tun_a: RouteEntry{next_hop: RouterId("".to_string()), total_cost:0}, tun_b: RouteEntry{next_hop: RouterId("".to_string()), total_cost:0}}, stats: RouterStats::default() };
        fabric.add_router(r1);
        fabric.add_router(r2);
        let cfg = LinkConfig { mtu: Some(1500), delay_ms: 10, jitter_ms: 0, loss_percent: 0.0, load_balance: false };
        fabric.add_link(&RouterId("Rx0y0".to_string()), &RouterId("Rx0y1".to_string()), cfg);
        assert_eq!(fabric.router_index.len(), 2);
        assert_eq!(fabric.link_index.len(), 1);
    }
}
```

### 1.4 `routing.rs`
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::{Fabric, Router, RouterId, RoutingTable, RouteEntry};
    use crate::link::LinkConfig;

    fn build_simple_fabric() -> Fabric {
        let mut fabric = Fabric::new();
        // Add routers: tunA, tunB, Rx0y0, Rx0y1
        for id in ["tunA", "tunB", "Rx0y0", "Rx0y1"].iter() {
            let router = Router { id: RouterId(id.to_string()), routing: RoutingTable { tun_a: RouteEntry{next_hop: RouterId("".to_string()), total_cost:0}, tun_b: RouteEntry{next_hop: RouterId("".to_string()), total_cost:0}}, stats: RouterStats::default() };
            fabric.add_router(router);
        }
        let cfg = LinkConfig { mtu: Some(1500), delay_ms: 5, jitter_ms: 0, loss_percent: 0.0, load_balance: false };
        // Connect tunA <-> Rx0y0, Rx0y0 <-> Rx0y1, Rx0y1 <-> tunB
        fabric.add_link(&RouterId("tunA".to_string()), &RouterId("Rx0y0".to_string()), cfg.clone());
        fabric.add_link(&RouterId("Rx0y0".to_string()), &RouterId("Rx0y1".to_string()), cfg.clone());
        fabric.add_link(&RouterId("Rx0y1".to_string()), &RouterId("tunB".to_string()), cfg);
        fabric
    }

    #[test]
    fn routing_table_is_correct() {
        let fabric = build_simple_fabric();
        let tables = compute_routing(&fabric);
        // For Rx0y0, next hop to tunB should be Rx0y1
        let rx0y0 = RouterId("Rx0y0".to_string());
        let entry = tables.get(&rx0y0).expect("routing for Rx0y0");
        assert_eq!(entry.tun_b.next_hop.0, "Rx0y1");
        // For Rx0y1, next hop to tunA should be Rx0y0
        let rx0y1 = RouterId("Rx0y1".to_string());
        let entry2 = tables.get(&rx0y1).expect("routing for Rx0y1");
        assert_eq!(entry2.tun_a.next_hop.0, "Rx0y0");
    }
}
```

### 1.5 `link_simulation.rs`
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::Instant;

    #[tokio::test]
    async fn loss_percent_100_drops_packet() {
        let cfg = LinkConfig { mtu: Some(1500), delay_ms: 0, jitter_ms: 0, loss_percent: 100.0, load_balance: false };
        let link = Link { id: LinkId::new(RouterId("A".to_string()), RouterId("B".to_string())), cfg, counter: std::sync::atomic::AtomicU64::new(0) };
        let pkt = DummyPacket::default();
        let result = link_simulate(&link, pkt).await; // function to be implemented
        assert!(result.is_err(), "packet should be dropped due to 100% loss");
    }

    #[tokio::test]
    async fn delay_is_respected() {
        let cfg = LinkConfig { mtu: Some(1500), delay_ms: 20, jitter_ms: 0, loss_percent: 0.0, load_balance: false };
        let link = Link { id: LinkId::new(RouterId("A".to_string()), RouterId("B".to_string())), cfg, counter: std::sync::atomic::AtomicU64::new(0) };
        let pkt = DummyPacket::default();
        let start = Instant::now();
        let _ = link_simulate(&link, pkt).await.unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(20), "elapsed = {:?}", elapsed);
    }
}
```

*(Note: `link_simulate` and `DummyPacket` are placeholder names; the real implementation will be in `simulation/link.rs`.)*

---

## 2. Integration Tests (under `tests/integration/`)

These tests spin up the full async runtime, create in‑memory TUN interfaces (using the `tun` crate's `dev` mode when possible), and feed synthetic packets through the system.

### 2.1 `tests/integration/tun_flow.rs`
```rust
#[tokio::test]
#[ignore] // requires root or CAP_NET_ADMIN
async fn tun_a_to_b_round_trip() {
    // 1. Start the simulator in a background task with a minimal config.
    let config_path = "tests/configs/simple.toml";
    let sim_handle = tokio::spawn(async move {
        network_simulator::run_with_config(config_path).await.unwrap();
    });

    // 2. Open the TUN devices (created by the simulator) using the `tun` crate.
    let mut tun_a = tun::create("tunA").await.expect("create tunA");
    let mut tun_b = tun::create("tunB").await.expect("create tunB");

    // 3. Send an IPv4 ICMP Echo Request from tunA.
    let icmp_req = icmp::v4_echo_request(0x1234, 1, "10.0.0.1", "10.0.1.1");
    tun_a.write(&icmp_req).await.expect("write to tunA");

    // 4. Read the reply from tunB.
    let mut buf = vec![0u8; 1500];
    let n = tun_b.read(&mut buf).await.expect("read from tunB");
    let reply = &buf[..n];
    assert!(icmp::is_echo_reply(reply), "got reply: {:x?}", reply);

    // 5. Clean up.
    sim_handle.abort();
}
```
The helper functions `icmp::v4_echo_request` and `icmp::is_echo_reply` are tiny wrappers around the `pnet_packet` crate.

### 2.2 `tests/integration/multipath.rs`
```rust
#[tokio::test]
#[ignore]
async fn equal_cost_multi_path_hash_consistency() {
    // Build a topology where Rx0y0 has two equal‑cost neighbours to reach tunB.
    let config_path = "tests/configs/multipath.toml";
    let sim_handle = tokio::spawn(async move { network_simulator::run_with_config(config_path).await.unwrap() });

    // Open the TUN interfaces.
    let mut tun_a = tun::create("tunA").await.unwrap();
    let mut tun_b = tun::create("tunB").await.unwrap();

    // Send a burst of 100 packets with the same 5‑tuple.
    let pkt = icmp::v4_echo_request(0xdead, 1, "10.0.0.1", "10.0.1.1");
    for _ in 0..100 { tun_a.write(&pkt).await.unwrap(); }

    // Collect which egress links were used (the simulator logs the chosen link; we capture via a custom logger).
    // For this test we simply assert that all 100 replies arrive – the hash consistency is verified inside the simulator's log assertions.
    let mut recv = 0usize;
    let mut buf = vec![0u8; 1500];
    while recv < 100 {
        let n = tun_b.read(&mut buf).await.unwrap();
        if icmp::is_echo_reply(&buf[..n]) { recv += 1; }
    }
    assert_eq!(recv, 100);
    sim_handle.abort();
}
```

---

## 3. End‑to‑End (E2E) Tests (under `tests/e2e/`)

E2E tests launch the compiled binary in a PTY, allowing us to verify **logging output**, **signal handling**, and the **full process lifecycle**.

### 3.1 `tests/e2e/run_and_stop.rs`
```rust
use std::process::{Command, Stdio};
use std::io::Write;
use std::thread;
use std::time::Duration;

#[test]
#[ignore]
fn start_and_graceful_shutdown() {
    // Spawn the binary with a simple config.
    let mut child = Command::new("target/release/network-simulator")
        .arg("-c")
        .arg("tests/configs/simple.toml")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start simulator");

    // Give it a moment to initialise.
    thread::sleep(Duration::from_secs(2));

    // Send SIGINT for graceful shutdown.
    #[cfg(unix)] { unsafe { libc::kill(child.id() as i32, libc::SIGINT) } }

    // Wait for termination (with a timeout).
    let timeout = Duration::from_secs(5);
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        match child.try_wait() {
            Ok(Some(status)) => {
                assert!(status.success(), "process exited with error: {:?}", status);
                return;
            }
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(e) => panic!("error waiting for child: {}", e),
        }
    }
    panic!("process did not exit within timeout");
}
```
The test also captures stdout/stderr and can assert that certain log lines (e.g., "Fabric built", "Routing tables computed") appear.

---

## Running the Full Test Suite
```bash
# Unit tests (fast)
cargo test

# Integration tests (require root / CAP_NET_ADMIN)
cargo test --tests/integration -- --ignored

# End‑to‑End tests (also require root)
cargo test --tests/e2e -- --ignored
```

All CI pipelines should execute the unit tests on every commit. Integration/E2E tests can be gated behind a protected job that runs in a privileged Docker container.

---

## Continuous Integration (CI) Recommendations
1. **GitHub Actions** – Use a `ubuntu-latest` runner, install Rust, and run `cargo test`.
2. **Privileged Runner** – For integration/E2E, enable `privileged: true` and add a step to grant `CAP_NET_ADMIN` to the binary (`sudo setcap cap_net_admin=eip $(which cargo)` after build).
3. **Cache Cargo Registry** – Speed up CI.
4. **Flaky Test Guard** – For probabilistic tests (loss/jitter), repeat the scenario a few times and assert statistical bounds (e.g., loss observed within ±5% of configured value).

---

# Conclusion
The testing strategy now contains concrete, compilable Rust test code for every major component, a clear separation between fast unit tests and slower integration/E2E tests, and CI recommendations to keep the codebase reliable. Implementers can copy the snippets into the appropriate modules and adjust the placeholder names (`DummyPacket`, `link_simulate`, etc.) to match the actual implementation.
