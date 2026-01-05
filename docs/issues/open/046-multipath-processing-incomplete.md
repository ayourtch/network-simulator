# Issue 046: Multipath Processing Incomplete

## Summary
The multipath routing engine in `src/processor.rs` currently reuses the single‑path logic and does not actually implement ECMP (equal‑cost multi‑path) forwarding. Packets are always forwarded via the first entry in the multipath table, defeating the purpose of the multipath feature.

## Location
- File: `src/processor.rs`
- Function: `process_packet_multi()`

## Current Behavior
- Retrieves the multipath table for the current router.
- Selects a next‑hop using `select_next_hop_by_hash`, but this function is not defined in the repository (the code will not compile).
- Even if it compiled, the selected next hop is not used to forward the packet; the logic falls back to the single‑path `select_egress_link`.

## Expected Behavior
- Implement true ECMP: choose among multiple equal‑cost next‑hops based on a hash of the packet 5‑tuple.
- Forward the packet over the selected link, updating statistics accordingly.
- Handle the case where the multipath entry list is empty (log and drop).

## Suggested Solution
1. **Define `select_next_hop_by_hash`** in a new module `src/routing/multipath.rs` (or within `processor.rs`). It should:
   ```rust
   fn select_next_hop_by_hash(packet: &PacketMeta, entries: &[RouterId]) -> &RouterId {
       use std::hash::{Hash, Hasher};
       let mut hasher = std::collections::hash_map::DefaultHasher::new();
       packet.src_ip.hash(&mut hasher);
       packet.dst_ip.hash(&mut hasher);
       packet.src_port.hash(&mut hasher);
       packet.dst_port.hash(&mut hasher);
       let idx = (hasher.finish() as usize) % entries.len();
       &entries[idx]
   }
   ```
2. In `process_packet_multi`, replace the placeholder call with the new function and **use the returned router id** to locate the appropriate link via `fabric.get_link(&ingress, next_hop_id)`.
3. Remove the unused import of `select_egress_link` for the multipath path.
4. Add unit tests in `tests/multipath_test.rs` verifying that two equal‑cost routes receive traffic roughly proportionally.

## Effort Estimate
Medium (2–3 hours) – mainly adding the helper, wiring it into the loop, and writing tests.

## Related Plans
- Plan 6: Multipath routing and load‑balancing.
- Issue 047 (multipath destination detection) – will be addressed together.
