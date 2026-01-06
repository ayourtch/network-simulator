# Clippy Collapsible Else‑If Fact

The `src/tun/mod.rs` module now begins with `#![allow(clippy::collapsible_else_if)]`. This silences the Clippy warning about nested `else { if … }` blocks, which are used extensively for CIDR‑based direction detection. The attribute keeps the code tidy without altering functionality.
