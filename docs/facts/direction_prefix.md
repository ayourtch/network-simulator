# Direction Prefix Fact

The simulator now supports configurable IP prefixes for both IPv4 and IPv6. `tun_a_prefix`/`tun_b_prefix` handle IPv4, while `tun_a_ipv6_prefix`/`tun_b_ipv6_prefix` handle IPv6. This replaces the fragile hard‑coded `10.` heuristic and enables flexible routing based on address ranges.

