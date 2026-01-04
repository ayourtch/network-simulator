# Issue 027: No Graceful Shutdown Handler

## Summary
The simulator lacks graceful shutdown handling. When the process is terminated, there's no cleanup of TUN devices or final statistics output.

## Location
- File: `src/tun/mod.rs`
- Function: `start()`

## Current Behavior
```rust
loop {
    let n = match async_dev.read(&mut buf).await {
        Ok(0) => break, // EOF only
        // ...
    };
}
```

The loop only breaks on EOF or error. There's no signal handling for SIGTERM/SIGINT.

## Expected Behavior (from Plan 9)
From the plan's main event loop example:
```rust
tokio::select! {
    // ... packet handling ...
    _ = tokio::signal::ctrl_c() => {
        println!("\nShutting down...");
        break;
    }
}
```

## Recommended Solution

1. Add signal handling to the TUN loop:
```rust
use tokio::signal;

loop {
    tokio::select! {
        result = async_dev.read(&mut buf) => {
            // ... handle packet ...
        }
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal");
            break;
        }
    }
}

// Cleanup
info!("Cleaning up...");
fabric.print_statistics(); // Issue 012
// TUN devices are cleaned up when dropped
info!("Shutdown complete");
```

2. Consider adding SIGTERM handling for container environments:
```rust
use tokio::signal::unix::{signal, SignalKind};

let mut sigterm = signal(SignalKind::terminate())?;

tokio::select! {
    // ...
    _ = sigterm.recv() => {
        info!("Received SIGTERM");
        break;
    }
}
```

## Files to Modify
- `src/tun/mod.rs`
- `src/lib.rs` (if run() needs cleanup)

## Effort Estimate
Small (1-2 hours)

## Related Plans
- Plan 9: Integration and End-to-End Testing
