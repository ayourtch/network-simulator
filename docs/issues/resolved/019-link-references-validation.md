# Resolved: Link References Validation – issue closed

The configuration validation now checks that all link references point to existing routers. Invalid references cause `SimulatorConfig::validate` to return an error, preventing the simulator from starting with inconsistent topology.

All related tests have been added (`tests/config_validation_test.rs`) and pass.
