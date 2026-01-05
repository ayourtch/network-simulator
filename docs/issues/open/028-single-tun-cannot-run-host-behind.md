# Issue 028: Single TUN Device Mode - Cannot Run Linux Host Behind Simulator (Ref: Issue 009)

## Summary
The current real TUN device implementation only supports a single TUN device. The plan describes a two-TUN architecture where packets enter through one TUN (tunA) and exit through another (tunB), enabling a Linux host to communicate through the virtual network. Currently, packets are read from and written back to the SAME TUN device, which doesn't allow for the intended use case.

## Location
- File: `src/tun/mod.rs`
- Function: `start()` (lines 163-221)

## Current Behavior
```rust
// Real TUN mode - only ONE TUN device
let tun_name = &cfg.interfaces.real_tun.name;
let dev = TunDevice::new(&config)?;
let mut async_dev = tokio::fs::File::from_std(std_file);

loop {
    select! {
        read_res = async_dev.read(&mut buf) => {
            // Read packet from TUN
            let processed_packet = process_packet(...).await;
            // Write BACK to the SAME TUN device
            async_dev.write_all(&processed_packet.raw).await?;
        }
    }
}
```

Issues:
1. Only one TUN device is created (`real_tun.name`)
2. Packets are written back to the same device they came from
3. No way to have a "source" TUN and "destination" TUN
4. Cannot place a Linux host behind the simulator

## Expected Behavior (from Plan 3 and Master Plan)
The architecture should support:
1. **TUN A** (e.g., in a network namespace with a Linux host)
2. **TUN B** (e.g., with the user's network)
3. Packets from TUN A should traverse the fabric and exit via TUN B
4. Packets from TUN B should traverse the fabric and exit via TUN A

This enables use cases like:
- Testing network behavior between two simulated endpoints
- Running a Linux application in a namespace that communicates through the virtual fabric
- Simulating WAN links between two local networks

## Recommended Solution

1. Add configuration for two real TUN devices:
```rust
// In src/config.rs
#[derive(Debug, Deserialize, Default)]
pub struct InterfacesConfig {
    #[serde(default)]
    pub tun_a: String,
    #[serde(default)]
    pub tun_b: String,
    #[serde(default)]
    pub real_tun_a: RealTunConfig,  // First TUN device
    #[serde(default)]
    pub real_tun_b: RealTunConfig,  // Second TUN device
}

#[derive(Debug, Deserialize, Default)]
pub struct RealTunConfig {
    pub name: String,
    pub address: String,
    pub netmask: String,
}
```

2. Create both TUN devices in `start()`:
```rust
pub async fn start(cfg: &SimulatorConfig, fabric: &mut Fabric) -> Result<(), Box<dyn std::error::Error>> {
    // ... routing table computation ...
    
    // Create TUN A - use into_raw_fd() or tun crate's async API (see Issue 030)
    let tun_a_name = &cfg.interfaces.real_tun_a.name;
    let tun_a_addr: Ipv4Addr = cfg.interfaces.real_tun_a.address.parse()?;
    let mut config_a = Configuration::default();
    config_a.name(tun_a_name).address(tun_a_addr).up();
    // Option 1: Use tun crate's async device directly
    let mut async_dev_a = tun::create_as_async(&config_a)?;
    
    // Create TUN B
    let tun_b_name = &cfg.interfaces.real_tun_b.name;
    let tun_b_addr: Ipv4Addr = cfg.interfaces.real_tun_b.address.parse()?;
    let mut config_b = Configuration::default();
    config_b.name(tun_b_name).address(tun_b_addr).up();
    let mut async_dev_b = tun::create_as_async(&config_b)?;
    
    let mut buf_a = vec![0u8; cfg.simulation.mtu as usize];
    let mut buf_b = vec![0u8; cfg.simulation.mtu as usize];
    
    loop {
        select! {
            // Read from TUN A, process, write to TUN B
            read_res = async_dev_a.read(&mut buf_a) => {
                let n = read_res?;
                let packet = parse(&buf_a[..n])?;
                let processed = process_packet(
                    fabric, &routing_tables, ingress_a.clone(), 
                    packet, Destination::TunB
                ).await;
                async_dev_b.write_all(&processed.raw).await?;  // Write to TUN B
            }
            
            // Read from TUN B, process, write to TUN A
            read_res = async_dev_b.read(&mut buf_b) => {
                let n = read_res?;
                let packet = parse(&buf_b[..n])?;
                let processed = process_packet(
                    fabric, &routing_tables, ingress_b.clone(),
                    packet, Destination::TunA
                ).await;
                async_dev_a.write_all(&processed.raw).await?;  // Write to TUN A
            }
            
            _ = signal::ctrl_c() => break,
        }
    }
}
```

3. Update documentation with namespace setup example:
```markdown
## Running with Linux Namespace

To test with a Linux host behind the simulator:

```bash
# Create namespace for TUN A
sudo ip netns add ns_a

# Start simulator (creates tun_a and tun_b)
sudo ./network-simulator --config config.toml

# Move tun_a into namespace
sudo ip link set tun_a netns ns_a

# Configure TUN A in namespace
sudo ip netns exec ns_a ip addr add 10.0.0.1/24 dev tun_a
sudo ip netns exec ns_a ip link set tun_a up

# Configure TUN B on host
sudo ip addr add 10.0.1.1/24 dev tun_b
sudo ip link set tun_b up

# Test connectivity
sudo ip netns exec ns_a ping 10.0.1.1
```
```

4. Add tests for dual-TUN mode.

## Files to Modify
- `src/config.rs` (add dual TUN configuration)
- `src/tun/mod.rs` (implement dual TUN handling)
- `docs/` (add namespace setup documentation)
- `tests/` (add dual TUN tests)

## Effort Estimate
Large (4-6 hours)

## References
- Original Issue 009: docs/issues/resolved/009-tun-write-back-not-implemented.md

## Related Plans
- Plan 3: TUN Interface Management
- Plan 9: Integration and End-to-End Testing
