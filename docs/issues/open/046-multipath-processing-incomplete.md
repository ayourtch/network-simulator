# Issue 046: Multipath Processing Incomplete

**Summary**
The simulator has a placeholder multipath processing path (`process_packet_multi`) that currently mirrors the single‑path logic without truly handling equal‑cost multipath routing. Proper ECMP should select among multiple next hops per destination.

**Location**
- `src/processor.rs` – `process_packet_multi`.
- `src/routing/multipath.rs` – defines `MultiPathTable` but not fully utilized.

**Current Behavior**
- `process_packet_multi` simply calls the same forwarding logic as single‑path, ignoring the list of equal‑cost next hops.

**Expected Behavior**
- When a destination has multiple equal‑cost next hops, the router should randomly or hash‑based select one for each packet, updating counters accordingly.

**Suggested Solution (Low‑skill)**
1. In `process_packet_multi`, retrieve the list of equal‑cost next hops from `multipath_tables`.
2. Use the same `select_egress_link` helper but pass the multipath table instead of the single table.
3. Ensure the chosen link's counter is incremented as in single‑path.
4. Add unit tests that create a topology with ECMP and verify that packets are distributed across the parallel links.

**Effort Estimate**
Medium (2‑3 hours).