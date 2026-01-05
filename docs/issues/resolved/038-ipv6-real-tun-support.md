# IPv6 Real TUN Support Issue

**Summary**: The simulator's real‑TUN implementation only handles IPv4 addresses (`Ipv4Addr`). There is no support for configuring IPv6 addresses or netmasks for `real_tun_a` and `real_tun_b`.

**Location**: `src/config.rs` – `RealTunConfig` fields `address` and `netmask` are strings parsed as `Ipv4Addr` in `src/tun/mod.rs::create_async_tun`.

**Current Behavior**: Providing an IPv6 address in the configuration will either be rejected by the new IPv4 validation or cause the TUN device to be configured with the fallback IPv4 defaults, making IPv6 traffic impossible.

**Expected Behavior**: Allow the user to specify either an IPv4 or IPv6 address (and appropriate prefix length). The `create_async_tun` function should detect the address family and configure the `tun` crate accordingly (e.g., using `.address(IpAddr::V6(...))` and appropriate netmask/prefix).

**Implemented Solution**:
- Added generic IP address parsing and validation supporting IPv4 and IPv6 in `SimulatorConfig::validate`.
- Updated `create_async_tun` to configure IPv6 addresses using `.address(std::net::IpAddr::V6(v6))`.
- IPv6 netmask is treated as prefix length; defaults to /64 if omitted.
- Updated documentation and facts.

**Files Modified**:
- `src/config.rs`
- `src/tun/mod.rs`
- Docs facts updated.

**Status**: Resolved.
