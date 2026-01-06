# Issue 038: IPv6 Real TUN Support Synchronous Command (Resolved)

The synchronous `std::process::Command` used for IPv6 address configuration was replaced with `tokio::process::Command` and awaited, fixing the blocking behavior.
