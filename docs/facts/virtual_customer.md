# Virtual Customer Feature

- Added `VirtualCustomerConfig` to `src/config.rs` with fields `src_ip`, `dst_ip`, `protocol`, `size`, `rate`.
- Implemented packet generation in `src/tun/mod.rs` using these settings, creating a basic IPv4 packet, calculating checksum, and injecting via appropriate ingress based on CIDR prefixes or explicit `packet_inject_tun`.
- Updated imports to include `parse`.
- Added helper `ip_in_prefix` closure for CIDR detection.
- Ensures generated packets are processed through the same routing/multipath pipelines as mock packets.
