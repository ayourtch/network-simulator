#!/usr/bin/env python3
"""
Simple Network Simulator

A basic event-driven network simulator that models nodes, links, and packet transmission.
"""

import heapq
from collections import defaultdict, deque
from typing import Dict, List, Optional, Tuple
import time


class Packet:
    """Represents a packet being transmitted through the network."""
    
    def __init__(self, packet_id: int, source: str, destination: str, size: int = 1500):
        self.packet_id = packet_id
        self.source = source
        self.destination = destination
        self.size = size  # bytes
        self.creation_time = 0.0
        self.delivery_time = 0.0
        self.hops = []  # Track the path taken
        
    def __repr__(self):
        return f"Packet({self.packet_id}: {self.source}->{self.destination})"


class Link:
    """Represents a network link between two nodes."""
    
    def __init__(self, node1: str, node2: str, bandwidth: float = 1000000, latency: float = 0.001):
        self.node1 = node1
        self.node2 = node2
        self.bandwidth = bandwidth  # bits per second
        self.latency = latency  # seconds
        self.packets_transmitted = 0
        
    def get_transmission_time(self, packet_size: int) -> float:
        """Calculate transmission time for a packet."""
        return (packet_size * 8) / self.bandwidth + self.latency
    
    def get_other_end(self, node: str) -> str:
        """Get the node on the other end of the link."""
        if node == self.node1:
            return self.node2
        elif node == self.node2:
            return self.node1
        else:
            raise ValueError(f"Node {node} is not connected to this link")
    
    def __repr__(self):
        return f"Link({self.node1}<->{self.node2}, {self.bandwidth/1e6:.1f}Mbps, {self.latency*1000:.2f}ms)"


class Node:
    """Represents a network node (host, router, or switch)."""
    
    def __init__(self, name: str, node_type: str = "host"):
        self.name = name
        self.node_type = node_type  # host, router, switch
        self.links = []  # Links connected to this node
        self.routing_table = {}  # destination -> next_hop
        self.packets_sent = 0
        self.packets_received = 0
        self.packets_forwarded = 0
        
    def add_link(self, link: Link):
        """Add a link to this node."""
        self.links.append(link)
        
    def get_neighbors(self) -> List[str]:
        """Get all neighboring nodes."""
        neighbors = []
        for link in self.links:
            try:
                neighbors.append(link.get_other_end(self.name))
            except ValueError:
                pass
        return neighbors
    
    def __repr__(self):
        return f"Node({self.name}, {self.node_type})"


class Event:
    """Represents a simulation event."""
    
    def __init__(self, time: float, event_type: str, packet: Packet, node: str):
        self.time = time
        self.event_type = event_type  # 'arrival', 'departure'
        self.packet = packet
        self.node = node
        
    def __lt__(self, other):
        return self.time < other.time
    
    def __repr__(self):
        return f"Event({self.time:.6f}s, {self.event_type}, {self.packet}, {self.node})"


class NetworkSimulator:
    """Event-driven network simulator."""
    
    def __init__(self):
        self.nodes: Dict[str, Node] = {}
        self.links: List[Link] = {}
        self.events: List[Event] = []  # Priority queue
        self.current_time = 0.0
        self.packet_counter = 0
        self.delivered_packets = []
        self.lost_packets = []
        
    def add_node(self, name: str, node_type: str = "host") -> Node:
        """Add a node to the network."""
        if name in self.nodes:
            raise ValueError(f"Node {name} already exists")
        node = Node(name, node_type)
        self.nodes[name] = node
        return node
    
    def add_link(self, node1: str, node2: str, bandwidth: float = 1000000, latency: float = 0.001) -> Link:
        """Add a bidirectional link between two nodes."""
        if node1 not in self.nodes or node2 not in self.nodes:
            raise ValueError("Both nodes must exist before creating a link")
        
        link = Link(node1, node2, bandwidth, latency)
        self.links[f"{node1}-{node2}"] = link
        self.nodes[node1].add_link(link)
        self.nodes[node2].add_link(link)
        return link
    
    def compute_routing_tables(self):
        """Compute shortest path routing tables using Dijkstra's algorithm."""
        for source_name in self.nodes:
            # Dijkstra's algorithm
            distances = {node: float('inf') for node in self.nodes}
            distances[source_name] = 0
            next_hop = {node: None for node in self.nodes}
            visited = set()
            pq = [(0, source_name, source_name)]  # (distance, node, first_hop)
            
            while pq:
                dist, current, first = heapq.heappop(pq)
                
                if current in visited:
                    continue
                    
                visited.add(current)
                
                # Set next hop for this destination
                if current != source_name:
                    next_hop[current] = first
                
                # Check neighbors
                for link in self.nodes[current].links:
                    try:
                        neighbor = link.get_other_end(current)
                        new_dist = dist + 1  # Hop count
                        
                        if new_dist < distances[neighbor]:
                            distances[neighbor] = new_dist
                            # Determine first hop
                            if current == source_name:
                                heapq.heappush(pq, (new_dist, neighbor, neighbor))
                            else:
                                heapq.heappush(pq, (new_dist, neighbor, first))
                    except ValueError:
                        pass
            
            self.nodes[source_name].routing_table = next_hop
    
    def send_packet(self, source: str, destination: str, size: int = 1500, send_time: float = None):
        """Schedule a packet to be sent from source to destination."""
        if source not in self.nodes or destination not in self.nodes:
            raise ValueError("Source and destination must be valid nodes")
        
        if send_time is None:
            send_time = self.current_time
            
        packet = Packet(self.packet_counter, source, destination, size)
        self.packet_counter += 1
        packet.creation_time = send_time
        packet.hops.append(source)
        
        # Schedule the initial transmission
        event = Event(send_time, 'departure', packet, source)
        heapq.heappush(self.events, event)
        self.nodes[source].packets_sent += 1
        
        return packet
    
    def _process_event(self, event: Event):
        """Process a single event."""
        self.current_time = event.time
        packet = event.packet
        current_node = event.node
        
        if event.event_type == 'departure':
            # Packet is leaving this node
            node = self.nodes[current_node]
            
            # Check if this is the destination
            if current_node == packet.destination:
                packet.delivery_time = self.current_time
                self.delivered_packets.append(packet)
                node.packets_received += 1
                return
            
            # Forward the packet
            next_hop = node.routing_table.get(packet.destination)
            
            if next_hop is None:
                # No route found, packet is lost
                self.lost_packets.append(packet)
                return
            
            # Find the link to next hop
            link = None
            for l in node.links:
                try:
                    if l.get_other_end(current_node) == next_hop:
                        link = l
                        break
                except ValueError:
                    pass
            
            if link is None:
                self.lost_packets.append(packet)
                return
            
            # Schedule arrival at next hop
            transmission_time = link.get_transmission_time(packet.size)
            arrival_time = self.current_time + transmission_time
            
            packet.hops.append(next_hop)
            link.packets_transmitted += 1
            node.packets_forwarded += 1
            
            arrival_event = Event(arrival_time, 'departure', packet, next_hop)
            heapq.heappush(self.events, arrival_event)
    
    def run(self, until: float = None):
        """Run the simulation until specified time or until no events remain."""
        while self.events:
            event = heapq.heappop(self.events)
            
            if until is not None and event.time > until:
                # Put the event back and stop
                heapq.heappush(self.events, event)
                self.current_time = until
                break
                
            self._process_event(event)
    
    def get_statistics(self) -> Dict:
        """Get simulation statistics."""
        total_latency = sum(p.delivery_time - p.creation_time for p in self.delivered_packets)
        avg_latency = total_latency / len(self.delivered_packets) if self.delivered_packets else 0
        
        total_hops = sum(len(p.hops) - 1 for p in self.delivered_packets)
        avg_hops = total_hops / len(self.delivered_packets) if self.delivered_packets else 0
        
        return {
            'total_packets_sent': self.packet_counter,
            'packets_delivered': len(self.delivered_packets),
            'packets_lost': len(self.lost_packets),
            'delivery_rate': len(self.delivered_packets) / self.packet_counter if self.packet_counter > 0 else 0,
            'average_latency': avg_latency,
            'average_hops': avg_hops,
            'simulation_time': self.current_time
        }
    
    def print_statistics(self):
        """Print simulation statistics in a readable format."""
        stats = self.get_statistics()
        print("\n" + "="*60)
        print("SIMULATION STATISTICS")
        print("="*60)
        print(f"Simulation Time:      {stats['simulation_time']:.6f} seconds")
        print(f"Total Packets Sent:   {stats['total_packets_sent']}")
        print(f"Packets Delivered:    {stats['packets_delivered']}")
        print(f"Packets Lost:         {stats['packets_lost']}")
        print(f"Delivery Rate:        {stats['delivery_rate']*100:.2f}%")
        print(f"Average Latency:      {stats['average_latency']*1000:.3f} ms")
        print(f"Average Hops:         {stats['average_hops']:.2f}")
        print("="*60)
        
        print("\nNode Statistics:")
        print("-"*60)
        for name, node in sorted(self.nodes.items()):
            print(f"{name:15} Sent: {node.packets_sent:3}  Received: {node.packets_received:3}  Forwarded: {node.packets_forwarded:3}")
        print("-"*60)


def main():
    """Example usage of the network simulator."""
    print("Network Simulator Experiment\n")
    
    # Create a network simulator
    sim = NetworkSimulator()
    
    # Create a simple network topology
    #
    #     [H1] ---1Mbps--- [R1] ---10Mbps--- [R2] ---1Mbps--- [H2]
    #                       |                   |
    #                       +---100Mbps---------+
    #
    print("Creating network topology...")
    sim.add_node("H1", "host")
    sim.add_node("H2", "host")
    sim.add_node("R1", "router")
    sim.add_node("R2", "router")
    
    # Add links (bandwidth in bps, latency in seconds)
    sim.add_link("H1", "R1", bandwidth=1e6, latency=0.010)    # 1 Mbps, 10ms
    sim.add_link("R1", "R2", bandwidth=10e6, latency=0.005)   # 10 Mbps, 5ms
    sim.add_link("R2", "H2", bandwidth=1e6, latency=0.010)    # 1 Mbps, 10ms
    sim.add_link("R1", "R2", bandwidth=100e6, latency=0.001)  # 100 Mbps, 1ms (alternative fast path)
    
    print("Network topology created.")
    print("\nNodes:")
    for name, node in sim.nodes.items():
        print(f"  {node}")
    
    print("\nLinks:")
    for link in sim.links.values():
        print(f"  {link}")
    
    # Compute routing tables
    print("\nComputing routing tables...")
    sim.compute_routing_tables()
    
    print("\nRouting tables computed.")
    for name, node in sim.nodes.items():
        print(f"  {name}: {node.routing_table}")
    
    # Send some packets
    print("\nSending packets...")
    for i in range(5):
        sim.send_packet("H1", "H2", size=1500, send_time=i * 0.1)
        sim.send_packet("H2", "H1", size=1500, send_time=i * 0.1 + 0.05)
    
    # Run simulation
    print("\nRunning simulation...")
    sim.run()
    
    # Print statistics
    sim.print_statistics()
    
    # Show packet paths
    print("\nPacket Paths:")
    print("-"*60)
    for packet in sim.delivered_packets[:10]:  # Show first 10
        path = " -> ".join(packet.hops)
        latency = (packet.delivery_time - packet.creation_time) * 1000
        print(f"  {packet}: {path} ({latency:.3f} ms)")
    print("-"*60)


if __name__ == "__main__":
    main()
