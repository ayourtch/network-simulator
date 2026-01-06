# IPv6 TUN Async Command Fact

The simulator now uses `tokio::process::Command` to configure IPv6 addresses on real TUN interfaces, avoiding blocking the async runtime.
