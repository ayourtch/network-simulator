# Dual TUN Host Placement Fact

The simulator now supports a full dual‑TUN setup: a Linux host can be placed in a network namespace attached to `real_tun_a`, while the user interacts with `real_tun_b`. Packets flow bidirectionally through the simulated fabric, enabling realistic end‑to‑end testing of routing, load‑balancing, and packet‑loss handling.
