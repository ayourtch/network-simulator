# Issue 012: Statistics Not Exposed

**Summary**
Router statistics (received, forwarded, ICMP, lost) are collected in `Fabric` but there is no public API or CLI command to retrieve them, making it difficult for users to observe simulation results.

**Location**
- `src/topology/fabric.rs` provides internal counters.
- No command‑line flag or function in `src/main.rs` outputs these stats.

**Current Behavior**
- Statistics are updated internally but never displayed or exported.

**Expected Behavior**
- Provide a way to query and print router statistics after the simulation ends (e.g., `--stats` flag).
- Optionally expose a JSON endpoint via a function `Fabric::get_statistics_json()`.

**Suggested Solution (Low‑skill)**
1. Add a method `pub fn get_statistics(&self) -> &HashMap<RouterId, RouterStats>` in `Fabric`.
2. In `src/main.rs`, add a command‑line argument `--stats` (using `clap`) that, after `sim.run().await`, iterates over the stats and prints them.
3. Update documentation in `docs/facts/router_statistics.md` to mention the new output.
4. Add a simple integration test that runs a short simulation with `--stats` and checks for non‑zero output.

**Effort Estimate**
Medium (2 hours).