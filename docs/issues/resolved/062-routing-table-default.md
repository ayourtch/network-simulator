# Issue 062: RoutingTable lacks derived Default implementation

## Summary
The `RoutingTable` struct in `src/routing/mod.rs` manually implements `Default` even though all its fields already implement `Default`. Clippy warns that this can be derived, which adds unnecessary boilerplate.

## Suggested Solution
1. Add `#[derive(Default)]` to the `RoutingTable` definition.
2. Remove the manual `impl Default for RoutingTable` block.
3. Run `cargo clippy` to ensure the warning is gone.

Implementing this makes the code cleaner and aligns with idiomatic Rust practices.
