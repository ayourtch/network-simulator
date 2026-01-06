# IPv6 Real TUN Configuration Fact

The simulator now configures IPv6 addresses on real TUN interfaces by invoking the `ip -6 addr add <addr>/<prefix> dev <name>` command after creating the TUN device. This ensures IPv6 traffic can flow through the dual‑TUN setup, addressing the previous limitation where IPv6 TUN interfaces were left unconfigured.
