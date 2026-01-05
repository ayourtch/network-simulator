# Issue 042: Dual TUN Host Setup Example Missing

**Summary**
The documentation mentions that the simulator can run with two real TUN interfaces to allow a Linux host in a network namespace to communicate through the virtual network, but there is no step‑by‑step guide. Users need clear instructions to create the namespaces, configure the TUN devices, and start the simulator.

**Location**
- Documentation folder `docs/facts/dual_tun.md` provides high‑level description but no concrete example.

**Current Behavior**
- No tutorial or script is provided, making it hard for new users to set up the dual‑TUN scenario.

**Expected Behavior**
- Add a markdown guide (e.g., `docs/facts/dual_tun_host_setup.md`) with commands to:
  1. Create two network namespaces (`ns1` and `ns2`).
  2. Create TUN interfaces (`tunA` and `tunB`) and assign them to the namespaces.
  3. Configure IP addresses and routes.
  4. Start the simulator with appropriate config pointing to `real_tun_a` and `real_tun_b`.
  5. Verify connectivity (ping between host in `ns1` and external network).

**Suggested Solution (Low‑skill)**
1. Write the guide in markdown, using `ip netns add`, `ip link add`, `ip link set netns`, `ip address add`, `ip link set up`.
2. Include example `SimulatorConfig` snippet showing the `real_tun_a`/`real_tun_b` sections.
3. Add a small shell script (`run_dual_tun.sh`) that automates the setup for testing.
4. Reference the new guide from `docs/facts/dual_tun.md`.

**Effort Estimate**
Small (1–2 hours).