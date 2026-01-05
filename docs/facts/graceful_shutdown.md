# Graceful Shutdown Fact

When the simulator receives a termination signal (Ctrl‑C), it exits the dual‑TUN processing loop, brings both real TUN interfaces down using `ip link set dev <name> down` (Linux), and cleanly releases resources. This prevents orphaned TUN devices after the program ends.