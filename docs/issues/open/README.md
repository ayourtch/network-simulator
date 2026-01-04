# Open Issues Index

This directory contains documented open issues for the network simulator project. Each issue is small, granular, and designed to be implementable by a less experienced developer.

## Issue Categories

### Critical Path Issues (Core Functionality)
These issues block basic end-to-end functionality:

| Issue | Title | Priority | Effort |
|-------|-------|----------|--------|
| [008](008-hop-by-hop-forwarding-not-implemented.md) | Hop-by-Hop Forwarding Loop Not Implemented | High | Medium |
| [009](009-tun-write-back-not-implemented.md) | TUN Write-Back Not Implemented | High | Medium |
| [014](014-destination-detection-incorrect.md) | Destination Detection Based on Routing Table Incorrect | High | Small |
| [020](020-raw-packet-bytes-not-preserved.md) | Raw Packet Bytes Not Preserved Through Processing | High | Medium |
| [024](024-two-tun-devices-not-supported.md) | Two TUN Devices Not Fully Supported | High | Medium |

### Packet Processing Issues
| Issue | Title | Priority | Effort |
|-------|-------|----------|--------|
| [001](001-ipv6-packet-parsing-not-implemented.md) | IPv6 Packet Parsing Not Implemented | Medium | Small |
| [002](002-ttl-decrement-not-implemented.md) | TTL/Hop-Limit Decrement Not Implemented | Medium | Small |
| [003](003-port-extraction-not-implemented.md) | TCP/UDP Port Extraction Not Implemented | Medium | Small |
| [021](021-ipv4-checksum-not-implemented.md) | IPv4 Header Checksum Recalculation Not Implemented | Medium | Small |

### Link Simulation Issues
| Issue | Title | Priority | Effort |
|-------|-------|----------|--------|
| [004](004-mtu-enforcement-not-implemented.md) | MTU Enforcement Not Implemented | Medium | Small |
| [022](022-jitter-implementation-asymmetric.md) | Jitter Implementation Asymmetric | Low | Small |
| [023](023-bandwidth-limiting-not-implemented.md) | Bandwidth Limiting Not Implemented | Low | Small-Medium |

### ICMP Issues
| Issue | Title | Priority | Effort |
|-------|-------|----------|--------|
| [005](005-icmp-time-exceeded-stub.md) | ICMP Time Exceeded Generation Is Stub | Medium | Medium |
| [006](006-icmp-fragmentation-needed-stub.md) | ICMP Fragmentation Needed Generation Not Implemented | Medium | Medium |
| [007](007-icmpv6-not-implemented.md) | ICMPv6 Support Not Implemented | Low | Medium |
| [011](011-icmp-routing-not-implemented.md) | ICMP Routing Back to Source Not Implemented | Medium | Medium |

### Multi-path Routing Issues
| Issue | Title | Priority | Effort |
|-------|-------|----------|--------|
| [013](013-link-counter-not-used-in-hash.md) | Link Counter Not Fully Integrated for Per-Packet Load Balancing | Medium | Small |

### Configuration & Validation Issues
| Issue | Title | Priority | Effort |
|-------|-------|----------|--------|
| [016](016-bidirectional-link-validation.md) | Bidirectional Link Validation Not Implemented | Low | Small |
| [017](017-router-name-validation-not-strict.md) | Router Name Validation Not Strict | Low | Small |
| [018](018-ingress-router-validation-missing.md) | Ingress Router Validation Missing | Low | Small |
| [019](019-link-references-validation.md) | Link References Unknown Routers Not Fully Validated | Low | Small |

### Infrastructure Issues
| Issue | Title | Priority | Effort |
|-------|-------|----------|--------|
| [012](012-statistics-not-exposed.md) | Statistics Collection Not Exposed | Low | Small |
| [027](027-no-graceful-shutdown.md) | No Graceful Shutdown Handler | Low | Small |
| [028](028-seed-not-used.md) | Seed for Reproducible Random Behavior Not Used | Low | Small |
| [029](029-no-benchmarks.md) | No Benchmarks Implemented | Low | Small |
| [030](030-no-fabric-auto-generation.md) | Default 6x6 Fabric Not Auto-Generated | Low | Small |

### Documentation & Cleanup Issues
| Issue | Title | Priority | Effort |
|-------|-------|----------|--------|
| [010](010-virtual-customer-not-used.md) | VirtualCustomer Feature Not Used | Low | Small |
| [015](015-readme-roadmap-outdated.md) | README Roadmap Should Be Updated | Low | Small |
| [025](025-documentation-inconsistencies.md) | Documentation Inconsistencies with Implementation | Medium | Medium |
| [026](026-integration-tests-incomplete.md) | Integration Tests Don't Test Full Pipeline | Medium | Medium-Large |

## Getting Started

For new contributors, we recommend starting with **Small effort** issues in the following order:

1. **Validation issues** (016-019) - Good for learning the codebase structure
2. **Documentation issues** (015, 025) - Learn what the system should do
3. **Packet processing issues** (001, 003) - Core functionality, well-defined scope
4. **Statistics & infrastructure** (012, 027, 028) - Useful improvements

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
