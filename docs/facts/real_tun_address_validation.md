# Real TUN Address Validation Fact

The simulator now validates the `real_tun_a` and `real_tun_b` configuration fields as generic IP addresses, accepting both IPv4 and IPv6. IPv4 entries require a dotted‑quad netmask, while IPv6 entries use a prefix length (default /64 if omitted). Invalid addresses or out‑of‑range prefixes cause configuration validation to fail, preventing runtime errors.
