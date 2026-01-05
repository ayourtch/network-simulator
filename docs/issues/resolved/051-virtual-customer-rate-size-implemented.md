# Issue 051: Virtual Customer Rate and Size Implemented

**Summary**
The `VirtualCustomerConfig` now fully supports `size` and `rate`. Generated packets include the specified payload size, and packets are emitted periodically at the configured rate using a `tokio::time::Interval`. IPv4 and IPv6 packets are both handled.

**Resolution**
- Updated `src/tun/mod.rs` to extend packet payload by `vc.size` bytes and to generate packets periodically according to `vc.rate`.
- Added CIDR‑based ingress detection and IPv6 support for virtual customer packets.
- Updated documentation and facts accordingly.

**Verification**
- All tests pass. Integration tests confirm periodic packet generation and correct payload sizes.

*Closed as implemented.*