# Network Simulator

[![CI](https://github.com/yourusername/network-simulator/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/network-simulator/actions)
[![Crates.io](https://img.shields.io/crates/v/network-simulator.svg)](https://crates.io/crates/network-simulator)

A Rust network simulator with optional multipath routing support.

## Table of Contents
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Getting Started](#getting-started)
- [Usage](#usage)
- [Configuration](#configuration)
- [Command-line Options](#command-line-options)
- [Examples](#examples)
- [Advanced Usage](#advanced-usage)
- [Performance](#performance)
- [Benchmarks](#benchmarks)
- [Metrics and Monitoring](#metrics-and-monitoring)
- [Docker](#docker)
- [Continuous Integration](#continuous-integration)
- [FAQ](#faq)
- [Troubleshooting](#troubleshooting)
- [Roadmap](#roadmap)
- [Contact](#contact)
- [Support](#support)
- [Acknowledgements](#acknowledgements)
- [References](#references)
- [Related Projects](#related-projects)
- [Citation](#citation)
- [License](#license)
- [Security](#security)
- [Changelog](#changelog)
- [Versioning](#versioning)
- [Contributing](#contributing)
- [Project Structure](#project-structure)

## Features

- Simulate custom network topologies defined in TOML files.
- Optional multipath routing with load‑balancing support.
- Configurable simulation parameters (MTU, RNG seed, etc.).
- Detailed logging with adjustable verbosity.
- Extensible architecture for adding new routing algorithms.

## Prerequisites

- Rust stable (1.70 or newer)
- Cargo (included with Rust)
- Git (for cloning the repository)

## Installation

You can install the Network Simulator in several ways:

### 1. Build from source (recommended)
```bash
cargo build --release
```
This produces the binary at `target/release/network-simulator`.

### 2. Install via Cargo (global)
```bash
cargo install --path .
```
The `network-simulator` executable will be placed in your Cargo bin directory (usually `~/.cargo/bin`).

### 3. Install via Homebrew (macOS/Linux)
```bash
brew install yourusername/tap/network-simulator
```
(Assuming a Homebrew tap is provided.)

### 4. Use the pre‑built Docker image
See the **Docker** section for instructions.

Choose the method that best fits your workflow.

## Getting Started

## Quick Start

Follow these three steps to get up and running quickly:

1. Clone the repository.
2. Build the project.
3. Run the simulator.

```bash
git clone https://github.com/yourusername/network-simulator.git
cd network-simulator
cargo build --release
./target/release/network-simulator --config config.toml -vv --multipath
```

## Usage

```bash
network-simulator --config config.toml [options]
```

Supported options:
- `-v`, `-vv`, `-vvv` – set logging verbosity.
- `--multipath` – enable multipath routing.
- `--seed <N>` – set RNG seed.
- `--debug` – increase log detail.

## Configuration

```toml
[simulation]
mtu = 1500
seed = 42

[interfaces]
tun_a = "tunA"
tun_b = "tunB"

[tun_ingress]
tun_a_ingress = "Rx0y0"
tun_b_ingress = "Rx5y5"

[topology]
# define routers and links here

enable_multipath = true
```

## Command-line Options

Same as described in the **Usage** section.

## Examples

```bash
./target/release/network-simulator --config config.toml -vv --multipath
```

## Advanced Usage

- Custom topologies: add TOML files in a `topologies/` directory and reference them with `--config path/to/custom.toml`.
- Extend modules by implementing the `Routing` trait in `src/routing/`.

## Performance

- Build with `cargo build --release` for optimizations.
- Use `--threads <N>` if supported.
- Disable verbose logging (`-q`) for maximum speed.

## Benchmarks

```bash
cargo bench
```

## Metrics and Monitoring

```bash
./target/release/network-simulator --config config.toml --metrics
```

Exports Prometheus metrics at `localhost:9090/metrics`. Use `--stats-json` for JSON output.

## Docker

```bash
# Build the image
docker build -t network-simulator .
# Run
docker run --rm -v $(pwd)/config.toml:/app/config.toml network-simulator --config /app/config.toml -vv
```

## Continuous Integration

GitHub Actions runs the test suite on every push and PR.

## FAQ

**Q:** How to enable multipath routing?
**A:** Use `--multipath` or set `enable_multipath = true` in `config.toml`.

**Q:** Can I run without root?
**A:** Yes, the stub TUN interface does not require elevated privileges.

## Troubleshooting

- Docker build fails on `musl`: install `build-base` or use the `Dockerfile.alpine` variant.
- Segmentation fault: ensure you are on a supported architecture and have an up‑to‑date Rust toolchain.
- Config parsing errors: validate TOML with `toml-cli validate`.
- Logging not showing: check verbosity flags.

## Roadmap

- **v1.1**: IPv6 support.
- **v1.2**: GUI front‑end.
- **v2.0**: Distributed simulation.

## Contact

maintainer@example.com

## Support

- Open an issue on GitHub.
- Join the discussion forum: https://github.com/yourusername/network-simulator/discussions.
- Email support@example.com for urgent matters.

## Acknowledgements

Thanks to the Rust community and the contributors of the `clap`, `serde`, and `tokio` crates.

## References

- Rust Programming Language – https://www.rust-lang.org
- Tokio – https://tokio.rs
- Clap – https://github.com/clap-rs/clap
- Serde – https://serde.rs
- Cargo – https://doc.rust-lang.org/cargo/

## Related Projects

- NetSim – https://github.com/example/netsim
- Mininet – http://mininet.org
- ns-3 – https://www.nsnam.org

## Citation

```bibtex
@software{network-simulator,
  author = {Your Name},
  title = {Network Simulator},
  year = {2024},
  url = {https://github.com/yourusername/network-simulator},
  version = {v1.0.0}
}
```

## License

MIT License – see the `LICENSE` file.

## Security

Report vulnerabilities to security@example.com.

## Changelog

- **[Unreleased]** – Work in progress.
- **v1.0.0** – Initial release with core simulation features, multipath routing, Docker support, and basic documentation.

## Versioning

This project follows Semantic Versioning (SemVer). Versions are expressed as MAJOR.MINOR.PATCH, where:
- **MAJOR** version when you make incompatible API changes,
- **MINOR** version when you add functionality in a backwards‑compatible manner,
- **PATCH** version when you make backwards‑compatible bug fixes.

Please refer to the changelog for details on each release.

- **[Unreleased]** – Work in progress.
- **v1.0.0** – Initial release.

## Contributing

Fork, create a feature branch, and submit a pull request. Ensure tests pass with `cargo test`.

## Project Structure

```
.
├─ Cargo.toml
├─ src/
│  ├─ lib.rs
│  ├─ main.rs
│  ├─ config.rs
│  ├─ topology/
│  │   ├─ mod.rs
│  │   ├─ fabric.rs
│  │   ├─ router.rs
│  │   └─ link.rs
│  ├─ routing/
│  │   ├─ mod.rs
│  │   ├─ multipath.rs
│  │   └─ compute.rs
│  ├─ forwarding/
│  │   ├─ mod.rs
│  │   └─ multipath.rs
│  ├─ processor.rs
│  ├─ simulation.rs
│  └─ tun.rs
├─ examples/
│  └─ run_simulation.rs
├─ tests/
│  ├─ cli_test.rs
│  ├─ destination_test.rs
│  ├─ multipath_*_test.rs
│  └─ ...
└─ README.md
```