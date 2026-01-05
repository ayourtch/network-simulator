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

**Summary**
The `VirtualCustomerConfig` includes optional `size` and `rate` fields intended to control the packet payload size and generation rate (packets per second). The current implementation in `src/tun/mod.rs` generates a single fixed‑size IPv4 header (20 bytes) packet at startup and ignores both `size` and `rate`.

**Location**
- `src/config.rs` defines `VirtualCustomerConfig { src_ip, dst_ip, protocol, size, rate }`.
- `src/tun/mod.rs` creates a 20‑byte IPv4 packet using only `src_ip`, `dst_ip`, and `protocol`.

**Current Behavior**
- Only one packet is generated regardless of `size` or `rate`.
- Payload is always empty; `size` is not used to extend the packet.
- No periodic generation loop; `rate` is ignored, so traffic patterns cannot be simulated.

**Expected Behavior**
- If `size` is provided, the generated packet should include a payload of the specified length (minimum IPv4 header size of 20 bytes plus `size` bytes of data, zero‑filled or configurable).
- If `rate` is provided, the simulator should spawn an asynchronous task that generates packets at the given packets‑per‑second rate, injecting each into the processing pipeline using the same ingress/destination logic.
- Support IPv6 virtual customers as well, respecting `src_ip`/`dst_ip` being IPv6 addresses.

**Suggested Solution (Low‑skill)**
1. In `src/tun/mod.rs`, after constructing the base IPv4 header, if `vc.size` is `Some(sz)`, extend `raw` with `vec![0u8; sz]` before checksum calculation (re‑calculate checksum after payload addition).
2. Create an async loop (e.g., using `tokio::time::interval`) that runs at `vc.rate.unwrap_or(1)` packets per second, generating a new `PacketMeta` each tick and processing it via `process_packet` / `process_packet_multi`.
3. Detect IPv6 addresses: attempt to parse `src_str`/`dst_str` as `IpAddr` (use `std::net::IpAddr::from_str`). If IPv6, build an IPv6 header (40 bytes) with appropriate fields, compute IPv6 pseudo‑header checksum for ICMP generation, etc.
4. Update documentation in `docs/facts/virtual_customer.md` to reflect the new behavior.
5. Add unit tests that verify multiple packets are generated at the configured rate (use a mock `Fabric` and check counters) and that packet size matches `size`.
