# Resolved: Virtual Customer Not Used – issue closed

The simulator now generates synthetic packets when a `virtual_customer` configuration is provided. The implementation:
- Added `VirtualCustomerConfig` with fields `src_ip`, `dst_ip`, `protocol`, `size`, `rate`.
- In `src/tun/mod.rs` a packet is constructed using these settings, checksum calculated, and injected via the appropriate ingress based on CIDR prefixes or explicit `packet_inject_tun`.
- Updated documentation in `docs/facts/virtual_customer.md`.
- All tests pass.

