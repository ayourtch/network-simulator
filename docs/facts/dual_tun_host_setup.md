# Dual TUN Host Setup Guide

This guide shows how to run the network simulator with two **real** TUN interfaces attached to separate Linux network namespaces. It enables a host in each namespace to communicate through the simulated network topology.

## Prerequisites
- Linux with `ip` (iproute2) installed.
- `sudo` privileges.
- The `network-simulator` binary built (`cargo build --release`).
- The `tun` kernel module loaded (`modprobe tun`).

## Step‑by‑Step
1. **Create two network namespaces**
   ```bash
   sudo ip netns add ns1
   sudo ip netns add ns2
   ```
2. **Create the TUN devices** (they will appear as `tun0a` and `tun0b`).
   ```bash
   # Create TUN A and move it to ns1
   sudo ip tuntap add dev tun0a mode tun
   sudo ip link set tun0a netns ns1

   # Create TUN B and move it to ns2
   sudo ip tuntap add dev tun0b mode tun
   sudo ip link set tun0b netns ns2
   ```
3. **Configure IP addresses inside each namespace**
   ```bash
   # In ns1 (ingress A)
   sudo ip netns exec ns1 ip addr add 10.0.0.1/24 dev tun0a
   sudo ip netns exec ns1 ip link set tun0a up
   sudo ip netns exec ns1 ip route add default dev tun0a

   # In ns2 (ingress B)
   sudo ip netns exec ns2 ip addr add 10.0.1.1/24 dev tun0b
   sudo ip netns exec ns2 ip link set tun0b up
   sudo ip netns exec ns2 ip route add default dev tun0b
   ```
4. **Create a simple simulator configuration** (`dual_tun.toml`)
   ```toml
   [interfaces]
   real_tun_a = { name = "tun0a", address = "10.0.0.1", netmask = "255.255.255.0" }
   real_tun_b = { name = "tun0b", address = "10.0.1.1", netmask = "255.255.255.0" }

   [tun_ingress]
   tun_a_ingress = "Rx0y0"   # match your topology router ids
   tun_b_ingress = "Rx0y1"

   [topology]
   # define routers and links as needed …
   ```
5. **Run the simulator**
   ```bash
   sudo ./target/release/network-simulator --config dual_tun.toml --stats
   ```
   The `--stats` flag prints per‑router statistics after termination.

6. **Test connectivity** (in separate terminals):
   ```bash
   # From ns1 ping ns2's address (through the simulated network)
   sudo ip netns exec ns1 ping -c 3 10.0.1.1

   # From ns2 ping ns1's address
   sudo ip netns exec ns2 ping -c 3 10.0.0.1
   ```
   Successful replies indicate the simulator correctly forwards packets between the two TUN interfaces.

## Cleanup
```bash
sudo ip netns del ns1
sudo ip netns del ns2
```

Feel free to adjust the topology, routing tables, or enable multipath routing via the `--multipath` flag.
