

**Summary**
The virtual customer configuration `rate` field now drives periodic packet generation. The simulator emits packets at the configured packets‑per‑second using a `tokio::time::Interval` within the main event loop.

**Resolution**
- Updated `src/tun/mod.rs` to use a single periodic generation arm in the `select!` loop, removing duplicate code.
- Added initialization of `vc_interval` and tick handling to generate packets continuously.
- Updated documentation and facts accordingly.

**Verification**
- All tests pass, and periodic packet generation is exercised in integration tests.

*Closed as implemented.*
