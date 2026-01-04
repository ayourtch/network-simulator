Running this command:

cargo run --bin network-simulator -- -p examples/packet_hex.txt  -v -v -v

It seems like the packet does not exit the simulator ?

There should be a way to capture the packets that are supposed to exit the mock tun and put them into the file,
there should be one file per mock tun.

Also, there should be a way to specify to which tun interface the packet file is "injecting" the packet.




