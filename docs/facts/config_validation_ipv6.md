# IPv6 Netmask/Prefix Validation Fact

`SimulatorConfig::validate()` now verifies that IPv6 netmask fields for `real_tun_a` and `real_tun_b` are valid prefix lengths (0‑128). If omitted, the default `/64` is applied. Invalid values produce a clear configuration error, preventing runtime panics.
