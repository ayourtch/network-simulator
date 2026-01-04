# Issue 029: No Benchmarks Implemented

## Summary
Plan 9 specifies performance benchmarking, but no benchmarks exist in the project.

## Location
- Missing directory: `benches/`
- Missing in: `Cargo.toml`

## Current Behavior
No benchmarks exist. The `Cargo.toml` doesn't have a `[[bench]]` section or criterion dependency.

## Expected Behavior (from Plan 9)
From the plan:
> Performance benchmarking

The project should include benchmarks for:
- Packet parsing throughput
- Routing table lookup speed
- Link simulation overhead
- End-to-end forwarding latency

## Recommended Solution

1. Add criterion dependency to `Cargo.toml`:
```toml
[dev-dependencies]
criterion = "0.5"
# ... existing deps ...

[[bench]]
name = "forwarding_bench"
harness = false
```

2. Create `benches/forwarding_bench.rs`:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use network_simulator::packet::parse;

fn bench_packet_parse(c: &mut Criterion) {
    let packet = vec![
        0x45, 0x00, 0x00, 0x14,
        0x00, 0x00, 0x00, 0x00,
        0x40, 0x06, 0x00, 0x00,
        192, 168, 1, 1,
        192, 168, 1, 2,
    ];
    
    c.bench_function("parse_ipv4", |b| {
        b.iter(|| parse(black_box(&packet)))
    });
}

fn bench_routing_lookup(c: &mut Criterion) {
    // Setup fabric and routing tables
    // Benchmark lookup time
}

criterion_group!(benches, bench_packet_parse, bench_routing_lookup);
criterion_main!(benches);
```

3. Run benchmarks with:
```bash
cargo bench
```

## Files to Create
- `benches/forwarding_bench.rs`

## Files to Modify
- `Cargo.toml`

## Effort Estimate
Small (1-2 hours)

## Related Plans
- Plan 9: Integration and End-to-End Testing
