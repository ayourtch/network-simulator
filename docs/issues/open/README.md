# Open Issues Index

This directory contains documented open issues for the network simulator project. Each issue is small, granular, and designed to be implementable by a less experienced developer. When you address a given issue, please move it to the docs/issues/resolved/ directory (thus, ../resolved relative to this one). You can also add "Fix summary" section to the file after moving it, documenting what you have done and why.

## Summary of Open Issues (as of 2026-01-04)

### Critical Issues (Blocking Core Functionality)

| Issue | Title | Effort | Dependencies |
|-------|-------|--------|--------------|
| [023](023-ipv4-icmp-error-stub-not-resolved.md) | IPv4 ICMP Error Generation Still a Stub | Medium | None |
| [024](024-ttl-expiration-icmp-not-generated.md) | TTL Expiration Does Not Generate ICMP | Small | 023 |
| [025](025-multipath-processing-is-noop.md) | Multipath Packet Processing Is a No-Op | Medium | None |
| [027](027-forwarding-lacks-destination-detection.md) | Forwarding Loop Lacks Destination Detection | Medium | None |
| [028](028-single-tun-cannot-run-host-behind.md) | Single TUN Mode - Cannot Run Linux Host Behind | Large | None |

### Important Issues (Affecting Quality/Correctness)

| Issue | Title | Effort | Dependencies |
|-------|-------|--------|--------------|
| [026](026-router-statistics-never-updated.md) | Router Statistics Never Updated | Small | None |
| [029](029-real-tun-direction-detection-fragile.md) | Real TUN Direction Detection Is Fragile | Small | 028 |
| [030](030-tun-device-memory-safety.md) | TUN Device Memory Safety Issue | Small | None |
| [031](031-icmp-routing-after-generation-incorrect.md) | ICMP Routing After Generation Incorrect | Medium | 023, 027 |
| [032](032-load-balancing-counter-not-used-in-processor.md) | Load Balancing Counter Not Used in Processor | Small | None |
| [033](033-packet-loss-not-tracked.md) | Packet Loss Not Tracked in Statistics | Small | 026 |

### Documentation & Testing Issues

| Issue | Title | Effort | Dependencies |
|-------|-------|--------|--------------|
| [034](034-documentation-mismatch.md) | Documentation Does Not Match Implementation | Small | None |
| [035](035-no-end-to-end-tests.md) | No Tests for End-to-End Packet Delivery | Medium | 024, 026, 027 |

## Implementation Order for Full Functionality

To achieve end-to-end packet forwarding with a Linux host behind the simulator:

### Phase 1: Fix Core Processing
1. [023](023-ipv4-icmp-error-stub-not-resolved.md) - Fix IPv4 ICMP generation
2. [024](024-ttl-expiration-icmp-not-generated.md) - Add TTL expiration handling
3. [027](027-forwarding-lacks-destination-detection.md) - Add proper destination detection
4. [026](026-router-statistics-never-updated.md) - Fix statistics collection

### Phase 2: Fix TUN Support
5. [030](030-tun-device-memory-safety.md) - Fix TUN memory safety
6. [028](028-single-tun-cannot-run-host-behind.md) - Implement dual TUN support
7. [029](029-real-tun-direction-detection-fragile.md) - Improve direction detection

### Phase 3: Fix Advanced Features
8. [025](025-multipath-processing-is-noop.md) - Implement real multipath processing
9. [031](031-icmp-routing-after-generation-incorrect.md) - Fix ICMP routing
10. [032](032-load-balancing-counter-not-used-in-processor.md) - Integrate load balancing
11. [033](033-packet-loss-not-tracked.md) - Track packet loss statistics

### Phase 4: Documentation & Testing
12. [034](034-documentation-mismatch.md) - Update documentation
13. [035](035-no-end-to-end-tests.md) - Add end-to-end tests

## Issues Referencing Previously "Resolved" Issues

Some issues in the resolved directory were found to be incomplete:

- **Issue 023** references resolved issues 005 and 006 (ICMP stubs)
- **Issue 026** references resolved issue 012 (statistics)
- **Issue 028** references resolved issue 009 (TUN write-back)

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
