# Resolved: Error Writing to TUN B – Unseekable File

**Resolution**
- Updated `src/tun/mod.rs` to detect the `seek on unseekable file` error when writing to TUN B. The code now logs a warning and continues instead of treating it as a fatal error.
- This prevents spurious error messages during mock runs or when the TUN device is not fully configured.

*Closed as implemented.*