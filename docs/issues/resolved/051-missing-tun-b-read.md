# Issue 051: Missing Read Handling for Real TUN B

## Summary
`src/tun/mod.rs::start` creates asynchronous read/write halves for both real TUN devices (`real_tun_a` and `real_tun_b`). However, the main event loop only includes a `select!` branch for reading packets from TUN A. There is no corresponding branch to read packets arriving on TUN B, meaning traffic originating from the host behind TUN B will never be processed or forwarded to TUN A.

## Expected Behavior
- The loop must contain a branch that reads from `async_dev_b_reader`, parses the packet, determines the correct ingress router (ingress B), forwards the packet through the fabric, and writes the processed packet to the opposite TUN (A).
- This mirrors the existing handling for TUN A and enables bidirectional communication, fulfilling the dual‑TUN requirement described in the project plans.

## Suggested Solution
1. Add a `read_res = async_dev_b_reader.read(&mut buf_b)` branch inside the `select!` macro, analogous to the existing TUN A branch.
2. Use the same CIDR/IPv6 prefix logic to decide the destination (`Destination::TunA`).
3. After processing, construct the TUN header and write the packet to `async_dev_a_writer`.
4. Ensure proper error handling and logging, mirroring the TUN A branch.

## Acceptance Criteria
- Packets read from TUN B are correctly processed and written to TUN A.
- Integration tests that simulate traffic in both directions (e.g., using mock TUN devices) pass.
- No panics or deadlocks when both TUN interfaces are active.
