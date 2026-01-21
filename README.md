# ReXile 🦎# ReXile



**A fast regex-lite engine built on `memchr` and `aho-corasick`**ReXile — a small, focused crate to help migrate away from ad-hoc `regex` usage in

rust-rule-engine's legacy parser. It's intentionally minimal: a cached-regex

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)wrapper today, and a place to experiment with faster, literal-first matching

strategies later.

## 🎯 Purpose

Usage example

ReXile is a **zero-dependency regex alternative** designed for use cases where you need:

```rust

- ✅ **Fast literal searches** (using `memchr`)let r = rexile::get_regex("^rule\\s+").unwrap();

- ✅ **Multi-pattern matching** (using `aho-corasick`)assert!(r.is_match("rule foo"));

- ✅ **Simple anchoring** (`^start`, `end$`)```

- ✅ **Minimal dependencies**

- ✅ **Predictable performance**Next steps



ReXile is **NOT** a full regex replacement. It's intentionally minimal and focused on common patterns used in parsers, lexers, and rule engines.- Expand API to support zero-allocation matching helpers

- Add optional global registry for `'static` lifetime reuse

## 🚀 Quick Start- Add feature flags for advanced backends (literal-search optimized)


```rust
use rexile::Pattern;

// Compile once, reuse many times
let pattern = Pattern::new("hello").unwrap();
assert!(pattern.is_match("hello world"));
assert_eq!(pattern.find("say hello"), Some((4, 9)));

// Multi-pattern matching (fast!)
let multi = Pattern::new("foo|bar|baz").unwrap();
assert!(multi.is_match("the bar is open"));

// Anchors
let exact = Pattern::new("^hello$").unwrap();
assert!(exact.is_match("hello"));
assert!(!exact.is_match("hello world"));
```

### Cached API (Recommended)

For patterns used repeatedly, use the cached API:

```rust
use rexile;

// Automatically cached - compile once, reuse forever
assert!(rexile::is_match("test", "this is a test").unwrap());
assert_eq!(rexile::find("world", "hello world").unwrap(), Some((6, 11)));
```

## ✨ Supported Features (v0.1)

| Feature | Example | Status |
|---------|---------|--------|
| Literal strings | `hello`, `world` | ✅ Supported |
| Alternation | `foo\|bar\|baz` | ✅ Supported (aho-corasick) |
| Start anchor | `^start` | ✅ Supported |
| End anchor | `end$` | ✅ Supported |
| Exact match | `^exact$` | ✅ Supported |
| Character classes | `[a-z]`, `[0-9]` | 🚧 Planned |
| Quantifiers | `*`, `+`, `?` | 🚧 Planned |
| Escape sequences | `\.`, `\d`, `\w` | 🚧 Planned |

## 🚫 NOT Supported (by design)

ReXile intentionally does **NOT** support:

- ❌ Lookahead/lookbehind assertions
- ❌ Backreferences
- ❌ Capturing groups (use plain `find()` instead)
- ❌ Unicode property classes (use explicit ranges)
- ❌ Complex nested patterns

**Why?** These features require a full regex engine. If you need them, use the excellent [`regex`](https://docs.rs/regex) crate instead.

## 📊 Performance

ReXile is built on:

- **`memchr`** - SIMD-accelerated substring search (faster than naive loops)
- **`aho-corasick`** - Efficient multi-pattern matching (faster than multiple regex alternations)

Benchmarks coming soon!

## 🏗️ Architecture

```
Pattern String → Parser → AST → Compiler → Matcher
                                              ↓
                          memchr      ← Literal(String)
                          aho-corasick ← MultiLiteral(AhoCorasick)
                          anchored     ← AnchoredLiteral{...}
```

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rexile = "0.1"
```

## 🎓 Examples

### Literal Search

```rust
let p = Pattern::new("needle").unwrap();
assert!(p.is_match("needle in a haystack"));
assert_eq!(p.find("where is the needle?"), Some((13, 19)));

// Find all occurrences
let matches = p.find_all("needle and needle");
assert_eq!(matches, vec![(0, 6), (11, 17)]);
```

### Multi-Pattern (Alternation)

```rust
// Fast multi-pattern search using aho-corasick
let keywords = Pattern::new("import|export|function|class").unwrap();
assert!(keywords.is_match("export default function"));
```

### Anchored Patterns

```rust
// Must start with pattern
let starts = Pattern::new("^Hello").unwrap();
assert!(starts.is_match("Hello World"));
assert!(!starts.is_match("Say Hello"));

// Must end with pattern
let ends = Pattern::new("World$").unwrap();
assert!(ends.is_match("Hello World"));
assert!(!ends.is_match("World Peace"));

// Exact match
let exact = Pattern::new("^exact$").unwrap();
assert!(exact.is_match("exact"));
assert!(!exact.is_match("not exact"));
```

### Cached API (Best for Repeated Patterns)

```rust
// First call compiles and caches
rexile::is_match("keyword", "find keyword here").unwrap();

// Subsequent calls reuse cached pattern (zero compile cost)
rexile::is_match("keyword", "another keyword").unwrap();
rexile::is_match("keyword", "more keyword text").unwrap();
```

## 🔧 Use Cases

ReXile is ideal for:

- **Parsers and lexers** - Fast token matching without regex overhead
- **Rule engines** - Simple pattern matching in business rules
- **Log processing** - Find keywords and patterns in logs
- **Configuration parsing** - Match simple patterns in config files
- **Migration from regex** - Gradual migration away from regex crate

## 🤝 Contributing

Contributions welcome! This is an early-stage project.

Priority areas:
- Character classes `[a-z]`, `[0-9]`
- Simple quantifiers `*`, `+`, `?`
- Escape sequences `\.`, `\n`, `\t`
- Performance benchmarks vs `regex` crate

## 📜 License

Licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## 🙏 Credits

Built on top of:
- [`memchr`](https://docs.rs/memchr) by Andrew Gallant
- [`aho-corasick`](https://docs.rs/aho-corasick) by Andrew Gallant
- [`once_cell`](https://docs.rs/once_cell) by Aleksey Kladov

Inspired by the need for a lightweight regex alternative in the [rust-rule-engine](https://github.com/KSD-CO/rust-rule-engine) project.

---

**Status:** 🚧 Early Development (v0.1) - API may change

For full regex features, use the [`regex`](https://docs.rs/regex) crate.
