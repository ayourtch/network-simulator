# TUN Permission Fallback Fact

`src/tun/mod.rs` now detects `EPERM`/`Operation not permitted` when opening real TUN devices without CAP_NET_ADMIN. It logs a warning and returns early, allowing the simulator to run in mock mode. This makes the tool usable for developers without root privileges.
