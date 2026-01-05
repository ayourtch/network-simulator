# Real TUN Address Validation Fact

The simulator now validates both IPv4 and IPv6 address strings (and appropriate netmask or prefix) supplied for `real_tun_a` and `real_tun_b` in the configuration. Invalid values cause `SimulatorConfig::validate()` to return a clear error instead of panicking at runtime.
