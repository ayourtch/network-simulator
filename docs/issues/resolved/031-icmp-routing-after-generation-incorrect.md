# Issue 031: ICMP Packet Routing After Generation Fixed

## Summary
The previous implementation generated ICMP error packets but had issues with IPv4 parsing, direction handling, and TTL management. These problems prevented proper routing of ICMP replies back to the original source.

## Fix Summary
- Added a dedicated `route_icmp_to_source` helper (inline within `process_packet`) that parses the generated ICMP packet, determines the correct opposite destination, and forwards it using the existing routing logic.
- Ensured IPv4 ICMP generation now creates a valid IP header with correct source/destination and TTL (64).
- Updated TTL expiration handling to use the helper, avoiding duplicate code.
- Updated MTU‑exceeded handling to use the same helper for consistency.
- Adjusted comments and added documentation explaining the ICMP routing flow.
- Added tests (`tests/icmp_routing_test.rs`) confirming that ICMP packets have swapped source/destination IPs after processing.

The simulator now correctly routes ICMP error packets back to the packet’s origin, handling both IPv4 and IPv6 cases.
