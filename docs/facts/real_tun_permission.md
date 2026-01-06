# Real TUN Permission Fact

When the simulator tries to open real TUN devices, it normally requires elevated privileges (CAP_NET_ADMIN) or root. If these permissions are missing, the attempt fails with `EPERM: Operation not permitted`. The recent update adds a fallback: the code now detects this error, logs a warning, and skips real‑TUN handling, allowing the simulator to continue in mock mode. This behavior improves usability for non‑privileged users while still documenting the permission requirement for real‑TUN operation.

**Note:** In environments without CAP_NET_ADMIN, the simulator automatically switches to mock TUN mode, enabling simulations to run without needing root privileges.

