#!/usr/bin/env python3
"""
Advanced Network Simulator Examples

Demonstrates different network topologies and simulation scenarios.
"""

from network_simulator import NetworkSimulator


def example_star_topology():
    """Example: Star topology with central router."""
    print("\n" + "="*70)
    print("EXAMPLE 1: Star Topology")
    print("="*70)
    
    sim = NetworkSimulator()
    
    # Create star topology: multiple hosts connected to central router
    #
    #       H1
    #        |
    #   H2--R1--H3
    #        |
    #       H4
    #
    sim.add_node("R1", "router")
    for i in range(1, 5):
        sim.add_node(f"H{i}", "host")
        sim.add_link(f"H{i}", "R1", bandwidth=100e6, latency=0.001)
    
    sim.compute_routing_tables()
    
    # Send packets between hosts (must go through router)
    sim.send_packet("H1", "H2", size=1500)
    sim.send_packet("H2", "H3", size=1500)
    sim.send_packet("H3", "H4", size=1500)
    sim.send_packet("H4", "H1", size=1500)
    
    sim.run()
    sim.print_statistics()


def example_linear_topology():
    """Example: Linear chain of nodes."""
    print("\n" + "="*70)
    print("EXAMPLE 2: Linear Chain Topology")
    print("="*70)
    
    sim = NetworkSimulator()
    
    # Create linear topology: H1 -- R1 -- R2 -- R3 -- H2
    nodes = ["H1", "R1", "R2", "R3", "H2"]
    types = ["host", "router", "router", "router", "host"]
    
    for node, node_type in zip(nodes, types):
        sim.add_node(node, node_type)
    
    for i in range(len(nodes) - 1):
        sim.add_link(nodes[i], nodes[i+1], bandwidth=10e6, latency=0.005)
    
    sim.compute_routing_tables()
    
    # Send packets end-to-end
    for i in range(3):
        sim.send_packet("H1", "H2", size=1500, send_time=i * 0.1)
    
    sim.run()
    sim.print_statistics()
    
    # Show all packet paths
    print("\nAll Packet Paths:")
    for packet in sim.delivered_packets:
        path = " -> ".join(packet.hops)
        latency = (packet.delivery_time - packet.creation_time) * 1000
        print(f"  {packet}: {path} (latency: {latency:.3f} ms)")


def example_mesh_topology():
    """Example: Fully connected mesh network."""
    print("\n" + "="*70)
    print("EXAMPLE 3: Mesh Topology (4 nodes)")
    print("="*70)
    
    sim = NetworkSimulator()
    
    # Create mesh topology: all nodes connected to all others
    #
    #   R1 --- R2
    #   |  \ / |
    #   |   X  |
    #   |  / \ |
    #   R3 --- R4
    #
    nodes = ["R1", "R2", "R3", "R4"]
    for node in nodes:
        sim.add_node(node, "router")
    
    # Connect all pairs
    for i in range(len(nodes)):
        for j in range(i+1, len(nodes)):
            sim.add_link(nodes[i], nodes[j], bandwidth=50e6, latency=0.002)
    
    sim.compute_routing_tables()
    
    # Send packets (will take shortest path)
    sim.send_packet("R1", "R4", size=1500)
    sim.send_packet("R2", "R3", size=1500)
    
    sim.run()
    sim.print_statistics()


def example_bandwidth_comparison():
    """Example: Compare different link bandwidths."""
    print("\n" + "="*70)
    print("EXAMPLE 4: Bandwidth Comparison")
    print("="*70)
    
    # Test different bandwidths
    bandwidths = [1e6, 10e6, 100e6, 1e9]  # 1Mbps, 10Mbps, 100Mbps, 1Gbps
    packet_size = 12000  # bytes (large packet)
    
    print(f"\nTransmitting {packet_size} byte packet over different bandwidths:\n")
    
    for bw in bandwidths:
        sim = NetworkSimulator()
        sim.add_node("H1", "host")
        sim.add_node("H2", "host")
        sim.add_link("H1", "H2", bandwidth=bw, latency=0.001)
        
        sim.compute_routing_tables()
        sim.send_packet("H1", "H2", size=packet_size)
        sim.run()
        
        stats = sim.get_statistics()
        print(f"  {bw/1e6:8.1f} Mbps: {stats['average_latency']*1000:8.3f} ms")


def example_traffic_burst():
    """Example: Burst of traffic."""
    print("\n" + "="*70)
    print("EXAMPLE 5: Traffic Burst")
    print("="*70)
    
    sim = NetworkSimulator()
    
    # Simple topology
    sim.add_node("H1", "host")
    sim.add_node("R1", "router")
    sim.add_node("H2", "host")
    sim.add_link("H1", "R1", bandwidth=10e6, latency=0.005)
    sim.add_link("R1", "H2", bandwidth=10e6, latency=0.005)
    
    sim.compute_routing_tables()
    
    # Send burst of packets
    print("\nSending burst of 20 packets...")
    for i in range(20):
        sim.send_packet("H1", "H2", size=1500, send_time=0.0)  # All at time 0
    
    sim.run()
    sim.print_statistics()


def example_bidirectional_traffic():
    """Example: Bidirectional traffic between hosts."""
    print("\n" + "="*70)
    print("EXAMPLE 6: Bidirectional Traffic")
    print("="*70)
    
    sim = NetworkSimulator()
    
    # Create simple network
    sim.add_node("H1", "host")
    sim.add_node("R1", "router")
    sim.add_node("H2", "host")
    sim.add_link("H1", "R1", bandwidth=100e6, latency=0.002)
    sim.add_link("R1", "H2", bandwidth=100e6, latency=0.002)
    
    sim.compute_routing_tables()
    
    # Simultaneous bidirectional traffic
    print("\nSimulating bidirectional traffic...")
    for i in range(5):
        sim.send_packet("H1", "H2", size=1500, send_time=i * 0.01)
        sim.send_packet("H2", "H1", size=1500, send_time=i * 0.01)
    
    sim.run()
    sim.print_statistics()
    
    # Show packet paths
    print("\nPacket Paths:")
    for packet in sorted(sim.delivered_packets, key=lambda p: p.creation_time):
        path = " -> ".join(packet.hops)
        latency = (packet.delivery_time - packet.creation_time) * 1000
        created = packet.creation_time * 1000
        print(f"  t={created:6.2f}ms: {packet}: {path} ({latency:.3f} ms)")


def main():
    """Run all examples."""
    print("\n" + "#"*70)
    print("# Network Simulator - Advanced Examples")
    print("#"*70)
    
    examples = [
        example_star_topology,
        example_linear_topology,
        example_mesh_topology,
        example_bandwidth_comparison,
        example_traffic_burst,
        example_bidirectional_traffic,
    ]
    
    for example in examples:
        try:
            example()
        except Exception as e:
            print(f"\nError in {example.__name__}: {e}")
    
    print("\n" + "#"*70)
    print("# All examples completed!")
    print("#"*70 + "\n")


if __name__ == "__main__":
    main()
