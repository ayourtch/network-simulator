# Issue 013: Link Counter Not Used in Hash

**Summary**
The load‑balancing hash in `src/forwarding/mod.rs` currently only considers packet fields (src/dst IP/port, protocol) but ignores the per‑link counter that should influence ECMP decisions for more realistic traffic distribution.

**Location**
- `src/forwarding/mod.rs` – function `select_egress_link`.

**Current Behavior**
- All equal‑cost links are chosen purely based on a hash of packet metadata, leading to static distribution and not reflecting link utilization.

**Expected Behavior**
- Incorporate each link's `counter` (number of packets sent) into the hash or selection algorithm so that less‑used links are preferred, achieving better load balancing.

**Suggested Solution (Low‑skill)**
1. Extend the hash calculation to include `link.counter` (e.g., `hash ^ link.counter`).
2. Update the selection logic to pick the link with the lowest resulting hash value.
3. Ensure the counter is incremented after a packet is forwarded (already done in `simulate_link`).
4. Add a test that sends many packets over multiple equal‑cost links and verifies a more even distribution.

**Effort Estimate**
Small (1 hour).