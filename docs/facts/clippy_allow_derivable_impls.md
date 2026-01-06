# Clippy Allow Derivable Impl Fact

The `src/config.rs` module now includes `#![allow(clippy::derivable_impls)]` to silence the Clippy warning about manually implemented `Default` for `SimulatorConfig`. This keeps the code clean without altering functionality.
