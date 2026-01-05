# Direction Prefix Fact

The simulator now supports configurable IP prefixes (`tun_a_prefix` and `tun_b_prefix`) to determine packet direction in real TUN mode, replacing the fragile hard‑coded `10.` heuristic. This allows flexible routing based on IPv4 or IPv6 address ranges.
