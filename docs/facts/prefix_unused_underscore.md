# Unused Prefix Variable Fact

In `src/tun/mod.rs` the previously unused variables `prefix` in IPv4 and IPv6 handling blocks were renamed to `_prefix` (prefixed with an underscore). This silences compiler warnings about unused variables without altering logic, keeping the code clean.
