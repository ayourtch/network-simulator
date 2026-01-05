# IPv6 Netmask Prefix Not Applied Issue

**Summary**: While IPv6 addresses are now supported for `real_tun_a` and `real_tun_b`, the IPv6 netmask (prefix length) provided in configuration is currently ignored during TUN device creation. The code defaults to a placeholder and does not set the prefix, potentially leading to mismatched network behavior.

**Location**: `src/tun/mod.rs::create_async_tun` – IPv6 branch uses `.address(std::net::IpAddr::V6(v6)).up();` without applying netmask/prefix.

**Current Behavior**: IPv6 netmask field is parsed and validated (allowing empty or a prefix 0‑128), but during device setup the prefix is not configured. The TUN interface may have a default /64 or undefined prefix, which could cause routing issues.

**Expected Behavior**: The IPv6 prefix from configuration should be applied to the TUN interface, e.g., using `.netmask` method for IPv6 if supported, or by invoking appropriate system commands to set the prefix.

**Fix Summary**: Updated `create_async_tun` to apply IPv6 prefix using a Linux `ip -6 addr add <addr>/<prefix> dev <name>` command after bringing the interface up. Default prefix is /64 when omitted. Errors are propagated.

**Files Modified**:
- `src/tun/mod.rs`

**Effort Estimate**: Small (≈1‑2 hours).

**Dependencies**: None.
