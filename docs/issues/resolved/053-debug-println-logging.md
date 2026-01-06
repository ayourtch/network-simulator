# Issue 053: Debug Print Statements Should Use Logging

## Summary
`src/tun/mod.rs` contains `println!("Loop");` and `println!("Exiting");` statements used for debugging. The project uses the `tracing` crate for structured logging throughout the codebase. Direct `println!` calls bypass the logging configuration, making it harder to control output verbosity and format.

## Expected Behavior
- Replace `println!("Loop");` with a `debug!` or `info!` log call (e.g., `debug!("Entering dual‑TUN processing loop");`).
- Replace `println!("Exiting");` with an appropriate log call (e.g., `info!("TUN handling loop exited");`).
- Ensure all user‑visible messages are emitted via the `tracing` macros, respecting the configured log level.

## Suggested Solution
1. Import `tracing::{debug, info}` at the top of `src/tun/mod.rs` if not already imported (it already is).
2. Change the two `println!` statements to `debug!` or `info!` as appropriate.
3. Run `cargo clippy` and the test suite to confirm no warnings remain.

## Acceptance Criteria
- No `println!` macros remain in `src/tun/mod.rs`.
- The log output appears at the correct log level when running the simulator with `RUST_LOG=debug`.
- All tests continue to pass.
