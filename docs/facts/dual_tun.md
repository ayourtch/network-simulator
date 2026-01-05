# Dual TUN Support Fact

The simulator now supports two real TUN interfaces (`real_tun_a` and `real_tun_b`). Packets read from one TUN are processed and written out the opposite TUN, enabling a Linux host to run behind the simulator and communicate through the virtual network.
