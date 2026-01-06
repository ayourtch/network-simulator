# Issue 052: IPv6 Real TUN Configuration Incomplete

## Summary
The helper `create_async_tun` in `src/tun/mod.rs` builds a TUN device using `tokio_tun::TunBuilder`. For IPv6 addresses the code parses the prefix but never sets the IPv6 address on the builder (the `address` method is only called for IPv4). Consequently, when a user configures an IPv6 real TUN (`real_tun_a` or `real_tun_b`) the resulting interface will lack the configured address, leading to communication failures.

## Expected Behavior
- When `cfg.interfaces.real_tun_{a,b}.address` is an IPv6 address, the builder should receive that address via `.address(ipv6_addr)` and the appropriate prefix length should be applied (e.g., via `.netmask(prefix)` or a system command). The resulting TUN interface must be correctly configured with the IPv6 address and netmask/prefix.

## Suggested Solution
1. Extend the IPv6 branch in `create_async_tun` to call `builder = builder.address(ipv6_addr);` (the `address` method accepts `std::net::Ipv6Addr`).
2. Use the parsed prefix (`netmask_str` parsed as `u8`) to set the netmask/prefix length, possibly via `.netmask(prefix)` if the library supports it, or otherwise document that a post‑creation `ip` command is required.
3. Add unit tests that instantiate the builder with an IPv6 address and verify no panic occurs.

## Acceptance Criteria
- IPv6 real TUN interfaces are created with the correct address and prefix.
- Configuration examples in `docs/example/dual_tun_host_setup.md` that use IPv6 work without manual post‑configuration.
- No runtime errors when IPv6 addresses are provided.
