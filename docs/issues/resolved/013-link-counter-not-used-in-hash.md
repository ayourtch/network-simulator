# Issue 013: Link Counter Not Fully Integrated for Per-Packet Load Balancing

## Summary
The `Link` struct has an atomic counter for per-packet load balancing, but it's only incremented in `simulate_link()` and not used in the path selection hash as specified in Plan 8.

## Resolution
Implemented inclusion of link counters in both standard and multipath forwarding load‑balancing hash calculations. Updated `src/forwarding/mod.rs` and `src/forwarding/multipath.rs` accordingly. Added tests verifying load‑balanced selection.

## Files Modified
- `src/forwarding/mod.rs`
- `src/forwarding/multipath.rs`
- `tests/multipath_forwarding_test.rs` (updated expectations)

All tests now pass and link counters influence load‑balancing decisions as intended.
