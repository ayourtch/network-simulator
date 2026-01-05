# Issue 030: TUN Device Memory Safety Issue with from_raw_fd

## Summary
The current TUN device handling uses `unsafe { std::fs::File::from_raw_fd(...) }` which creates a potential memory safety issue. The `TunDevice` owns the file descriptor, and creating a `File` from the same raw fd results in double-ownership, which can lead to use-after-free or double-close bugs.

## Location
- File: `src/tun/mod.rs`
- Function: `start()` (lines 171-174)

## Current Behavior
```rust
let dev = TunDevice::new(&config)
    .map_err(|e| format!("Failed to create TUN device: {}", e))?;
let std_file = unsafe { std::fs::File::from_raw_fd(dev.as_raw_fd()) };
let mut async_dev = tokio::fs::File::from_std(std_file);
```

Issues:
1. `TunDevice` owns the file descriptor
2. `File::from_raw_fd()` takes ownership of the fd
3. Both `dev` and `std_file` now think they own the fd
4. When `dev` goes out of scope (if it does), it may close the fd
5. Then `std_file`/`async_dev` would be operating on a closed fd

## Expected Behavior
Proper ownership transfer or use of references:
1. Either transfer ownership explicitly (consuming `TunDevice`)
2. Or use the TUN device's async API directly
3. Or use `into_raw_fd()` to take ownership away from `TunDevice`

## Recommended Solution

1. Use `IntoRawFd` to properly transfer ownership:
```rust
use std::os::unix::io::IntoRawFd;

let dev = TunDevice::new(&config)
    .map_err(|e| format!("Failed to create TUN device: {}", e))?;

// Transfer ownership of the fd away from TunDevice
let raw_fd = dev.into_raw_fd();  // TunDevice no longer owns the fd

// Now safely create File from the fd
let std_file = unsafe { std::fs::File::from_raw_fd(raw_fd) };
let mut async_dev = tokio::fs::File::from_std(std_file);
```

Note: This requires `TunDevice` to implement `IntoRawFd`. If it doesn't, we need to use a different approach.

2. Alternative: Use the `tun` crate's async device directly:
```rust
use tun::AsyncDevice;

let dev = tun::create_as_async(&config)
    .map_err(|e| format!("Failed to create async TUN device: {}", e))?;

// Use dev.read() and dev.write() directly
// No need for raw fd manipulation
```

3. **Note**: The ManuallyDrop approach creates a memory leak and should NOT be used:
```rust
// DON'T DO THIS - Memory leak!
// ManuallyDrop prevents the TunDevice from being cleaned up
// This is only documented here to show what NOT to do
```

4. Preferred solution - use tun crate's async API if available:
```rust
// Check if tun crate provides async support
// The tun crate has `features = ["async"]` which provides AsyncDevice

use tun::create_as_async;

let mut dev = create_as_async(&config)?;

loop {
    select! {
        n = dev.read(&mut buf) => {
            // Handle read
        }
        _ = signal::ctrl_c() => break,
    }
}
```

## Files to Modify
- `src/tun/mod.rs` (fix ownership handling)

## Effort Estimate
Small (1-2 hours)

## Security Implications
- Use-after-free vulnerability if TunDevice is dropped while File is still in use
- Potential undefined behavior from operating on closed file descriptor

## Related Plans
- Plan 3: TUN Interface Management
