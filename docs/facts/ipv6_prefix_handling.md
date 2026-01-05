# IPv6 Prefix Handling Fact

When configuring a real TUN interface with an IPv6 address, the simulator applies the prefix length (netmask) from the `netmask` field. The default IPv6 prefix is now `::/0`, matching all IPv6 addresses, unless a specific prefix is provided. The configuration is applied using a Linux `ip -6 addr add <addr>/<prefix> dev <name>` command after the interface is up.
