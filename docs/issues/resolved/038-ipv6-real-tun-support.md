# Issue 038: IPv6 Real TUN Support Incomplete (Resolved)

IPv6 address configuration is now performed after TUN device creation using an async `ip -6 addr add` command, completing IPv6 support for real TUN devices.
