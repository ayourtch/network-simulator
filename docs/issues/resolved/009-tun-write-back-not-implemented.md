# Issue 009: TUN Write-Back Not Implemented

## Summary
After a packet traverses the fabric and reaches the destination ingress router, it should be written back to the appropriate TUN interface. Currently, packets are processed but never delivered to the destination TUN.

## Location
- File: `src/tun/mod.rs`
- Function: `start()`

## Current Behavior
The TUN module:
1. Reads packets from mock file or real TUN device
2. Calls `process_packet()` or `process_packet_multi()`
3. Processing ends - no delivery to destination TUN

## Expected Behavior (from Plan 3 and Plan 9)
1. After a packet traverses the fabric:
   - If from TUN A, deliver to TUN B
   - If from TUN B, deliver to TUN A
2. Use `AsyncWriteExt` to write the packet bytes to the TUN device
3. Handle write errors appropriately

## Recommended Solution

1. Modify `process_packet()` to return the processed packet:
```rust
pub async fn process_packet(...) -> Option<Vec<u8>> {
    // ... processing ...
    // Return the packet bytes if delivery should occur
    Some(processed_bytes)
}
```

2. Add write functionality to TUN handling:
```rust
use tokio::io::AsyncWriteExt;

// After processing a packet from TUN A
if let Some(packet_bytes) = process_packet(...).await {
    if let Err(e) = tun_b_device.write_all(&packet_bytes).await {
        error!("Failed to write to TUN B: {}", e);
    }
}
```

3. This requires maintaining handles to both TUN devices for reading and writing.

4. For the mock packet file mode, output can be logged instead of written.

## Files to Modify
- `src/processor.rs` (return packet data)
- `src/tun/mod.rs` (add write functionality)
- `tests/` (add TUN write tests)

## Effort Estimate
Medium (3-4 hours)

## Dependencies
- Issue 008: Hop-by-hop forwarding (needs to complete traversal first)

## Related Plans
- Plan 3: TUN Interface Management
- Plan 9: Integration and End-to-End Testing
