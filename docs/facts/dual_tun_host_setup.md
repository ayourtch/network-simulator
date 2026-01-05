# Dual TUN Host Setup Guide

This guide walks you through setting up a dual‑TUN environment so that a Linux host in a network namespace can communicate through the network simulator.

## Prerequisites
- Linux with `ip` and `ip netns` utilities.
- Root privileges (or use `sudo`).
- The simulator binary built (`cargo build --release`).
- Two real TUN devices defined in `config.toml` under `interfaces.real_tun_a` and `interfaces.real_tun_b`.

## Steps
1. **Create network namespaces**
   ```bash
   sudo ip netns add ns1
   sudo ip netns add ns2
   ```
2. **Create TUN interfaces**
   ```bash
   sudo ip tuntap add dev tunA mode tun
   sudo ip tuntap add dev tunB mode tun
   sudo ip link set tunA up
   sudo ip link set tunB up
   ```
3. **Assign TUN interfaces to namespaces**
   ```bash
   sudo ip link set tunA netns ns1
   sudo ip link set tunB netns ns2
   ```
4. **Configure IP addresses** (example using IPv4 `/24` prefixes):
   ```bash
   sudo ip netns exec ns1 ip addr add 10.0.0.2/24 dev tunA
   sudo ip netns exec ns2 ip addr add 10.0.1.2/24 dev tunB
   ```
5. **Add routes** so traffic goes via the TUN interfaces:
   ```bash
   sudo ip netns exec ns1 ip route add default dev tunA
   sudo ip netns exec ns2 ip route add default dev tunB
   ```
6. **Run the simulator** with the real TUN configuration:
   ```bash
   cargo run --release -- \
     --tun-name tunA \
     --packet-file /dev/null   # optional mock packets
   ```
   The simulator will read `config.toml` where `real_tun_a.name` should be `tunA` and `real_tun_b.name` should be `tunB`.
7. **Test connectivity** from one namespace to the other (or external network if configured):
   ```bash
   sudo ip netns exec ns1 ping 10.0.1.2
   ```
   You should see replies passing through the simulator.

## Notes
- Adjust IP addresses and prefixes in `config.toml` to match your topology.
- For IPv6, set the address with a `/64` prefix and ensure `real_tun_a`/`real_tun_b` have the correct IPv6 configuration.
- Use `ip netns exec nsX ip link show` to verify the interfaces are up.
- The simulator will forward packets between `tunA` and `tunB` using the fabric routing logic.
