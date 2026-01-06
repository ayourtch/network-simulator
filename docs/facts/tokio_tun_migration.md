# Tokio‑tun Migration Fact

The simulator now uses the `tokio-tun` crate for asynchronous TUN handling, replacing the legacy `tun` crate. This eliminates raw file descriptor hacks, provides async read/write halves, and simplifies IPv4/IPv6 configuration.
