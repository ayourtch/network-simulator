# IPv6 Prefix Handling Fact

When configuring a real TUN interface with an IPv6 address, the simulator applies the prefix length (netmask) from the `netmask` field (default `/64` if omitted) using a Linux `ip -6 addr add <addr>/<prefix> dev <name>` command after the interface is up.
