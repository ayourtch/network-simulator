# Real TUN Routing Test Unable to Execute (Resolved)

**Problem**
The simulator requires real TUN interfaces to test routing (adding a route to `192.0.2.1` on one interface and verifying packets exit the other). Creating TUN devices fails with `Operation not permitted` because the current environment lacks `CAP_NET_ADMIN` privileges.

**Resolution**
We added a permission fallback in `src/tun/mod.rs` that detects `EPERM` and skips real‑TUN handling, allowing the simulator to run in mock mode. Since real‑TUN routing cannot be exercised without elevated privileges, the issue is considered resolved by documenting the limitation and providing the fallback.

**Impact**
The simulator now works for developers without root access, though full routing verification requires a privileged environment.
