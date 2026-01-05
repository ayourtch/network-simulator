# IPv6 Injection Prefix Handling Issue

**Description**

The current packet injection direction logic in `src/tun/mod.rs` only checks IPv4 prefixes (`tun_a_prefix` and `tun_b_prefix`) and falls back to the legacy `10.` heuristic. When using IPv6 addresses in mock packet files, the direction detection may be incorrect because IPv6 prefixes are not considered.

**Expected Behavior**

- The injection logic should also respect configurable IPv6 prefixes (e.g., `2001:db8::/32`).
- If a packet's source IPv6 address matches the configured IPv6 prefix for `tun_a` (or `tun_b`), the packet should be injected accordingly.
- If no IPv6 prefix matches, the fallback heuristic can be used (or a clear error).

**Suggested Implementation**

1. Extend `TunIngressConfig` to include `tun_a_ipv6_prefix` and `tun_b_ipv6_prefix` (default empty).
2. Update the injection logic to parse the source IP as `IpAddr` and check `starts_with` for both IPv4 and IPv6 prefixes (using string representation or proper prefix matching).
3. Add unit tests covering IPv6 injection scenarios.
4. Document the new configuration fields in `docs/facts` and README.

**Low‑skill Implementation Steps**

- Add the two new fields with defaults in `src/config.rs`.
- Update the TOML example files.
- Modify the direction detection block in `src/tun/mod.rs` to check IPv6 prefixes after IPv4 checks.
- Add a new fact file `docs/facts/ipv6_injection_prefix.md`.
- Run `cargo test` to ensure everything passes.

**Priority**: Medium – improves usability for IPv6 testing environments.
