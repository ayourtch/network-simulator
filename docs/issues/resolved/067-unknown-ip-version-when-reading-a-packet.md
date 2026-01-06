# Issue: Unknown IP version when reading a packet

The error was caused by not stripping the TUN header before parsing. The code now correctly removes the 4‑byte header, checks the protocol (0x0800 for IPv4, 0x86DD for IPv6), and parses the payload, fixing the unsupported IP version error.
