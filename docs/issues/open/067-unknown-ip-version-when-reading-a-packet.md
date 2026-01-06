Problem: running with a real tun interface and sending a packet results in the following messages:

READ from A: Ok(88)
2026-01-06T10:53:05.063000Z ERROR network_simulator::tun: Failed to parse packet from TUN A: unsupported IP version

Root cause: unknown

Steps to solve: investigate the code path, and add the debug printing of received hex dump of data from tun interfaces, for easy debugging.
