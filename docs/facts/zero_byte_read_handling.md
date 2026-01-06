# Zero-byte Read Handling Fact

When reading from a real TUN interface, a zero‑byte read does **not** indicate EOF. The simulator now treats such reads as non‑fatal, logs a debug message and continues the loop, preventing premature termination of the simulation.
