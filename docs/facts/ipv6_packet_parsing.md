# IPv6 Packet Parsing Fact

The simulator now fully parses IPv6 packets in `src/packet/mod.rs`. It extracts source/destination IPs, next‑header, hop‑limit, and ports when applicable, and also correctly skips IPv6 Hop‑by‑Hop extension headers to locate the transport header. This enables proper handling of IPv6 traffic throughout the simulation, including packets that contain Hop‑by‑Hop options.
