# RoutingTable Derive Default Fact

The `RoutingTable` struct in `src/routing/mod.rs` now derives `Default` (`#[derive(..., Default)]`). The manual `impl Default` block was removed, simplifying the code and silencing the Clippy warning about a derivable implementation.
