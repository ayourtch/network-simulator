# Issue 008: Hop‑by‑Hop Forwarding Not Implemented

## Summary
IPv6 hop‑by‑hop options are not parsed or processed. Certain test scenarios or protocols may rely on hop‑by‑hop headers, and the simulator currently ignores them, potentially mis‑routing packets.

## Location
- File: `src/packet/mod.rs`
- Function: `parse()` where IPv6 header is handled.

## Current Behavior
- Parses the fixed 40‑byte IPv6 header and proceeds directly to the payload, without checking the `Next Header` chain for a Hop‑by‑Hop Options header (value 0).

## Expected Behavior
- Detect if the IPv6 `Next Header` field is 0 (Hop‑by‑Hop Options).
- Parse the Hop‑by‑Hop header length and skip over it to reach the actual transport header.
- Ensure that subsequent parsing (e.g., port extraction) uses the correct offset.

## Suggested Solution (low‑skill steps)
1. After reading the IPv6 header, check `let next_header = data[6];`.
2. If `next_header == 0`, read the Hop‑by‑Hop header length byte at offset 8 (`data[8]`). The header length is expressed in 8‑byte units; total size = (len + 1) * 8.
3. Advance the parsing offset by that size and read the actual next header (`data[8 + header_len]`).
4. Continue with port extraction using the new offset.
5. Add unit tests with a crafted IPv6 packet containing a Hop‑by‑Hop header.

---
