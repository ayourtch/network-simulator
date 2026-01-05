# Invalid Real TUN Address Issue

**Summary**: The simulator currently accepts any string for `real_tun_a.address`, `real_tun_a.netmask`, `real_tun_b.address`, and `real_tun_b.netmask`. Invalid IP strings will cause a panic at runtime when the TUN interface is configured.

**Location**: `src/config.rs` – `SimulatorConfig::validate()`.

**Current Behavior**: No validation; invalid values lead to runtime errors.

**Expected Behavior**: Validation should ensure that each address and netmask parses as a valid IPv4 address, returning a clear error message if not.

**Recommended Solution**: Add parsing using `std::net::Ipv4Addr::from_str` for each field. Return an error like `"Invalid IPv4 address for real_tun_a.address: '<value>'"`.

**Files to Modify**: `src/config.rs`.

**Effort Estimate**: Small (< 2 hours).

**Dependencies**: None.

**Related Plans**: Issue 036.
