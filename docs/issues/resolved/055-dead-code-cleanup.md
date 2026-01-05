# Issue 055: Dead Code Cleanup – `select_next_hop_by_hash`

**Summary**
The function `select_next_hop_by_hash` in `src/processor.rs` is defined but never used. The compiler emits a warning about dead code, indicating unnecessary code bloat and potential confusion.

**Location**
- `src/processor.rs` – function `select_next_hop_by_hash` near line 350.

**Current Behavior**
- The function is dead code; it is not called anywhere in the codebase.
- It was originally intended for ECMP selection but the logic has been moved to `process_packet_multi` and `select_egress_link`.

**Expected Behavior**
- Remove the dead function to clean up the codebase and eliminate the compiler warning.
- Ensure that any future ECMP logic re‑uses the existing hash‑based selection in `select_egress_link` or `process_packet_multi`.

**Suggested Solution (Low‑skill)**
1. Delete the `select_next_hop_by_hash` function from `src/processor.rs`.
2. Run `cargo build` and verify the warning is gone.
3. Update any documentation or comments that reference this function.

**Effort Estimate**
Small (≈15 minutes).