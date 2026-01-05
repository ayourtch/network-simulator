# Router Statistics Fact

The packet processor now updates router statistics on every packet event:
- `increment_received` on each ingress.
- `increment_forwarded` after successful forwarding.
- `increment_icmp` when an ICMP error is generated.
- `increment_lost` when a packet is dropped due to simulated loss.

These counters are accessible via `Fabric::get_statistics()` for monitoring and debugging.
