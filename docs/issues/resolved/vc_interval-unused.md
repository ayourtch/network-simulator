# Issue: Unused `vc_interval` variable in TUN module

The `vc_interval` variable was previously declared but never used, causing a warning. The interval has now been hooked into the select! loop, ticking at the configured rate and invoking `generate_virtual_packet`. This resolves the warning and provides periodic virtual‑customer traffic.
