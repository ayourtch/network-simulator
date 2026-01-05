# Issue 010: Virtual Customer Not Used

**Summary**
The configuration includes a `virtual_customer` section that is never referenced by the simulator. This prevents simulation of end‑user traffic or generation of synthetic packets based on the virtual customer definition.

**Location**
- Configuration struct in `src/config.rs` defines `virtual_customer` (currently not present, but the documentation mentions it).
- No code reads or utilizes this field.

**Current Behavior**
- The `virtual_customer` configuration is ignored; the simulator only processes packets from mock files or real TUN interfaces.

**Expected Behavior**
- When a `virtual_customer` is defined, the simulator should generate packets according to its specification (e.g., source/destination IPs, traffic patterns) and inject them into the fabric.

**Suggested Solution**
1. Add a `VirtualCustomerConfig` struct to `src/config.rs` with fields such as `src_ip`, `dst_ip`, `protocol`, `size`, `rate`.
2. Extend `SimulatorConfig` to include an optional `virtual_customer: Option<VirtualCustomerConfig>`.
3. In `src/tun/mod.rs` (or a new module), implement a task that periodically creates `PacketMeta` instances based on the virtual customer settings and sends them through the processing pipeline.
4. Ensure proper statistics are updated for generated traffic.
5. Add unit tests verifying that packets are generated and processed.
