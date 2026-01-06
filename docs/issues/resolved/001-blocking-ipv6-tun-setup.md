# Issue 001: Blocking IPv6 TUN Setup Command (Resolved)

The issue was resolved by replacing the synchronous `std::process::Command` with the asynchronous `tokio::process::Command` in `src/tun/mod.rs::create_async_tun`. This prevents blocking the async runtime during IPv6 address configuration.
