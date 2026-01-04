# Issue 020: Raw Packet Bytes Not Preserved Through Processing

## Summary
The packet processing pipeline converts raw packet bytes into `PacketMeta` and discards the original bytes. When packets need to be written back to TUN, only metadata is available, not the original packet.

## Location
- Files: `src/packet/mod.rs`, `src/processor.rs`, `src/tun/mod.rs`

## Current Behavior
1. Raw bytes are parsed into `PacketMeta` struct
2. Only `PacketMeta` is passed through processing
3. Original raw bytes are lost
4. Cannot reconstruct the packet for TUN write-back

## Expected Behavior (from Plan 3, Plan 4)
The system should be able to:
1. Preserve original packet bytes through processing
2. Modify TTL/hop-limit in the packet bytes (not just metadata)
3. Recalculate checksums after modification
4. Write the modified packet bytes to the destination TUN

## Recommended Solution

### Option A: Carry raw bytes alongside metadata
Create a combined structure:
```rust
pub struct ProcessingPacket {
    pub meta: PacketMeta,
    pub raw: Vec<u8>,
}
```

### Option B: Modify bytes in-place
Process raw bytes directly without separate metadata:
```rust
pub fn process_in_place(raw: &mut [u8]) -> Result<(), Error> {
    // Parse, validate, decrement TTL, recalculate checksum in-place
}
```

### Recommended Approach
Option A is cleaner and allows metadata-based decisions without re-parsing:

1. Update `parse()` to return both:
```rust
pub fn parse(data: &[u8]) -> Result<ProcessingPacket, &'static str> {
    let meta = parse_meta(data)?;
    Ok(ProcessingPacket {
        meta,
        raw: data.to_vec(),
    })
}
```

2. Update processor to carry `ProcessingPacket`.

3. Add function to apply TTL decrement to raw bytes:
```rust
pub fn apply_ttl_decrement(packet: &mut ProcessingPacket) -> Result<(), Error> {
    // Decrement TTL in raw bytes
    // Recalculate checksum
    // Update meta.ttl
}
```

4. Return raw bytes for TUN write-back.

## Files to Modify
- `src/packet/mod.rs`
- `src/processor.rs`
- `src/tun/mod.rs`
- `tests/`

## Effort Estimate
Medium (3-4 hours)

## Dependencies
- Issue 009: TUN Write-Back (needs raw bytes to write)

## Related Plans
- Plan 3: TUN Interface Management
- Plan 4: Packet Processing and Forwarding Engine
