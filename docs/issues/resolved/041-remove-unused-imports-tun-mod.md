# Remove Unused Imports in src/tun/mod.rs

**Summary**: The file `src/tun/mod.rs` previously contained unused imports (`std::net::Ipv4Addr` and `std::os::unix::io::FromRawFd`). These have been cleaned up.

**Fix Summary**: Updated imports to `use std::net::Ipv4Addr;` and `use std::os::fd::{FromRawFd, IntoRawFd};`, removing the unused lines and aligning with current Rust standards.
