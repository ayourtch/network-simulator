# Needless Borrow Fix Fact

The `src/tun/mod.rs` module no longer passes `&fabric` to `compute_multi_path_routing`; the unnecessary borrow was removed, silencing the Clippy `needless_borrow` warning.
