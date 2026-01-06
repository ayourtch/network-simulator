# Issue 063: Clippy warning – needless borrow in compute_multi_path_routing call

## Summary
In `src/tun/mod.rs` the call to `compute_multi_path_routing` passes `&fabric` even though the function takes `&Fabric` by reference and the borrow is immediately dereferenced, triggering the Clippy `needless_borrow` warning.

## Suggested Solution
Replace the call with `compute_multi_path_routing(fabric, …)` (remove the `&`). This eliminates the unnecessary borrow and silences the warning.

## Fix
Edit `src/tun/mod.rs` to change the call accordingly.
