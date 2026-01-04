# Resolved: ICMP Routing Not Implemented – issue closed

Implemented basic ICMP routing: when TTL expires or MTU exceeded, an ICMP error packet is generated, parsed, and re‑processed through the fabric with the original ingress router as the destination (using `Destination::TunA`). This ensures ICMP packets are sent back toward the source. The logic is integrated into `process_packet` and `process_packet_multi`.
