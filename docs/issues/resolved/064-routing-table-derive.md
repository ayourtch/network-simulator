# Issue 064: RoutingTable should derive Default

## Summary
`src/routing/mod.rs` defines `RoutingTable` with a manual `impl Default` even though all fields implement `Default`. Clippy warns that this can be derived, adding unnecessary code.

## Suggested Solution
1. Add `#[derive(Default)]` to the `RoutingTable` struct definition.
2. Remove the manual `impl Default for RoutingTable` block.
3. Run `cargo clippy` to verify the warning is gone.

This simplifies the code and follows idiomatic Rust practices.
