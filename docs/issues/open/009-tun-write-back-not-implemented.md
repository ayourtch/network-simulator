# Issue 009: TUN Write-Back Not Implemented

## Summary
Packets processed by the simulator are never written back to the destination TUN interface. After `process_packet` (or `process_packet_multi`) completes, the resulting packet bytes are discarded, so the simulated host never receives the packet.

## Location
- File: `src/tun/mod.rs`
- Function: `start()` – the main loop that reads packets from a file or virtual‑customer generator and forwards them through the fabric.

## Expected Behavior
1. After a packet reaches its destination router (i.e., the egress router is the opposite TUN), the simulator should write the packet bytes to the opposite TUN device using `AsyncWriteExt::write_all`.
2. Errors while writing should be logged but must not crash the simulator.
3. In mock‑file mode, the packet should be written to the appropriate output file (or logged) to allow verification.

## Recommended Solution (low‑skill developer)
1. Modify `process_packet` and `process_packet_multi` to return the final `PacketMeta` (already does) and expose its raw bytes.
2. In `src/tun/mod.rs::start()`, after calling the processor, inspect the returned `PacketMeta`. Determine the opposite TUN (if ingress was `ingress_a` then write to `tun_b`, and vice‑versa).
3. Keep handles to both TUN devices (or mock files) open for writing. Example snippet:
```rust
let result = if cfg.enable_multipath {
    process_packet_multi(...).await
} else {
    process_packet(...).await
};
// `result` is the final PacketMeta
let target = if ingress == ingress_a { &mut tun_b } else { &mut tun_a };
if let Err(e) = target.write_all(&result.raw).await {
    error!("Failed to write packet to TUN: {}", e);
}
```
4. For mock mode, replace the write with `writeln!(mock_output, "{:x?}", result.raw)`.

## Files to Modify
- `src/tun/mod.rs` (add write‑back logic after processing)
- Possibly adjust `src/processor.rs` to ensure the final packet is returned unchanged.
- Add a new test in `tests/tun_write_back_test.rs` that verifies a packet read from the mock file appears in the opposite mock output.

*This issue is opened because the original resolved issue 009 states the feature is still missing.*