# Issue 042: Provide End‑to‑End Example for Dual‑TUN Host Placement

**Summary**
The simulator now supports a dual‑TUN architecture that allows a Linux host to run inside a network namespace attached to `real_tun_a` while the user interacts via `real_tun_b`. The repository lacked a clear step‑by‑step example.

**Resolution**
Added `docs/example/dual_tun_host_setup.md` containing the full tutorial, updated `README.md` and `docs/build_and_run_instructions.md` to reference it.

**Effort**
Small (≈1 hour).