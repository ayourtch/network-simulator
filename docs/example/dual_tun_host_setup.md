# Dual‑TUN Host Setup Example

This guide demonstrates how to run a Linux host inside a network namespace behind the **dual‑TUN** simulator, allowing you to experience a full virtual network.

## Prerequisites
- Linux host with `ip` and `sudo` available.
- The simulator built (`cargo build --release`).
- Root privileges (or appropriate capabilities) to create TUN devices and namespaces.

## Step‑by‑Step

### 1. Create a network namespace for the host
```bash
sudo ip netns add ns_host
```

### 2. Create the two TUN interfaces
The simulator expects two real TUN devices defined in the configuration (`real_tun_a` and `real_tun_b`). Create them:
```bash
# TUN A – will be moved into the namespace
sudo ip tuntap add dev tunA mode tun
# TUN B – stays in the host namespace (used by the user)
sudo ip tuntap add dev tunB mode tun
```

### 3. Bring the interfaces up
```bash
sudo ip link set tunA up
sudo ip link set tunB up
```

### 4. Move `tunA` into the namespace and assign IPs
```bash
sudo ip link set tunA netns ns_host
# Inside the namespace, configure the interface
sudo ip netns exec ns_host ip addr add 10.0.0.2/24 dev tunA
sudo ip netns exec ns_host ip link set tunA up
```

### 5. Configure `tunB` on the host side
```bash
sudo ip addr add 10.0.0.1/24 dev tunB
```

### 6. Prepare a configuration file
Create `dual_tun_config.toml` (or edit your existing one) with the relevant sections:
```toml
[interfaces]
# Real TUN devices used by the simulator
real_tun_a = { name = "tunA", address = "10.0.0.2", netmask = "255.255.255.0" }
real_tun_b = { name = "tunB", address = "10.0.0.1", netmask = "255.255.255.0" }

[tun_ingress]
# Ingress routers – adjust to match your topology
tun_a_ingress = "R1"
tun_b_ingress = "R2"
# Optional IPv6 prefixes for injection direction (default ::/0 matches all IPv6 addresses)
tun_a_ipv6_prefix = "::/0"
tun_b_ipv6_prefix = "::/0"
```
Adjust router IDs (`R1`, `R2`) to the ones defined in your topology.

### 7. Start the simulator
```bash
./target/release/network-simulator --config dual_tun_config.toml -vv
```
You should see log lines indicating the creation of both TUN devices and the start of the dual‑TUN processing loop.

### 8. Verify connectivity from the namespace
In another terminal, run:
```bash
sudo ip netns exec ns_host ping -c 3 10.0.0.1
```
You should receive replies, meaning packets travelled:
`ns_host (tunA) → simulator (ingress A) → fabric → simulator (egress B) → tunB (host)`.

### 9. Clean‑up
After you are done, stop the simulator (Ctrl‑C) and run:
```bash
sudo ip netns delete ns_host
sudo ip link del tunA
sudo ip link del tunB
```
The simulator’s graceful‑shutdown logic will bring the TUN interfaces down automatically, but the explicit deletions ensure a clean environment.

## Notes
- The example assumes IPv4 only; you can use IPv6 addresses by adjusting the `address` and `netmask` (prefix length) fields.
- If you prefer not to run as root, you can give the binary the `CAP_NET_ADMIN` capability instead of using `sudo` for each command.
- The `packet_inject_tun` option is not needed here because the direction is derived from the TUN device the packet arrived on.

Enjoy experimenting with the virtual network!