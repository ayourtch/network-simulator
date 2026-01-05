# Dual TUN Shutdown Fact

When the simulator receives a termination signal (Ctrl‑C), the dual‑TUN processing loop exits, and the Linux `ip` command is used to bring both real TUN interfaces (`real_tun_a` and `real_tun_b`) down, ensuring no stray TUN devices remain after the program ends.