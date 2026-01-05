# Virtual Customer IPv6 Support Fact

- The virtual customer packet generator now supports IPv6 addresses. When `src_ip` and `dst_ip` are valid IPv6 strings, a 40‑byte IPv6 header is constructed, payload length is set, and the packet is injected using the same routing logic.
- This enables end‑to‑end simulations involving IPv6 traffic from synthetic customers.