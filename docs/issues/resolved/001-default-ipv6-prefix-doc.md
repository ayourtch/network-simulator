# Issue 001: Default IPv6 Prefix Documentation

## Summary
The configuration fields `tun_a_ipv6_prefix` and `tun_b_ipv6_prefix` default to `::/0`, matching all IPv6 addresses. This ensures IPv6 traffic direction detection works without explicit configuration.

## Resolution
- Updated `src/config.rs` defaults to `"::/0"`.
- Documented IPv6 prefix fields in `docs/example/dual_tun_host_setup.md`.
- Added fact about IPv6 prefix handling.

The issue is now resolved.