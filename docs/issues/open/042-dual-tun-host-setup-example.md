# Issue 042: Provide End‑to‑End Example for Dual‑TUN Host Placement

**Summary**
The simulator now supports a dual‑TUN architecture that allows a Linux host to run inside a network namespace attached to `real_tun_a` while the user interacts via `real_tun_b`. However, the repository lacks a clear, step‑by‑step example (script or documentation) demonstrating how to set up the namespace, configure the interfaces, start the simulator, and verify traffic flow. This makes it difficult for users to exercise the primary use‑case of the project.

**Location**
- Documentation files: `README.md`, `docs/build_and_run_instructions.md`
- No existing example script in the repository.

**Current Behavior**
- Users must manually configure TUN devices and namespace.
- No guidance on IP addressing, routing, or how to generate traffic.
- Potential for misconfiguration, leading to failed experiments.

**Expected Behavior**
Provide a concise, reproducible example that:
1. Creates a network namespace `ns_host`.
2. Moves `real_tun_a` into the namespace and assigns an IP address.
3. Configures `real_tun_b` on the host with a complementary address.
4. Starts the simulator with appropriate config (dual‑TUN enabled).
5. Demonstrates connectivity (e.g., `ping` from the namespace to an address reachable via the simulated fabric).
6. Includes clean‑up steps.

**Recommended Solution**
- Add a new markdown file `docs/example/dual_tun_host_setup.md` containing the full script and explanations.
- Update `README.md` to link to this example.
- Ensure the example works on a typical Linux system with `ip` and `sudo` privileges.
- Optionally add a small integration test that runs the script in a container (if CI permits).

**Files to Modify / Add**
- `docs/example/dual_tun_host_setup.md` (new)
- `README.md` (add link under "Examples" section)
- `docs/build_and_run_instructions.md` (brief mention of the new example)

**Effort Estimate**
Small (≈1 hour for scripting and documentation, plus testing).

**Dependencies**
- None beyond existing simulator binary.
- Relies on the dual‑TUN implementation already present.

**Related Issues**
- Issue 028 (dual‑TUN architecture implementation) – already resolved.
- Issue 034 (documentation mismatch) – partially addressed, but still lacking concrete usage example.
