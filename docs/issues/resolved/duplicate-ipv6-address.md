# Issue: Duplicate IPv6 address configuration in TUN setup

## Problem
The `create_async_tun` function adds an IPv6 address to the interface using the `ip -6 addr add` command **twice**:
1. Immediately after parsing the IPv6 address (line ~450).
2. Again after building the `TunBuilder` (line ~489).
If the interface already has the address from the first command, the second command fails with `RTNETLINK answers: File exists`, causing the simulator to abort during startup on IPv6 configurations.

## Suggested Solution
- Remove the first `ip` command block (lines 447‑459) and keep only the one after the builder, or
- Guard the second command to run only if the address is not already present (e.g., check the command's exit status for `File exists` and ignore it).

This will make IPv6 TUN initialization robust and allow users to run the simulator with IPv6 real interfaces.
