# Virtual Customer Rate and Size Fact

- The `VirtualCustomerConfig` now supports a `size` field which extends the generated packet payload with zero‑filled bytes.
- The `rate` field specifies the number of packets to generate per second. The simulator now emits packets periodically at this rate after the initial burst, using a `tokio::time::Interval`.