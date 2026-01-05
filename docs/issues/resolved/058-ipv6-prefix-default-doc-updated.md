# Issue 058: IPv6 Prefix Default Documentation Updated

## Summary
The default IPv6 prefix for injection direction detection has been changed to `::/0` (match all IPv6 addresses). The configuration defaults and documentation in `docs/example/dual_tun_host_setup.md` have been updated accordingly.

## Resolution
- Updated `default_ipv6_prefix_a` and `default_ipv6_prefix_b` in `src/config.rs` to return `"::/0"`.
- Added IPv6 prefix fields documentation to the dual‑TUN host setup guide.
- The change ensures IPv6 traffic is correctly routed without requiring explicit prefix configuration.
