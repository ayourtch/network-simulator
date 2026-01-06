# Issue 009: Mock TUN Write‑Back Implemented

## Summary
The mock TUN handling now writes the processed packet bytes as hex strings to a corresponding `*_out.txt` file, simulating the write‑back behavior of a real TUN device.

## Location
- File: `src/tun/mod.rs`
- Logic: After processing each mock packet, the packet's raw bytes are hex‑encoded and appended to an output file named `<input_path>_out.txt`.

## Resolution
- Added output file creation and writing logic.
- Updated documentation fact `docs/facts/tun_write_back_fact.md`.
- Updated related tests to verify write‑back behavior.

The issue is now resolved.
