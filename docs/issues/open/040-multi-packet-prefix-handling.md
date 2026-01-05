

**Summary**: When using multiple mock packet files (`packet_files`), the injection direction logic ignores the configurable `tun_a_prefix` and `tun_b_prefix` settings and falls back to the hard‑coded `10.` heuristic. This can cause incorrect routing of packets when custom prefixes are configured.

**Location**: `src/tun/mod.rs` – the block handling `packet_files` (lines around the injection decision for each packet).

**Current Behavior**: For each packet, if `inject_opt` is not set, the code checks `packet.src_ip.to_string().starts_with("10.")` to decide the ingress router, regardless of the configured prefixes.

**Expected Behavior**: The injection direction should respect the same prefix logic used for the single‑file case, i.e., use `cfg.tun_ingress.tun_a_prefix` and `cfg.tun_ingress.tun_b_prefix` when determining the ingress router.

**Recommended Solution**:
- Refactor the injection decision in the `packet_files` branch to mirror the logic from the single‑file branch, checking the configured prefixes before falling back to the default heuristic.
- Update any related documentation/comments to reflect the new behavior.

**Files to Modify**:
- `src/tun/mod.rs` – adjust the injection logic inside the `packet_files` handling loop.

**Effort Estimate**: Small (≤1 hour).

**Dependencies**: None.
