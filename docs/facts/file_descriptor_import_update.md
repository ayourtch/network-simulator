# File Descriptor Import Update Fact

The `src/tun/mod.rs` file now imports `FromRawFd` and `IntoRawFd` from the modern `std::os::fd` module instead of the older `std::os::unix::io` path. This aligns the code with the current Rust standard library layout and resolves compilation errors related to missing import paths.
