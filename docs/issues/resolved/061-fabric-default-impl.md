# Issue 061: Missing `Default` implementation for `Fabric`

## Summary
The `src/topology/fabric.rs` module defines a `Fabric` struct with a `new()` constructor but does not implement the `Default` trait. Several parts of the code (e.g., tests or other modules) may benefit from a `Default::default()` implementation, and Clippy warns about this (`new_without_default`). Adding a `Default` impl would simplify fabric creation and silence the warning.

## Suggested Solution
1. Add `impl Default for Fabric { fn default() -> Self { Self::new() } }` at the end of the file (or after the existing `impl Fabric`).
2. Ensure the implementation is covered by tests (e.g., instantiate a `Fabric` with `Fabric::default()` in a test).
3. Run `cargo clippy` to verify the warning is gone.

This change is straightforward, does not affect existing functionality, and improves code ergonomics.