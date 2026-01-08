# Issue 106: IPv6 Extension Header Support Incomplete

## Summary
The packet parsing code only handles the Hop-by-Hop extension header (Next Header = 0). If other IPv6 extension headers are present (Routing, Fragment, Destination Options, etc.), the code will fail to correctly locate the transport header and extract port information.

## Priority
**Low** - Affects advanced IPv6 use cases with extension headers.

## Location
- File: `src/packet/mod.rs`
- Function: `parse`
- Lines: ~162-175 (based on exploration summary)

## Current Behavior

The code checks for Hop-by-Hop extension header:
```rust
if next_header == 0 {
    // Hop-by-Hop: read extension header length and skip
    let ext_len = raw[40 + 1] as usize;
    let ext_total = 8 + ext_len * 8;
    transport_offset += ext_total;
    next_header = raw[40]; // Next header after Hop-by-Hop
}
```

But it doesn't handle:
- Type 43: Routing Header
- Type 44: Fragment Header
- Type 60: Destination Options
- Type 51: Authentication Header
- Type 50: ESP (Encapsulating Security Payload)

## Expected Behavior

The code should iterate through the extension header chain until it finds a transport protocol (TCP=6, UDP=17, ICMPv6=58, etc.):

```rust
let mut offset = 40; // After fixed IPv6 header
let mut next_header = raw[6];

// Extension headers that can be chained
const EXTENSION_HEADERS: [u8; 5] = [0, 43, 44, 60, 135];

while EXTENSION_HEADERS.contains(&next_header) && offset < raw.len() {
    let ext_next = raw[offset];
    let ext_len = match next_header {
        44 => 8, // Fragment header is always 8 bytes
        _ => 8 + (raw[offset + 1] as usize) * 8,
    };
    offset += ext_len;
    next_header = ext_next;
}

// Now offset points to transport header, next_header is the protocol
```

## Impact
- Packets with Routing headers won't have ports extracted correctly
- Fragmented IPv6 packets won't be parsed correctly
- Load balancing based on 5-tuple will fall back to using port 0
- This is relatively rare in practice as most IPv6 traffic doesn't use extension headers

## Suggested Implementation

1. Create a helper function to skip extension headers:
```rust
fn skip_ipv6_extension_headers(raw: &[u8]) -> (usize, u8) {
    let mut offset = 40;
    let mut next_header = raw[6];
    
    loop {
        match next_header {
            0 | 43 | 60 | 135 => {
                // Variable-length extension headers
                if offset + 2 > raw.len() { break; }
                let ext_len = 8 + (raw[offset + 1] as usize) * 8;
                next_header = raw[offset];
                offset += ext_len;
            }
            44 => {
                // Fragment header: fixed 8 bytes
                if offset + 8 > raw.len() { break; }
                next_header = raw[offset];
                offset += 8;
            }
            _ => break, // Not an extension header
        }
    }
    (offset, next_header)
}
```

2. Update the parse function to use this helper.

3. Add tests with packets containing extension headers.

## Resolution
(To be filled when resolved)

---
*Created: 2026-01-08*
