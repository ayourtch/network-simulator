## Issue Resolved: Simulator exits immediately

**Problem**: The simulator terminated shortly after start when running with real TUN interfaces, as zero‑byte reads from the TUN devices were treated as EOF, causing the main loop to break.

**Fix**: Updated the TUN read handling in `src/tun/mod.rs` to treat `Ok(0)` as a non‑fatal condition: log a debug message and `continue` the loop instead of breaking. This ensures the simulation stays alive until a shutdown signal (Ctrl‑C) is received.

**Additional Documentation**: Added `docs/facts/zero_byte_read_handling.md` describing the new behavior.
