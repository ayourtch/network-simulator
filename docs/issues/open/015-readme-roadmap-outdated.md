# Issue 015: README Roadmap Should Be Updated

## Summary
The README.md contains a roadmap section with future versions (v1.1, v1.2, v2.0) that are not accurate. According to the problem statement, this is the final version and no further versions are planned.

## Location
- File: `README.md`
- Section: `## Roadmap`

## Current Content
```markdown
## Roadmap

- **v1.1**: IPv6 support.
- **v1.2**: GUI front‑end.
- **v2.0**: Distributed simulation.
```

## Expected Behavior
The README should accurately reflect that:
1. This is the final version
2. Any unimplemented features should be documented as known limitations or open issues

## Recommended Solution

Replace the Roadmap section with:

```markdown
## Known Limitations

This is the final version of the network simulator. The following features are documented as open issues but not planned for implementation:

- Full IPv6 packet processing
- ICMPv6 error generation
- Per-packet raw byte forwarding (currently uses PacketMeta)
- Prometheus metrics export
- GUI front-end
- Distributed simulation

See `docs/issues/open/` for detailed issue descriptions and implementation guidance.
```

Alternatively, remove the Roadmap section entirely and reference the issues documentation.

## Files to Modify
- `README.md`

## Effort Estimate
Small (< 1 hour)

## Related Documents
- All open issues in `docs/issues/open/`
