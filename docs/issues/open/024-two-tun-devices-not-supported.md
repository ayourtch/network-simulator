# Issue 024: Two TUN Devices Not Fully Supported

## Summary
The plan specifies two separate TUN devices (tunA and tunB), but the current implementation only opens a single real TUN device.

## Location
- File: `src/tun/mod.rs`
- Function: `start()`

## Current Behavior
```rust
// Only opens one TUN device
let tun_name = &cfg.interfaces.real_tun.name;
let dev = TunDevice::new(&config)?;
```

Only `real_tun.name` is opened. The `tun_a` and `tun_b` from config are not used for real TUN handling.

## Expected Behavior (from Plan 3)
From the plan:
> Create TUN interface for tunA and tunB
> TunManager manages both tunA and tunB interfaces

The implementation should:
1. Create/open two TUN devices
2. Read packets from both
3. Write packets to the appropriate destination TUN

## Recommended Solution

1. Update TUN handling to open two devices:
```rust
pub async fn start(cfg: &SimulatorConfig, fabric: &mut Fabric) -> Result<(), Box<dyn std::error::Error>> {
    // Open both TUN devices
    let mut config_a = Configuration::default();
    config_a.name(&cfg.interfaces.tun_a).up();
    let dev_a = TunDevice::new(&config_a)?;
    
    let mut config_b = Configuration::default();
    config_b.name(&cfg.interfaces.tun_b).up();
    let dev_b = TunDevice::new(&config_b)?;
    
    // Convert to async
    let mut async_a = tokio::fs::File::from_std(unsafe { std::fs::File::from_raw_fd(dev_a.as_raw_fd()) });
    let mut async_b = tokio::fs::File::from_std(unsafe { std::fs::File::from_raw_fd(dev_b.as_raw_fd()) });
    
    // Use select! to read from both
    loop {
        tokio::select! {
            result = async_a.read(&mut buf_a) => {
                // Process packet from TUN A, deliver to TUN B
            }
            result = async_b.read(&mut buf_b) => {
                // Process packet from TUN B, deliver to TUN A
            }
        }
    }
}
```

2. Update config structure to include separate addresses/netmasks for each TUN.

3. Add tests for dual-TUN handling.

## Files to Modify
- `src/tun/mod.rs`
- `src/config.rs` (add tun_a/tun_b addresses)
- `tests/` (add dual-TUN tests)

## Effort Estimate
Medium (3-4 hours)

## Dependencies
- Issue 009: TUN Write-Back

## Related Plans
- Plan 3: TUN Interface Management
