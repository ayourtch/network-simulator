# IPv6 Hop‑by‑Hop Extension Header Handling Fact

The packet parser now detects IPv6 Hop‑by‑Hop Options (Next Header = 0), reads the extension header length, and skips over it to locate the actual transport layer header. This enables correct port extraction and routing decisions for packets that include Hop‑by‑Hop options.
