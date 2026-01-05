# Issue 034: Documentation Does Not Match Implementation

## Summary
Several documentation files describe features or architecture that doesn't match the current implementation. This creates confusion for developers trying to understand or extend the system.

## Location
- File: `README.md`
- File: `docs/build_and_run_instructions.md`
- File: `docs/configuration_schema.md`

## Discrepancies Found

### 1. README.md - Dual TUN Architecture
The README describes:
```markdown
## Quick Start
...
### Testing Connectivity
After starting the simulator:
```bash
# Configure TUN interfaces
sudo ip addr add 192.168.100.1/24 dev tunA
sudo ip addr add 192.168.100.2/24 dev tunB
```
```

But the implementation only creates one TUN device (`real_tun.name`), not two separate `tunA` and `tunB` devices.

### 2. Configuration Schema - TUN Interfaces
The documentation may suggest `tun_a` and `tun_b` are created as real devices, but they're only used as labels for the mock packet file mode.

### 3. Command-Line Options
The README mentions:
```
- `--tun-name <NAME>` – Override the real TUN device name.
- `--tun-address <IP>` – Override the real TUN device IPv4 address.
```

But there's only ONE TUN device supported, not the two described in the architecture.

### 4. Missing Documentation for Mock Packet Mode
The mock packet file mode (which is the only fully functional mode) is not well documented. Users may not understand:
- Format of packet files (hex-encoded)
- Output file generation (`_out.txt`)
- Direction detection based on IP prefix

## Recommended Solution

1. Update README.md to accurately describe current capabilities:
```markdown
## Current Limitations

The simulator currently supports two modes:

### Mock Packet Mode (Fully Functional)
- Provide hex-encoded packets in a file
- Packets are processed through the virtual fabric
- Output written to `<filename>_out.txt`
- Useful for testing without root privileges

### Real TUN Mode (Limited)
- Creates a single TUN device
- Packets are read, processed, and written back to the same device
- Does not support the dual-TUN architecture yet (see Issue 028)
```

2. Add documentation for mock packet file format:
```markdown
## Mock Packet File Format

Each line should contain a hex-encoded IP packet:
```
# Comment lines start with #
45000014000000004001xxxx0a0001010a000102
# Above is: IPv4, TTL=64, ICMP, src=10.0.1.1, dst=10.0.1.2
```

Lines are processed sequentially. Output is written to `<input_file>_out.txt`.
```

3. Document current vs planned architecture:
```markdown
## Architecture

### Current Implementation
- Single TUN device mode
- Packets read and written to same device
- Direction detected by source IP prefix

### Planned (Not Yet Implemented)
- Dual TUN device mode (Issue 028)
- Linux namespace support
- Bidirectional traffic between tunA and tunB
```

4. Update configuration documentation to match actual fields.

## Files to Modify
- `README.md` (update to reflect current capabilities)
- `docs/build_and_run_instructions.md` (add mock packet documentation)
- `docs/configuration_schema.md` (update to match actual config)

## Effort Estimate
Small (1-2 hours)

## Related Plans
- Plan 9: Integration and End-to-End Testing
