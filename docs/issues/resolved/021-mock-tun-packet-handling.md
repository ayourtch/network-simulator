# Fixed Mock TUN Packet Handling

## Summary
Implemented output capture for mock TUN packet processing and added configuration to specify injection direction.

## Changes
- Added `packet_inject_tun` field to `SimulatorConfig`.
- Updated `src/tun/mod.rs` to write processed packet raw bytes as hex to `<packet_file>_out.txt`.
- Added logic to respect `packet_inject_tun` for injection direction, falling back to IP‑based detection.
- Updated imports for file handling.
- Updated all tests to include the new config field.
- Adjusted documentation and comments.

## Validation
All tests now pass (`cargo test`) and mock TUN processing writes output files as expected.
