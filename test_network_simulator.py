#!/usr/bin/env python3
"""
Unit tests for the network simulator.
"""

import unittest
from network_simulator import NetworkSimulator, Node, Link, Packet


class TestNetworkSimulator(unittest.TestCase):
    """Test cases for the network simulator."""
    
    def setUp(self):
        """Set up a fresh simulator for each test."""
        self.sim = NetworkSimulator()
    
    def test_add_node(self):
        """Test adding nodes to the network."""
        node = self.sim.add_node("H1", "host")
        self.assertIsNotNone(node)
        self.assertEqual(node.name, "H1")
        self.assertEqual(node.node_type, "host")
        self.assertIn("H1", self.sim.nodes)
        
    def test_duplicate_node(self):
        """Test that adding duplicate nodes raises an error."""
        self.sim.add_node("H1", "host")
        with self.assertRaises(ValueError):
            self.sim.add_node("H1", "host")
    
    def test_add_link(self):
        """Test adding links between nodes."""
        self.sim.add_node("H1", "host")
        self.sim.add_node("H2", "host")
        link = self.sim.add_link("H1", "H2", bandwidth=1e6, latency=0.01)
        
        self.assertIsNotNone(link)
        self.assertEqual(link.bandwidth, 1e6)
        self.assertEqual(link.latency, 0.01)
        self.assertIn(link, self.sim.nodes["H1"].links)
        self.assertIn(link, self.sim.nodes["H2"].links)
    
    def test_link_nonexistent_nodes(self):
        """Test that linking nonexistent nodes raises an error."""
        with self.assertRaises(ValueError):
            self.sim.add_link("H1", "H2")
    
    def test_simple_routing(self):
        """Test routing table computation for a simple network."""
        # Create: H1 -- R1 -- H2
        self.sim.add_node("H1", "host")
        self.sim.add_node("R1", "router")
        self.sim.add_node("H2", "host")
        self.sim.add_link("H1", "R1")
        self.sim.add_link("R1", "H2")
        
        self.sim.compute_routing_tables()
        
        # H1 should route to H2 via R1
        self.assertEqual(self.sim.nodes["H1"].routing_table["H2"], "R1")
        # H2 should route to H1 via R1
        self.assertEqual(self.sim.nodes["H2"].routing_table["H1"], "R1")
    
    def test_packet_transmission(self):
        """Test basic packet transmission."""
        # Create: H1 -- H2
        self.sim.add_node("H1", "host")
        self.sim.add_node("H2", "host")
        self.sim.add_link("H1", "H2", bandwidth=1e6, latency=0.001)
        
        self.sim.compute_routing_tables()
        packet = self.sim.send_packet("H1", "H2", size=1500)
        
        self.assertIsNotNone(packet)
        self.assertEqual(packet.source, "H1")
        self.assertEqual(packet.destination, "H2")
        self.assertEqual(len(self.sim.events), 1)
    
    def test_packet_delivery(self):
        """Test that packets are delivered successfully."""
        # Create: H1 -- H2
        self.sim.add_node("H1", "host")
        self.sim.add_node("H2", "host")
        self.sim.add_link("H1", "H2", bandwidth=1e6, latency=0.001)
        
        self.sim.compute_routing_tables()
        self.sim.send_packet("H1", "H2", size=1500)
        self.sim.run()
        
        self.assertEqual(len(self.sim.delivered_packets), 1)
        self.assertEqual(len(self.sim.lost_packets), 0)
        self.assertEqual(self.sim.nodes["H1"].packets_sent, 1)
        self.assertEqual(self.sim.nodes["H2"].packets_received, 1)
    
    def test_multi_hop_routing(self):
        """Test packet delivery through multiple hops."""
        # Create: H1 -- R1 -- R2 -- H2
        self.sim.add_node("H1", "host")
        self.sim.add_node("R1", "router")
        self.sim.add_node("R2", "router")
        self.sim.add_node("H2", "host")
        self.sim.add_link("H1", "R1")
        self.sim.add_link("R1", "R2")
        self.sim.add_link("R2", "H2")
        
        self.sim.compute_routing_tables()
        self.sim.send_packet("H1", "H2", size=1500)
        self.sim.run()
        
        self.assertEqual(len(self.sim.delivered_packets), 1)
        packet = self.sim.delivered_packets[0]
        self.assertEqual(len(packet.hops), 4)  # H1 -> R1 -> R2 -> H2
        self.assertEqual(packet.hops, ["H1", "R1", "R2", "H2"])
    
    def test_statistics(self):
        """Test statistics calculation."""
        # Create simple network
        self.sim.add_node("H1", "host")
        self.sim.add_node("H2", "host")
        self.sim.add_link("H1", "H2", bandwidth=1e6, latency=0.001)
        
        self.sim.compute_routing_tables()
        
        # Send multiple packets
        for i in range(5):
            self.sim.send_packet("H1", "H2", size=1500)
        
        self.sim.run()
        
        stats = self.sim.get_statistics()
        self.assertEqual(stats['total_packets_sent'], 5)
        self.assertEqual(stats['packets_delivered'], 5)
        self.assertEqual(stats['packets_lost'], 0)
        self.assertEqual(stats['delivery_rate'], 1.0)
        self.assertGreater(stats['average_latency'], 0)
    
    def test_link_get_other_end(self):
        """Test Link.get_other_end method."""
        link = Link("A", "B")
        self.assertEqual(link.get_other_end("A"), "B")
        self.assertEqual(link.get_other_end("B"), "A")
        
        with self.assertRaises(ValueError):
            link.get_other_end("C")
    
    def test_transmission_time(self):
        """Test transmission time calculation."""
        # 1 Mbps link, 1ms latency
        link = Link("A", "B", bandwidth=1e6, latency=0.001)
        
        # 1500 bytes = 12000 bits
        # At 1 Mbps = 12000 / 1000000 = 0.012 seconds
        # Plus 0.001 seconds latency = 0.013 seconds
        time = link.get_transmission_time(1500)
        self.assertAlmostEqual(time, 0.013, places=6)
    
    def test_no_route(self):
        """Test behavior when there's no route to destination."""
        # Create two disconnected pairs
        self.sim.add_node("H1", "host")
        self.sim.add_node("H2", "host")
        self.sim.add_node("H3", "host")
        self.sim.add_node("H4", "host")
        self.sim.add_link("H1", "H2")
        self.sim.add_link("H3", "H4")
        
        self.sim.compute_routing_tables()
        
        # Try to send from H1 to H3 (no route)
        self.sim.send_packet("H1", "H3", size=1500)
        self.sim.run()
        
        self.assertEqual(len(self.sim.delivered_packets), 0)
        self.assertEqual(len(self.sim.lost_packets), 1)


class TestNode(unittest.TestCase):
    """Test cases for the Node class."""
    
    def test_node_creation(self):
        """Test node initialization."""
        node = Node("R1", "router")
        self.assertEqual(node.name, "R1")
        self.assertEqual(node.node_type, "router")
        self.assertEqual(len(node.links), 0)
        self.assertEqual(node.packets_sent, 0)
        self.assertEqual(node.packets_received, 0)
        self.assertEqual(node.packets_forwarded, 0)
    
    def test_get_neighbors(self):
        """Test getting neighboring nodes."""
        node1 = Node("N1")
        node2 = Node("N2")
        node3 = Node("N3")
        
        link1 = Link("N1", "N2")
        link2 = Link("N1", "N3")
        
        node1.add_link(link1)
        node1.add_link(link2)
        
        neighbors = node1.get_neighbors()
        self.assertEqual(set(neighbors), {"N2", "N3"})


class TestPacket(unittest.TestCase):
    """Test cases for the Packet class."""
    
    def test_packet_creation(self):
        """Test packet initialization."""
        packet = Packet(1, "H1", "H2", 1500)
        self.assertEqual(packet.packet_id, 1)
        self.assertEqual(packet.source, "H1")
        self.assertEqual(packet.destination, "H2")
        self.assertEqual(packet.size, 1500)
        self.assertEqual(len(packet.hops), 0)


if __name__ == "__main__":
    unittest.main()
