# Resolved: Link Counter Not Used in Hash – issue closed

Implemented true load‑balancing by incorporating per‑link counters into the hash calculation in `select_egress_link` (and multipath processing). This ensures traffic is distributed based on link utilization.

*Closed as implemented.*