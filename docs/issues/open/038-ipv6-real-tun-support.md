# Issue 038: IPv6 Real TUN Support Incomplete

## Summary
The simulator can create real TUN devices for IPv4 addresses, but IPv6 address configuration is only partially handled. In `src/tun/mod.rs::create_async_tun` the IPv6 case parses the address but does not set the IPv6 address on the interface. This prevents users from running a Linux host behind the simulator using IPv6.

## Location
- File: `src/tun/mod.rs`
- Function: `create_async_tun`

## Current Behavior
- For IPv6, the builder does not call `.address()` or configure the netmask/prefix.
- The TUN interface is brought up without an IPv6 address, making IPv6 traffic impossible.

## Expected Behavior
- After creating the TUN device, assign the IPv6 address and prefix (netmask) using `ip -6 addr add <addr>/<prefix> dev <name>` or an equivalent `tokio::process::Command`.
- Ensure the interface is up and reachable for IPv6 packets.

## Suggested Implementation
1. Detect IPv6 in `create_async_tun`.
2. After `builder.try_build()`, run a command:
   ```rust
   let prefix = if cfg.netmask.is_empty() { 64 } else { cfg.netmask.parse::<u8>().unwrap() };
   let addr = format!("{}/{}", cfg.address, prefix);
   tokio::process::Command::new("ip")
       .args(["-6", "addr", "add", &addr, "dev", name])
       .status()
       .await?;
   ```
3. Handle errors gracefully, logging a warning if the command fails (e.g., insufficient permissions).
4. Update tests to cover IPv6 real‑TUN creation.

## Low‑Skill Steps for Developer
- Add `use tokio::process::Command;`.
- Insert the command after the TUN builder block.
- Parse the prefix safely, defaulting to 64.
- Log with `warn!` on failure.

## References
- Resolved issue 038‑real‑tun‑not‑implemented.md mentions the stub.
- Plan 3 requires dual‑TUN architecture supporting both IPv4 and IPv6.

---
*Created by automated audit.*
