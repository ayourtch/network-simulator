# IPv6 Prefix Default Fact

The simulator now defaults the IPv6 injection prefixes (`tun_a_ipv6_prefix` and `tun_b_ipv6_prefix`) to `::/0`, matching all IPv6 addresses. This ensures IPv6 traffic direction detection works out‑of‑the‑box without additional configuration.