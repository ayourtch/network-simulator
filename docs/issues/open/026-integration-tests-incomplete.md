# Issue 026: Integration Tests Don't Test Full Pipeline

## Summary
While there are many unit tests, there are no integration tests that verify the complete packet flow from TUN read to fabric traversal to TUN write.

## Location
- Directory: `tests/`

## Current Test Coverage
Tests exist for:
- CLI argument parsing
- Configuration parsing
- Routing computation
- Multipath routing
- Packet parsing
- Link simulation
- Mock packet file handling

Missing tests for:
- Full hop-by-hop packet traversal
- TTL expiration mid-path
- MTU enforcement mid-path
- ICMP error generation and routing
- Delay accumulation across multiple hops
- Packet loss probability across multiple hops

## Expected Testing (from Plan 9)
From the plan:
> End-to-end ping tests (IPv4 and IPv6)
> MTU verification tests
> Delay and jitter measurement tests
> Packet loss verification tests
> Multi-path routing verification
> ICMP error generation tests
> Performance benchmarking

## Recommended Solution

1. Create integration test file `tests/e2e_test.rs`:
```rust
/// Test packet traverses multiple hops with accumulated delay
#[tokio::test]
async fn test_multi_hop_delay() {
    // Setup 3-hop fabric with 10ms delay each
    // Send packet
    // Verify total delay is ~30ms (±jitter)
}

/// Test packet dropped when TTL expires mid-path
#[tokio::test]
async fn test_ttl_expiry_mid_path() {
    // Setup 5-hop path
    // Send packet with TTL=3
    // Verify packet dropped at 3rd hop
    // Verify ICMP Time Exceeded generated
}

/// Test packet dropped when exceeding link MTU
#[tokio::test]
async fn test_mtu_exceeded() {
    // Setup path with 1000-byte MTU link
    // Send 1500-byte packet
    // Verify ICMP Fragmentation Needed generated
}

/// Test packet loss across multiple hops
#[tokio::test]
async fn test_cumulative_loss() {
    // Setup path with 10% loss on each of 3 hops
    // Send 1000 packets
    // Verify ~27% total loss rate (1 - 0.9^3)
}
```

2. These tests require the full pipeline (Issue 008) to be implemented first.

## Files to Create
- `tests/e2e_test.rs`
- `tests/fixtures/multi_hop_config.toml`

## Effort Estimate
Medium-Large (4-6 hours)

## Dependencies
- Issue 008: Hop-by-hop forwarding
- Issue 002: TTL decrement
- Issue 004: MTU enforcement
- Issue 005: ICMP Time Exceeded

## Related Plans
- Plan 9: Integration and End-to-End Testing
