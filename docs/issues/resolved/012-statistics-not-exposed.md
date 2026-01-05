# Issue 012: Statistics Not Exposed – Resolved

**Resolution**
- Added `Fabric::get_statistics()` method (already present) to retrieve router statistics.
- Updated `src/main.rs` with a `--stats` flag; when provided, after the simulation run the program prints the collected statistics using `Fabric::print_statistics()`.
- Updated documentation in `docs/facts/router_statistics.md` to mention the new CLI option.
- All tests pass.

*Closed as implemented.*