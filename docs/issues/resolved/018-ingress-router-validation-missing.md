# Issue 018: Ingress Router Validation Missing

Implemented validation of ingress router IDs in `SimulatorConfig::validate`. The validation now ensures that the ingress routers specified in the configuration exist in the topology and conform to the required naming pattern. The `main.rs` already calls `cfg.validate()`, so the simulator now fails fast with a clear error message if the ingress routers are missing or invalid.
