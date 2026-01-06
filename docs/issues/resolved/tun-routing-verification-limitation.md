# Real TUN Routing Verification Limitation (Resolved)

**Problem**
The final step of the workflow requires creating a real TUN interface, adding a route to `192.0.2.1`, and verifying that packets injected on one TUN exit the other. In this CI environment the user lacks `CAP_NET_ADMIN`, so `ip tuntap add` and `ip route add` commands fail with `Operation not permitted`.

**Resolution**
The simulator now includes a permission‑fallback in `src/tun/mod.rs` that detects `EPERM`, logs a warning, and runs in mock mode. This allows the simulator to function without root privileges, though full routing verification must be performed in a privileged environment (e.g., a VM or container with CAP_NET_ADMIN). The limitation is documented in `docs/facts/real_tun_routing_test_limitation.md`.

**Impact**
Developers can run the simulator without needing elevated rights, while being aware that end‑to‑end TUN routing tests require appropriate permissions.
