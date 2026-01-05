# Remove Unused Imports in src/tun/mod.rs

**Summary**: The file `src/tun/mod.rs` contains unused imports (`std::net::Ipv4Addr` and `std::os::unix::io::FromRawFd`). These generate compiler warnings and reduce code clarity.

**Location**: `src/tun/mod.rs` near the top of the file.

**Current Behavior**: The imports are present but never referenced in the code, leading to unnecessary compilation warnings.

**Expected Behavior**: Unused imports should be removed to keep the codebase clean and warning‑free.

**Recommended Solution**: Delete the lines:
```rust
use std::net::Ipv4Addr;
use std::os::unix::io::{FromRawFd};
```
Commit the changes after removal.

**Files to Modify**:
- `src/tun/mod.rs`

**Effort Estimate**: Small (under 15 minutes).

**Dependencies**: None.
