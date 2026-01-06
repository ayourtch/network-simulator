# Return Semicolon Bug Fact

A stray semicolon after a `Ok(...)` or `Err(...)` return value in Rust causes the function to return the unit type `()` instead of the intended `Result`. This leads to mismatched type errors like `expected Result<..., &str>, found ()`. Removing the semicolon restores the correct return type.
