# Example Usage

This directory contains a minimal configuration and instructions for running the network simulator.

## `config.toml`
A basic configuration is provided at the repository root (`config.toml`). It defines two routers (`Rx0y0` and `Rx5y5`) connected by a single link.

## Running the simulator
```bash
# Build the binary (if not already built)
cargo build --release

# Run with the default configuration
./target/release/network-simulator --config config.toml -vv
```

## Overriding the real TUN device via CLI
If you want to use a real TUN interface instead of the mock file, you can override the configuration values directly from the command line:
```bash
./target/release/network-simulator \
    --config config.toml \
    --tun-name tun0 \
    --tun-address 10.0.0.2 \
    --tun-netmask 255.255.255.0 \
    -vv
```
The flags `--tun-name`, `--tun-address`, and `--tun-netmask` will replace the values under `[interfaces].real_tun`.

## Enabling multipath routing
```bash
./target/release/network-simulator --config config.toml --multipath -vv
```

Feel free to edit `config.toml` to add more routers, links, and simulation parameters.
