# ReXile

ReXile — a small, focused crate to help migrate away from ad-hoc `regex` usage in
rust-rule-engine's legacy parser. It's intentionally minimal: a cached-regex
wrapper today, and a place to experiment with faster, literal-first matching
strategies later.

Usage example

```rust
let r = rexile::get_regex("^rule\\s+").unwrap();
assert!(r.is_match("rule foo"));
```

Next steps

- Expand API to support zero-allocation matching helpers
- Add optional global registry for `'static` lifetime reuse
- Add feature flags for advanced backends (literal-search optimized)
