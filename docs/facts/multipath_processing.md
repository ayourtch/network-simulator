# Multipath Processing Fact

The simulator now fully supports multipath routing. `process_packet_multi` uses the `MultiPathTable` to select among equal‑cost next hops based on a hash of the packet's 5‑tuple, updates router statistics, handles TTL, ICMP generation, packet loss, and MTU errors.
