# Issue 060: Add IPv6 example to Dual TUN Host Setup Guide

## Summary
The current `docs/facts/dual_tun_host_setup.md` guide only shows IPv4 address configuration. With recent support for IPv6 real TUN interfaces, the documentation should include an example configuration and steps for IPv6.

## Suggested Solution
1. Extend the guide with an IPv6‑specific section.
2. Show a sample TOML config using IPv6 addresses (e.g., `address = "2001:db8::1"`, `netmask = "64"`).
3. Update the setup commands to configure IPv6 on the TUN devices using `ip -6 addr add`.
4. Ensure the guide notes that the IPv6 netmask defaults to `/64` when omitted.

Providing this example will help users leverage the new IPv6 capabilities without confusion.
