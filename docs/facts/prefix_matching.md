# Prefix Matching Fact

The simulator now uses CIDR‑based prefix matching for both IPv4 and IPv6 injection direction logic, replacing fragile string `starts_with` checks. This ensures robust handling of any valid CIDR prefix defined in the configuration.