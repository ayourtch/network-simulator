# Open Issues Index

This directory contains documented open issues for the network simulator project. Each issue is small, granular, and designed to be implementable by a less experienced developer. When you address a given issue, please move it to the docs/issues/resolved/ directory (thus, ../resolved relative to this one). You can also add "Fix summary" section to the file after moving it, documenting what you have done and why.

## Implementation Order for Full Functionality

To achieve end-to-end packet forwarding, implement in this order:

1. [014](014-destination-detection-incorrect.md) - Fix routing direction
2. [020](020-raw-packet-bytes-not-preserved.md) - Preserve packet bytes
3. [008](008-hop-by-hop-forwarding-not-implemented.md) - Implement forwarding loop
4. [002](002-ttl-decrement-not-implemented.md) + [021](021-ipv4-checksum-not-implemented.md) - TTL handling
5. [024](024-two-tun-devices-not-supported.md) + [009](009-tun-write-back-not-implemented.md) - Dual TUN support
6. [004](004-mtu-enforcement-not-implemented.md) - MTU checking
7. [005](005-icmp-time-exceeded-stub.md) + [006](006-icmp-fragmentation-needed-stub.md) - ICMP errors
8. [011](011-icmp-routing-not-implemented.md) - ICMP routing

## Issue Format

Each issue file contains:
- **Summary**: Brief description of the issue
- **Location**: Files and functions affected
- **Current Behavior**: What happens now
- **Expected Behavior**: What should happen (with plan references)
- **Recommended Solution**: Code snippets and implementation guidance
- **Files to Modify**: Specific files that need changes
- **Effort Estimate**: Small (< 2 hours), Medium (2-4 hours), Large (4+ hours)
- **Dependencies**: Other issues that should be completed first
- **Related Plans**: Reference to specification documents
