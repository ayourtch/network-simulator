# Issue 050: Real TUN Device Support Not Implemented

## Summary
The simulator configuration allows specifying real TUN interfaces (`real_tun_a` and `real_tun_b`). However, `src/tun/mod.rs::start` contains no implementation for reading from or writing to real TUN devices. The function merely skips TUN handling when real TUN addresses are empty and otherwise proceeds only with mock packet files or virtual customers.

## Expected Behavior
- When `real_tun_a.address` and/or `real_tun_b.address` are configured, the simulator should:
  1. Open both TUN devices (`real_tun_a.name` and `real_tun_b.name`).
  2. Asynchronously read packets from each device.
  3. Process packets through the fabric using the appropriate ingress router.
  4. Write the processed packet to the *opposite* TUN device, enabling a Linux host placed behind one TUN to communicate through the virtual network to the other side.

## Current Behavior
- The code checks if both addresses are empty and returns early.
- If addresses are non‑empty, the function still never creates or uses any TUN device; it only handles mock packet files and virtual customers.
- Consequently, users cannot place a Linux host behind the simulator as described in the project plans.

## Suggested Solution
1. Add a new branch in `start` that creates two `tokio_tun::TunDevice` (or appropriate library) instances for `real_tun_a` and `real_tun_b`.
2. Spawn separate async tasks (or use `select!`) to read from each device, determine the correct ingress router using the CIDR/IPv6 prefix logic already present, and forward the packet to the opposite device after processing.
3. Ensure graceful shutdown handling (e.g., on SIGINT) closes both devices.
4. Update documentation (`docs/example/dual_tun_host_setup.md`) to reflect the required configuration fields.
5. Add unit/integration tests exercising the dual‑TUN path (see Issue 054).

## Acceptance Criteria
- `cargo test` passes all existing tests.
- New tests for real TUN handling (mocked or using a virtual TUN interface) pass.
- The simulator can be launched with a real TUN configuration and successfully forwards packets between the two interfaces.
- No runtime panics when real TUN addresses are provided.
