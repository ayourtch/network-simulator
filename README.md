# network-simulator
=======

This is a very advanced Rust-based network simulator.

It simulates a virtual fabric of 6x6 routers, both on IPv4 and IPv6,
which is forwarding the traffic between the two tun interfaces that
it opens on the Linux host, we will call them tunA and tunB. We will
call these routers Rx0y0, Rx2y1, ... Rx5y5 - with the numbers denoting two coordinates, for easy visualisation during the design.

The forwarding respects the topology configured, as well as MTU, delay, jitter, and loss configuration on the links between the routers in the fabric, as well as between the tunA and tunB interfaces, taking into the account the TTL of the packets and sending back the ICMP errors as needed.

There are two routers which are designated as "dedicated ingress" for tunA and tunB each in configuration - the packets retrieved from tunA and tunB are injected into the forwarding graph in these respective routers.

The links are described in the toml file, with the section names being the concatenation of two sides of the link with underscore. Note, that unless noted otherwise, the links are bidirectional by default (thus, the order of the hosts in the name of the section does not matter - but, one should detect the sections which collide this way.

Based on this topology, each virtual router needs to have a simplified computed "routing table", with two entries: tunA and tunB. The packets incoming via tunA will use the routing table for tunB destination, and vice versa. The errors which need to be sent back will use obviously the opposite routing table. 

When the packet is received at tun interface, it should get a "virtual customer #" assigned to it, thus allowing multiple topologies in the future.

The architecture is such that it allows dynamic addition/removal and reconfiguration of all the topologies, without stopping the traffic, however, in the first version this configuration will not be updated dynamically.

If a given router has multiple egress routing entries for a given tapA or tapB destination, then it should choose one based on a hash of 5-tuple and the router hostname. There should be an option to specify a per-link flag, which will cause the addition of a counter to the hash - thus effectively making a per-packet load balancing simulation.

This project provides a very flexible framework for the network simulation.

