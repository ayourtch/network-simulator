# Multipath Loop Detection Fact

The multipath packet processor now checks if the selected next‑hop router equals the ingress router. If they match, processing stops, preventing infinite routing loops and ensuring correct termination of packets that have reached their egress destination.
