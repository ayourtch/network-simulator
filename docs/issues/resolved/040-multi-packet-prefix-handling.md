# Multi‑Packet Prefix Handling Issue

**Summary**: When using multiple mock packet files (`packet_files`), the injection direction logic ignored the configurable `tun_a_prefix` and `tun_b_prefix` settings and fell back to the hard‑coded `10.` heuristic. This caused incorrect routing of packets when custom prefixes were configured.

**Location**: `src/tun/mod.rs` – the block handling `packet_files` (lines around the injection decision for each packet).

**Current Behavior**: For each packet, if `inject_opt` is not set, the code checks `packet.src_ip.to_string().starts_with("10.")` to decide the ingress router, regardless of the configured prefixes.

**Expected Behavior**: The injection direction should respect the same prefix logic used for the single‑file case, i.e., use `cfg.tun_ingress.tun_a_prefix` and `cfg.tun_ingress.tun_b_prefix` when determining the ingress router.

**Fix Summary**: Updated the injection decision logic in the `packet_files` handling loop to first check the configured prefixes (`tun_a_prefix` and `tun_b_prefix`). If neither matches, it falls back to the previous heuristic (`10.`). This aligns multi‑packet handling with the single‑file behavior.

**Files Modified**:
- `src/tun/mod.rs`

**Effort Estimate**: Small (≤1 hour).

**Dependencies**: None.
