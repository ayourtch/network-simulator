# Issue 001: Documentation Mismatch – IPv6 Packet Parsing

## Summary
The resolved issue `001-ipv6-packet-parsing-not-implemented.md` claimed IPv6 packet parsing was missing, but the current implementation in `src/packet/mod.rs` fully parses IPv6 headers, extracting source/destination IPs, next‑header, hop‑limit and ports where applicable.

## Resolution
- Updated documentation to reflect the actual state.
- Added a fact file `docs/facts/ipv6_packet_parsing.md`.
- Closed this issue by moving it to the resolved directory.
