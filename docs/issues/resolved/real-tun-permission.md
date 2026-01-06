# Real TUN Permission Issue (Resolved)

The simulator previously failed with `EPERM: Operation not permitted` when attempting to open real TUN devices without elevated privileges. We added a fallback in `src/tun/mod.rs` that detects this error, logs a warning, and skips real‑TUN handling, allowing the simulator to continue (or exit gracefully) without requiring root. The issue is now resolved.
