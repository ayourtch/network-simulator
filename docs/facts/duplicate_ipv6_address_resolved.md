# Duplicate IPv6 Address Issue Resolved Fact

The previous duplicate IPv6 address configuration in `src/tun/mod.rs` caused `RTNETLINK answers: File exists` errors during TUN initialization. The redundant `ip -6 addr add` command was removed, making IPv6 TUN setup robust and allowing the simulator to start with IPv6 real interfaces without failure.
