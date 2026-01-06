# Dual TUN Host Integration Needed Fact

The simulator currently lacks an automated integration test that verifies a Linux host placed behind one of the real TUN interfaces can communicate through the virtual network to the other TUN interface. Adding such a test (using `ip netns` and `ip tuntap`) is required for full validation of the dual‑TUN architecture.
