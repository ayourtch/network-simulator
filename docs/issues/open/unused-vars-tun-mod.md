# Issue: Unused variables in `src/tun/mod.rs`

## Problem
Compiler warnings show several unused variables (e.g., `v4`, `v6`, `netmask`). They arise from match arms that only need the address for configuration side‑effects.

## Suggested Solution
* Prefix unused bindings with an underscore (`_v4`, `_v6`, `_netmask`).
* Alternatively, restructure the code to avoid unnecessary bindings, e.g., using `if let Ok(_) = ...` where the value is not needed.

Cleaning these up will produce a warning‑free build.
