# IPv6 Netmask Default Fact

When an IPv6 address is configured for a real TUN interface without an explicit netmask/prefix, the simulator defaults to a `/64` prefix length. This matches the common convention for IPv6 subnets and ensures the interface is usable out‑of‑the‑box.
