# Issue: Unused variables in `src/tun/mod.rs`

The module previously generated warnings for unused bindings such as `v4`, `v6`, and `netmask`. The code was refactored to prefix these bindings with underscores (`_v4`, `_v6`, `_netmask`) and the `vc_interval` variable was integrated into the main event loop. All compiler warnings have been eliminated, improving code health.
