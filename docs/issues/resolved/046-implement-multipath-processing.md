# Issue 046: Implement Proper Multipath Packet Processing

**Summary**
`process_packet_multi` in `src/processor.rs` was previously a placeholder that reused single‑path logic, ignoring the `MultiPathTable` and not performing true multipath routing.

**Resolution**
Implemented full multipath processing:
- Uses `MultiPathTable` to obtain equal‑cost next‑hop entries.
- Selects a next hop via a hash of the packet's 5‑tuple (`select_next_hop_by_hash`).
- Handles TTL expiration, ICMP generation, packet loss, and MTU errors.
- Updates router statistics (received, forwarded, ICMP, lost).
- Retrieves links via `Fabric::get_link`.

**Effort**
Medium (≈3‑4 hours).
