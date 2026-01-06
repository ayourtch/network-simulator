# Graceful Shutdown Fact

The simulator’s dual‑TUN loop now listens for `Ctrl‑C` (SIGINT) via `tokio::signal::ctrl_c()`. Upon receiving the shutdown signal, it:
1. Logs the shutdown event.
2. Executes `ip link set dev <tun> down` for both real TUN interfaces to cleanly bring them down.
3. Exits the main processing loop, allowing the program to terminate gracefully without leaving stray TUN devices.

This ensures the host environment remains clean after the simulator exits.