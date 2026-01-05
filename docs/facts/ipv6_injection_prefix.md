# IPv6 Injection Prefix Fact

The simulator now respects configurable IPv6 prefixes (`tun_a_ipv6_prefix` and `tun_b_ipv6_prefix`) when determining packet injection direction for mock packets. This extends the earlier IPv4‑only prefix handling, allowing correct routing of IPv6 test traffic.
