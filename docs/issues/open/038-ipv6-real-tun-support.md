# IPv6 Real TUN Support Issue

**Summary**: The simulator's real‑TUN implementation only handles IPv4 addresses (`Ipv4Addr`). There is no support for configuring IPv6 addresses or netmasks for `real_tun_a` and `real_tun_b`.

**Location**: `src/config.rs` – `RealTunConfig` fields `address` and `netmask` are strings parsed as `Ipv4Addr` in `src/tun/mod.rs::create_async_tun`.

**Current Behavior**: Providing an IPv6 address in the configuration will either be rejected by the new IPv4 validation or cause the TUN device to be configured with the fallback IPv4 defaults, making IPv6 traffic impossible.

**Expected Behavior**: Allow the user to specify either an IPv4 or IPv6 address (and appropriate prefix length). The `create_async_tun` function should detect the address family and configure the `tun` crate accordingly (e.g., using `.address(IpAddr::V6(...))` and appropriate netmask/prefix).

**Recommended Solution**:
1. Extend `RealTunConfig` to include an optional `address_family: String` or detect family by parsing the string with `IpAddr::from_str`.
2. In `create_async_tun`, attempt to parse the address as `IpAddr`. If it is `V4`, configure as before. If it is `V6`, use `cfg.address(IpAddr::V6(addr))` and set a suitable netmask/prefix (e.g., `/64`).
3. Update validation in `SimulatorConfig::validate` to accept both IPv4 and IPv6 formats.
4. Update documentation and fact files to note IPv6 support when implemented.

**Files to Modify**:
- `src/config.rs` (validation logic)
- `src/tun/mod.rs` (creation logic)
- Documentation (README or config examples)

**Effort Estimate**: Medium (2‑4 hours) – requires handling of both address families and testing with an IPv6‑enabled TUN.

**Dependencies**: None.

**Related Plans**: Issue 038.
