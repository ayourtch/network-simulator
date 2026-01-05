# Issue 011: ICMP Routing Not Implemented

**Summary**
When a packet cannot be routed (no matching routing table entry or next hop), the simulator should generate an ICMP Destination Unreachable (Type 3 Code 0) message back to the source. Currently the processor simply breaks the forwarding loop without generating any ICMP error.

**Location**
- `src/processor.rs` in `process_packet` and `process_packet_multi`. The code checks for a routing table and breaks if missing, but does not create an ICMP packet.

**Current Behavior**
- Packets that cannot be forwarded are silently dropped. No ICMP error is produced, making debugging and realistic simulation impossible.

**Expected Behavior**
- Detect routing failure, generate an ICMP Destination Unreachable packet, and send it back towards the original source using the opposite destination.

**Suggested Solution (Low‑skill)**
1. In `process_packet`/`process_packet_multi`, after `let table = tables.get(&ingress)` returns `None`, call `icmp::generate_icmp_error(&packet, 3, 0)` to create the ICMP message.
2. Parse the ICMP bytes with `packet::parse`, swap the destination using `opposite_destination`, and continue processing the ICMP reply.
3. Increment the router's ICMP counter via `router.increment_icmp()`.
4. Add a unit test that forces a routing miss and checks that an ICMP packet is generated.

**Effort Estimate**
Small (1 hour).