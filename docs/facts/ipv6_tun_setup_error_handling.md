# IPv6 TUN Setup Error Handling Fact

The dual‑TUN initialization now verifies the success of the `ip -6 addr add` command used to configure IPv6 prefixes. If the command fails or returns a non‑zero exit status, the simulator reports a clear error instead of silently ignoring the failure. This improves reliability when setting up IPv6 TUN interfaces.
