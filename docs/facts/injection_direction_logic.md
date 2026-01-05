# Injection Direction Logic Fact

When processing mock packets, the simulator determines the ingress router based on the source IP:
1. If `packet_inject_tun` is set, it forces the direction.
2. Otherwise, it checks `tun_a_prefix` and `tun_b_prefix` from the config.
3. If neither matches, it falls back to the legacy heuristic of detecting a `10.` prefix.
This logic ensures custom IPv4/IPv6 prefixes are respected for both single and multiple packet file modes.