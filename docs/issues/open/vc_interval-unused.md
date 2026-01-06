# Issue: Unused `vc_interval` variable in TUN module

## Problem
The variable `vc_interval` is declared and assigned in `src/tun/mod.rs` but never used. This generates compiler warnings and indicates missing functionality for periodic virtual‑customer packet generation.

## Suggested Solution
1. Hook the interval into the main `select!` loop (uncomment and adapt the existing commented code).
2. On each tick, call `generate_virtual_packet` with the configured `virtual_customer`.
3. Ensure the interval respects the `rate` field and handles the initial burst correctly.

Implementing this will remove the warnings and provide the intended periodic traffic generation feature.
