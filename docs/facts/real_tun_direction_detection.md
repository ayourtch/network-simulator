# Real TUN Direction Detection Fact

In real TUN mode the simulator determines packet direction based on the originating TUN device: packets read from `real_tun_a` are forwarded to `real_tun_b` and vice versa. This removes reliance on IP‑prefix heuristics and works for both IPv4 and IPv6 traffic.