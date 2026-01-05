# All issues resolved

All previously listed issues have been addressed and moved to `docs/issues/resolved/`. No open issues remain.

# Issue 002: TTL Decrement Not Implemented

**Summary**
TTL (Time‑to‑Live) for IPv4 packets is not decremented when packets are processed, violating basic IP forwarding behavior.

**Location**
- File: `src/processor.rs` (or similar packet processing function).
- Function handling forwarding.

**Current Behavior**
- Packets retain original TTL, leading to infinite loops in simulations.

**Expected Behavior**
- Decrement TTL by 1 for each hop; drop packet and generate ICMP Time Exceeded when TTL reaches 0.

**Suggested Solution**
1. In the processing pipeline, after parsing, check if packet is IPv4 (`match packet.src_ip`).
2. Reduce `packet.ttl` (or appropriate field) by 1.
3. If TTL becomes 0, generate an ICMP Time Exceeded message (refer to issue 005 stub).
4. Ensure the modified packet is used for further routing.

---

# Issue 003: Port Extraction Not Implemented

**Summary**
Transport layer ports (TCP/UDP) are not extracted from packets, limiting ability to implement NAT, firewall rules, or statistics.

**Location**
- `src/packet.rs` or processing module where payload is examined.

**Current Behavior**
- Only IP headers are parsed; payload is left opaque.

**Expected Behavior**
- For IPv4/IPv6 packets with TCP or UDP protocols, extract source and destination ports.

**Suggested Solution**
1. After IP header parsing, inspect the `protocol` field (TCP=6, UDP=17).
2. Ensure enough bytes remain for the transport header.
3. Read the first 4 bytes as source/destination ports (big‑endian).
4. Store them in the `Packet` struct (add optional `src_port`, `dst_port`).
5. Add tests for both TCP and UDP packets.

---

# Issue 004: MTU Enforcement Not Implemented

**Summary**
The simulator does not enforce the configured MTU, allowing packets larger than the link MTU to be forwarded.

**Location**
- `src/processor.rs` or routing logic where packet size is considered.

**Current Behavior**
- Packets are forwarded regardless of size.

**Expected Behavior**
- Drop packets exceeding the MTU and optionally generate ICMP Fragmentation Needed.

**Suggested Solution**
1. Retrieve the link MTU from the configuration (already present in `SimulationConfig`).
2. In packet processing, compare `packet.raw.len()` to MTU.
3. If larger, drop and optionally create ICMP message (see issue 006).

---

# Issue 005: ICMP Time Exceeded Stub Not Implemented

**Summary**
When TTL reaches 0, an ICMP Time Exceeded message should be generated, but currently only a stub exists.

**Location**
- `src/icmp.rs` (or wherever ICMP generation is handled).

**Suggested Solution**
1. Implement a function `fn icmp_time_exceeded(original: &Packet) -> Packet` that builds an ICMP Type 11 message.
2. Populate appropriate fields (source IP = router, destination IP = original source).
3. Insert the new packet into the processing pipeline for sending back to the origin.

---

# Issue 006: ICMP Fragmentation Needed Stub Not Implemented

**Summary**
When a packet exceeds the MTU, an ICMP Destination Unreachable – Fragmentation Needed (Type 3 Code 4) should be generated.

**Suggested Solution**
- Similar to Issue 005, create a function that builds the ICMP packet with the MTU field set.

---

# Issue 007: ICMPv6 Not Implemented

**Summary**
ICMPv6 handling (e.g., Time Exceeded, Packet Too Big) is missing, preventing proper IPv6 error handling.

**Suggested Solution**
- Extend the ICMP module to support IPv6 types, mirroring the IPv4 implementations.

---

# Issue 008: Hop‑by‑Hop Forwarding Not Implemented

**Summary**
IPv6 hop‑by‑hop options are not processed, which may be required for certain test scenarios.

**Suggested Solution**
- Parse the Hop‑by‑Hop header when present and handle any extension headers.

---

# Issue 009: TUN Write‑Back Not Implemented (Real TUN Mode)

**Summary**
When using real TUN interfaces, packets are not written back to the opposite TUN device, preventing a Linux host behind the simulator from communicating.

**Current Status**
- Resolved by issue 028; dual‑TUN support added.

---

# Issue 010: Virtual Customer Not Used

**Summary**
The configuration includes a `virtual_customer` section that is never referenced, missing an opportunity to simulate end‑user traffic.

**Suggested Solution**
- Integrate the virtual customer into the packet generation pipeline or provide a CLI flag to enable it.

---

# Issue 011: ICMP Routing Not Implemented

**Summary**
When a packet cannot be routed, the simulator should generate an ICMP Destination Unreachable, but this is missing.

**Suggested Solution**
- Detect routing failures and emit ICMP Type 3 messages.

---

# Issue 012: Statistics Not Exposed

**Summary**
Router statistics (packet counts, bytes, drops) are collected internally but not exposed via any API or CLI.

**Suggested Solution**
- Add a `stats` command or dump to a JSON file after simulation.

---

# Issue 013: Link Counter Not Used in Hash

**Summary**
The link counter intended for load‑balancing hash calculations is never incorporated, reducing realism of ECMP.

**Suggested Solution**
- Modify the hash function in `src/routing.rs` to include the link counter.

---

# Issue 014: Destination Detection Incorrect

**Summary**
The logic that determines packet destination (e.g., which router or interface) contains bugs leading to mis‑routing.

**Suggested Solution**
- Review and correct the `determine_destination` function, ensuring it respects the ingress/egress config.

---

# Issue 016: Bidirectional Link Validation

**Summary**
Links should be validated as bidirectional, but current validation only checks one direction.

**Suggested Solution**
- In `SimulatorConfig::validate`, ensure that for each link A_B, a corresponding B_A exists or is treated as the same.

---

# Issue 017: Router Name Validation Not Strict

**Summary**
Router identifiers are not validated for allowed characters, potentially allowing illegal names.

**Suggested Solution**
- Add regex validation in `RouterId::validate` to enforce alphanumeric and underscores.

---

# Issue 018: Ingress Router Validation Missing

**Summary**
The configuration fields `tun_ingress.tun_a_ingress` and `tun_ingress.tun_b_ingress` are not validated against the topology.

**Current Status**
- Added validation in `SimulatorConfig::validate` (see code). Ensure tests cover missing routers.

---

# Issue 019: Link References Validation

**Summary**
Link definitions may reference routers that do not exist; validation partially checks this but may miss case‑sensitivity.

**Suggested Solution**
- Ensure router ID comparison is case‑insensitive or enforce case‑consistent naming.

---

# Issue 021: IPv4 Checksum Not Implemented

**Summary**
IPv4 header checksum calculation and verification are missing, which can affect packet integrity checks.

**Suggested Solution**
- Implement checksum calculation in the packet builder and verify on parsing.

---

# Issue 021 (duplicate): Mock TUN Packet Handling

**Summary**
Mock TUN packet handling has been partially implemented but may still lack proper injection direction handling for IPv6.

**Current Status**
- CIDR based direction logic added; verify IPv6 handling works.

---

# Issue 022: Multiple Mock Packets and Simultaneous Handling

**Summary**
Support for multiple mock packet files is present, but concurrency or ordering issues may exist.

**Suggested Solution**
- Add tests that run two packet files concurrently and verify correct routing.

---

# Issue 023: IPv4 ICMP Error Stub Not Resolved

**Summary**
Generation of ICMP error messages for IPv4 (e.g., Destination Unreachable) is still a stub.

**Suggested Solution**
- Implement full ICMP packet construction as per RFC 792.

---

# Issue 024: TTL Expiration ICMP Not Generated

**Summary**
When TTL expires, the simulator should send an ICMP Time Exceeded, but this is not yet functional.

**Suggested Solution**
- Combine with Issue 005 implementation.

---

# Issue 025: Multipath Processing Is No‑Op

**Summary**
Multipath routing logic is present but does not actually select alternate paths.

**Suggested Solution**
- Implement ECMP hashing and forwarding to multiple possible next‑hops.

---

# Issue 026: Router Statistics Never Updated

**Summary**
Counters for packets processed per router are never incremented.

**Suggested Solution**
- Increment stats in `process_packet` and expose via CLI.

---

# Issue 027: Forwarding Lacks Destination Detection

**Summary**
The forwarding code does not correctly determine the outgoing interface based on destination IP.

**Suggested Solution**
- Ensure routing tables are consulted and the correct `Destination` enum is used.

---

# Issue 029: Real TUN Direction Detection Fragile

**Summary**
Direction detection for real TUN interfaces relies on string prefixes, which is brittle.

**Current Status**
- Replaced with CIDR based detection; verify correctness.

---

# Issue 030: TUN Device Memory Safety

**Summary**
Unsafe handling of file descriptors may cause memory safety issues.

**Suggested Solution**
- Review usage of `FromRawFd`/`IntoRawFd` and ensure proper ownership.

---

# Issue 031: ICMP Routing After Generation Incorrect

**Summary**
Generated ICMP packets are not routed back correctly.

**Suggested Solution**
- After ICMP creation, set the correct ingress router and destination.

---

# Issue 033: Packet Loss Not Tracked

**Summary**
Packets dropped due to errors are not accounted for in statistics.

**Suggested Solution**
- Increment a loss counter in error paths.

---

# Issue 034: Documentation Mismatch

**Summary**
Some docs still describe old behavior (string prefix matching). Updated docs exist, but ensure all references are aligned.

**Suggested Solution**
- Search the repository for `starts_with("10.")` mentions and update.

---

# Issue 035: No End‑to‑End Tests

**Summary**
There are no integration tests that run the full simulator with real TUN interfaces.

**Suggested Solution**
- Add a test that spins up two network namespaces, creates TUN devices, runs the simulator, and verifies packet exchange.

---

# Issue 036: Invalid Real TUN Address

**Summary**
Configuration validation for real TUN IP addresses may allow invalid strings.

**Current Status**
- Validation added in `SimulatorConfig::validate`.

---

# Issue 037: TUN Cleanup on Shutdown

**Summary**
When the simulator exits, TUN interfaces may remain up.

**Current Status**
- Shutdown handling added to bring interfaces down.

---

# Issue 038: IPv6 Real TUN Support

**Summary**
Real TUN devices support IPv6 addresses; ensure configuration and creation handle this.

**Current Status**
- IPv6 handling added in `create_async_tun`.

---

# Issue 039: IPv6 Netmask Not Applied

**Summary**
IPv6 prefix length must be applied when configuring the interface.

**Current Status**
- Implemented via `ip -6 addr add <addr>/<prefix>`.

---

# Issue 040: Multi‑Packet Prefix Handling

**Summary**
Multiple mock packet files should each respect their own prefix settings; this is now implemented.

---

# Issue 041: Remove Unused Imports in `src/tun/mod.rs`

**Summary**
Unused imports were cleaned up.

---

# Issue 042: Dual TUN Host Setup Example

**Summary**
Documentation example added.

---

# Issue 046: Implement Multipath Processing

**Summary**
Multipath processing remains a stub.

---

# Issue 047: Multipath Destination Detection

**Summary**
Destination detection for multipath routes needs refinement.

---

# Issue 048: IPv6 Injection Prefix

**Summary**
IPv6 injection prefix handling added.

---

# Issue 049: Prefix Matching Improvement

**Summary**
CIDR based prefix matching implemented and issue resolved.
