# Load Balancing Fact

The packet processor now uses `select_egress_link` which incorporates per‑link counters into the hash calculation. This enables true per‑packet load‑balancing across links marked with `load_balance: true`.
