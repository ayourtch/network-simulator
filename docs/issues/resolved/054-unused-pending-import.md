# Issue 054: Unused Import `pending` in src/tun/mod.rs

## Summary
`src/tun/mod.rs` imports `futures::future::pending` but never uses it (the comment notes it is not used). This results in a compiler warning and clutters the code.

## Expected Behavior
- Remove the unused import or use it as intended (e.g., replace the placeholder `pending::<()>().await` with a proper future when no virtual‑customer interval is configured).

## Suggested Solution
1. Delete the line `use futures::future::pending;` if it is truly unnecessary.
2. If the intention was to have a no‑op future when `_vc_interval` is `None`, keep the import and replace the placeholder with `futures::future::pending::<()>` (already used). Ensure the import is actually referenced.
3. Run `cargo clippy` and ensure no warnings remain.

## Acceptance Criteria
- The compiler produces no warnings about an unused import.
- The code builds cleanly with `cargo test`.
