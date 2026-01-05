# IPv6 Real TUN Support Fact

The simulator now accepts IPv6 addresses for `real_tun_a` and `real_tun_b`. IPv6 netmask is provided as a prefix length (default /64 if omitted). During TUN setup the `ip -6 addr add` command is executed and its success is verified; any failure is reported as an error. This enables reliable end‑to‑end IPv6 traffic through the dual‑TUN setup.