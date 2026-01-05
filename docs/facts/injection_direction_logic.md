# Injection Direction Logic Fact

When processing mock packets, the simulator determines the ingress router based on the source IP:
1. If `packet_inject_tun` (or per‑file `packet_inject_tuns`) is set, it forces the direction.
2. Otherwise, it checks `tun_a_prefix`/`tun_b_prefix` (IPv4) **and** `tun_a_ipv6_prefix`/`tun_b_ipv6_prefix` (IPv6) from the config.
3. If none match, it falls back to the legacy heuristic of detecting a `10.` prefix.
This logic ensures custom IPv4 and IPv6 prefixes are respected for both single and multiple packet file modes.