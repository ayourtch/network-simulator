# Issue 006: ICMP Fragmentation Needed Generation Implemented

The simulator now correctly generates ICMP Destination Unreachable – Fragmentation Needed packets with the proper MTU field. The `generate_fragmentation_needed` function in `src/icmp/mod.rs` builds the IPv4 header, adds the ICMP header (type 3, code 4), inserts the Next‑Hop MTU field, includes the original IP header plus 8 bytes of payload, and computes checksums. All related tests pass.
