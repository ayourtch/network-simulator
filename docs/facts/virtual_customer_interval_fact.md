# Virtual Customer Interval Fact

The simulator defines a `vc_interval` variable for periodic virtual‑customer packet generation. It is now hooked into the main event loop in `src/tun/mod.rs`, ticking at the configured rate and invoking `generate_virtual_packet`. This eliminates previous unused‑variable warnings and enables steady traffic generation without an initial burst.
