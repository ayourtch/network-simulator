# Multipath Destination Detection Fact

The multipath packet processor (`process_packet_multi`) now checks whether the selected next‑hop router equals the current ingress router. If they match, processing stops, preventing infinite loops and correctly handling packets that have reached their egress destination.
