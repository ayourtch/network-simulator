# Issue 028: Single TUN Device Mode - Cannot Run Linux Host Behind Simulator (Ref: Issue 009)

## Summary
The current real TUN device implementation only supports a single TUN device. The plan describes a two‑TUN architecture where packets enter through one TUN (tunA) and exit through another (tunB), enabling a Linux host to communicate through the virtual network. Currently, packets are read from and written back to the SAME TUN device, which doesn't allow the intended use case.

## Location
- File: `src/tun/mod.rs`
- Function: `start()` (lines 163-221)

## Current Behavior
```rust
// Real TUN mode - only ONE TUN device
let tun_name = &cfg.interfaces.real_tun.name;
let dev = TunDevice::new(&config)?;
let mut async_dev = tokio::fs::File::from_std(std_file);

loop {
    select! {
        read_res = async_dev.read(&mut buf) => {
            // Read packet from TUN
            let processed_packet = process_packet(...).await;
            // Write BACK to the SAME TUN device
            async_dev.write_all(&processed_packet.raw).await?;
        }
    }
}
```

Issues:
1. Only one TUN device is created (`real_tun.name`)
2. Packets are written back to the same device they came from
3. No way to have a "source" TUN and "destination" TUN
4. Cannot place a Linux host behind the simulator

## Expected Behavior (from Plan 3 and Master Plan)
The architecture should support:
1. **TUN A** (e.g., in a network namespace with a Linux host)
2. **TUN B** (e.g., with the user's network)
3. Packets from TUN A should traverse the fabric and exit via TUN B
4. Packets from TUN B should traverse the fabric and exit via TUN A

This enables use cases like:
- Testing network behavior between two simulated endpoints
- Running a Linux application in a namespace that communicates through the virtual fabric
- Simulating WAN links between two local networks

## Fix summary
Implemented dual real‑TUN support:
- Added `real_tun_a` and `real_tun_b` configuration (already present).
- Refactored `src/tun/mod.rs::start()` to create two async TUN devices, read from each, process packets, and write to the opposite device.
- Updated shutdown handling and buffer management.
- Added documentation fact `docs/facts/dual_tun.md`.
- Adjusted imports and removed unused code paths.

The simulator now correctly forwards packets between two distinct TUN interfaces, allowing a Linux host to run behind the simulator.
