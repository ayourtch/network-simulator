# Issue 100: Verify dual TUN host behind simulator

## Summary
The simulator now supports two real TUN devices (`real_tun_a` and `real_tun_b`) and routes packets between them. However, there is no automated test or documentation confirming that a Linux host placed in a network namespace behind one of the TUN interfaces can communicate through the virtual network and reach the other side.

## Steps to Verify
1. Create a network namespace and attach a TUN interface (e.g., `tunA`).
2. Configure the TUN interface with the address specified in `config.toml`.
3. Run the simulator with the matching configuration.
4. Send a packet from the host inside the namespace to an address reachable via the other TUN interface (`tunB`).
5. Verify that the packet exits through `tunB` and is processed by the simulator (e.g., using `tcpdump` or logging).

## Suggested Solution
- Add an integration test under `tests/dual_tun_host_integration_test.rs` that automates the above steps using `ip netns` and `ip tuntap`.
- Document the setup procedure in `docs/example/dual_tun_host_setup.md` with clear commands.
- Ensure CI runs this test (may require privileged runner).

## Acceptance Criteria
- The test passes on a machine with `CAP_NET_ADMIN`.
- Documentation updated with example commands.
- No runtime warnings about missing TUN devices when running the test.
