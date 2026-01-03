# Network Simulator

A simple event-driven network simulator written in Python that models packet transmission through a network of nodes (hosts, routers, switches) connected by links.

## Features

- **Event-driven simulation**: Discrete event simulation engine for accurate timing
- **Network topology**: Create custom network topologies with nodes and links
- **Configurable links**: Set bandwidth and latency for each link
- **Shortest path routing**: Automatic routing table computation using Dijkstra's algorithm
- **Packet transmission**: Simulate packet sending with realistic timing
- **Statistics tracking**: Monitor packet delivery, latency, hop count, and per-node statistics

## Quick Start

Run the example simulation:

```bash
python3 network_simulator.py
```

## Usage

### Creating a Network

```python
from network_simulator import NetworkSimulator

# Create simulator
sim = NetworkSimulator()

# Add nodes
sim.add_node("H1", "host")      # Host node
sim.add_node("R1", "router")    # Router node
sim.add_node("H2", "host")

# Add links (bandwidth in bps, latency in seconds)
sim.add_link("H1", "R1", bandwidth=1e6, latency=0.010)   # 1 Mbps, 10ms
sim.add_link("R1", "H2", bandwidth=10e6, latency=0.005)  # 10 Mbps, 5ms

# Compute routing tables
sim.compute_routing_tables()

# Send packets
sim.send_packet("H1", "H2", size=1500)  # 1500 bytes

# Run simulation
sim.run()

# View statistics
sim.print_statistics()
```

### Network Topology

The example creates the following topology:

```
[H1] ---1Mbps--- [R1] ---100Mbps--- [R2] ---1Mbps--- [H2]
                  |                    |
                  +------10Mbps--------+
```

The simulator automatically finds the shortest path (by hop count) between nodes.

### Components

#### Nodes
- **Hosts**: End-point devices that send and receive packets
- **Routers**: Forward packets based on routing tables
- **Switches**: Can be modeled as routers with different characteristics

#### Links
- Bidirectional connections between nodes
- Configurable bandwidth (bits per second)
- Configurable latency (seconds)

#### Packets
- Data units transmitted through the network
- Track source, destination, size, timing, and path

### Statistics

The simulator tracks:
- Total packets sent/delivered/lost
- Delivery rate percentage
- Average end-to-end latency
- Average number of hops
- Per-node packet counts (sent/received/forwarded)

## Example Output

```
SIMULATION STATISTICS
============================================================
Simulation Time:      0.500200 seconds
Total Packets Sent:   10
Packets Delivered:    10
Packets Lost:         0
Delivery Rate:        100.00%
Average Latency:      50.200 ms
Average Hops:         3.00
============================================================
```

## Implementation Details

- Uses priority queue (heap) for efficient event scheduling
- Implements Dijkstra's algorithm for shortest path routing
- Calculates transmission time based on packet size and link bandwidth
- Models propagation delay through link latency

## Requirements

- Python 3.6+
- No external dependencies (uses only standard library)

## License

MIT