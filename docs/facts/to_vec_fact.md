# to_vec vs iter().cloned().collect() Fact

Using `.to_vec()` on a slice is more idiomatic and faster than `iter().cloned().collect()`. It directly clones the slice elements into a new `Vec`, avoiding the extra iterator allocation. This change resolves Clippy's `iter_cloned_collect` warning.
