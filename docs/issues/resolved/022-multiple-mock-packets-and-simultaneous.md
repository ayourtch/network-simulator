# Fixed Multiple Mock Packets and Simultaneous Injection

## Summary
Implemented support for injecting multiple mock packet files into the topology simultaneously and capturing the packets exiting each mock TUN. Added configuration fields `packet_files` and `packet_inject_tuns` to specify per‑file injection directions. Updated `src/tun/mod.rs` to process each file independently, write processed packets to `<packet_file>_out.txt`, and respect the injection direction.

## Changes
- Extended `SimulatorConfig` with `packet_files: Option<Vec<String>>` and `packet_inject_tuns: Option<Vec<String>>` (already present).
- Implemented loop in `src/tun/mod.rs` handling multiple packet files, determining injection direction per file, and writing output files.
- Updated CLI (`src/main.rs`) to accept `--packet-files` for multiple files.
- Added comprehensive test `tests/tun_multiple_mock_test.rs` verifying output files and link counters.
- Updated documentation and README entries.

## Validation
All tests pass (`cargo test`). The mock TUN processing now correctly handles multiple packet files, injects into specified TUNs, and captures output per file.
