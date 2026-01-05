# Prefix Matching Improvement Issue

**Description**

The current injection‑direction logic (both for single and multiple mock‑packet files) determines whether a source IP belongs to a configured prefix by using `String::starts_with`. This works for simple prefixes like `10.` but fails for more complex CIDR specifications (e.g., `2001:db8::/32` or `192.168.0.0/16`). It also does not validate that the prefix string is well‑formed.

**Expected Behavior**

- The configuration should allow CIDR‑style prefixes for both IPv4 and IPv6 (e.g., `192.168.0.0/16`, `2001:db8::/32`).
- Injection direction should be decided by checking whether the packet's source IP is *contained* within the configured network, using proper address‑mask matching rather than a simple string prefix.
- Invalid prefix strings should be caught during configuration validation with a clear error message.

**Suggested Implementation**

1. Add a helper function `fn ip_in_prefix(ip: &IpAddr, prefix: &str) -> Result<bool, String>` that parses the prefix as `IpNet` (using the `ipnet` crate) and checks containment.
2. Update `SimulatorConfig::validate` to verify that all IPv4/IPv6 prefix fields are either empty or parseable as CIDR networks.
3. Replace all `starts_with` checks in `src/tun/mod.rs` with calls to `ip_in_prefix`.
4. Add unit tests covering IPv4 and IPv6 CIDR matching, as well as error handling for malformed prefixes.
5. Document the new behavior in `docs/facts/injection_direction_logic.md` and add a short fact `docs/facts/prefix_matching.md`.

**Priority**: Medium – improves robustness and aligns the simulator with real‑world networking scenarios.
