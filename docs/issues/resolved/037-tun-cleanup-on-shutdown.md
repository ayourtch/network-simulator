# TUN Cleanup on Shutdown Issue

**Summary**: When the simulator is stopped (Ctrl‑C), the dual real‑TUN loop exits, but the underlying TUN interfaces remain up in the system. This can leave stray network interfaces and cause conflicts on subsequent runs.

**Location**: `src/tun/mod.rs` – the shutdown branch of the `select!` loop ends the function without bringing the TUN devices down.

**Current Behavior**: TUN devices stay active after the process exits.

**Expected Behavior**: The simulator should bring both TUN devices down (e.g., by calling `dev.down()` or equivalent) before exiting.

**Recommended Solution**: Store the `TunDevice` objects (or a handle that can call `down()`) before converting them to async files. After the loop breaks, invoke `dev_down(&dev_a); dev_down(&dev_b);` where `dev_down` uses the `tun` crate's `down()` method. Ensure proper error handling.

**Files to Modify**: `src/tun/mod.rs` – adjust `create_async_tun` to return both the async file and the original `TunDevice` (or keep a separate handle), and add cleanup after the loop.

**Effort Estimate**: Small (< 2 hours).

**Dependencies**: None.

**Related Plans**: Issue 037.
