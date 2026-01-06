# TUN Header Handling Fact

Linux TUN/TAP devices prepend a 4‑byte header to each packet. The first two bytes are flags, the next two bytes indicate the protocol (0x0800 for IPv4, 0x86DD for IPv6). When reading from a TUN device the header must be stripped before parsing the IP packet, and when writing back the header must be added. This simulator now handles the stripping on read and adds the appropriate header on write.
