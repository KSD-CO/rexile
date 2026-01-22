# ReXile 🦎

**A fast regex-lite engine built on `memchr` and `aho-corasick`**

ReXile is a **zero-dependency regex alternative** (no `regex` crate!) designed for use cases where you need:

- ✅ **Fast literal searches** (using `memchr`)
- ✅ **Multi-pattern matching** (using `aho-corasick`)
- ✅ **Character classes** (`[a-z]`, `[0-9]`, `[^abc]`)
- ✅ **Quantifiers** (`*`, `+`, `?`)
- ✅ **Simple anchoring** (`^start`, `end$`)
- ✅ **Minimal dependencies**
- ✅ **Predictable performance**

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

## 🎯 Purpose

ReXile is a **full-featured regex engine** built from scratch without using the `regex` crate. Goals:

- 🎯 **Complete regex support** - All standard regex features (quantifiers, groups, lookahead, etc.)
- ⚡ **Maximum performance** - Literal-first matching, SIMD acceleration, smart optimizations
- 📦 **Minimal dependencies** - Only `memchr` and `aho-corasick` for low-level primitives
- 🔧 **Full control** - Custom optimizations for parsers, lexers, and rule engines
- 🚀 **Better compilation speed** - 100x faster pattern compilation than `regex` crate

**Current Status:** Phase 2 complete (character classes + quantifiers working!)

**Current Status:**
- ✅ Phase 0: Literals, alternation, anchors
- ✅ Phase 1: Character classes `[a-z]`, `[0-9]`, `[^abc]`
- ✅ Phase 2: Quantifiers `*`, `+`, `?`
- ✅ Phase 3: Escape sequences `\d`, `\w`, `\s`, etc.
- ✅ Phase 4: Sequences `ab+c*`, `\d+\w*`
- ✅ Phase 5: Groups `(abc)`, `(?:...)`, `(foo|bar)+` (basic)
- ✅ Phase 6: Word boundaries `\b`, `\B` (zero-width assertions)
- 🔄 Phase 7+: Lookahead, lookbehind, captures (see future roadmap)

## 🚀 Quick Start

```rust
use rexile::ReXile;  // or use rexile::Pattern

// Literal matching
let pattern = ReXile::new("hello").unwrap();
assert!(pattern.is_match("hello world"));
assert_eq!(pattern.find("say hello"), Some((4, 9)));

// Multi-pattern matching (fast!)
let multi = ReXile::new("foo|bar|baz").unwrap();
assert!(multi.is_match("the bar is open"));

// Character classes
let digits = ReXile::new("[0-9]+").unwrap();
let matches = digits.find_all("Order #12345 costs $67.89");
// Returns: [(7, 12), (20, 22), (23, 25)]

// Escape sequences
let word = ReXile::new("\\w+").unwrap();
assert!(word.is_match("hello123"));

// Sequences
let pattern = ReXile::new("ab+c*").unwrap();
assert!(pattern.is_match("abbcc"));

// Groups and alternation
let protocol = ReXile::new("(http|https|ftp)").unwrap();
assert!(protocol.is_match("http://example.com"));

// Anchors
let exact = ReXile::new("^hello$").unwrap();
assert!(exact.is_match("hello"));
assert!(!exact.is_match("hello world"));

// Word boundaries (Phase 6!)
let boundary = ReXile::new("\\b").unwrap();
let text = "hello world";
let boundaries = boundary.find_all(text);
// Returns: [(0,0), (5,5), (6,6), (11,11)] - all word boundary positions
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
| Character classes | `[a-z]`, `[0-9]`, `[^abc]` | ✅ Supported (Phase 1) |
| Basic quantifiers | `*`, `+`, `?` | ✅ Supported (Phase 2) |
| Escape sequences | `\d`, `\w`, `\s`, `\.` | ✅ Supported (Phase 3) |
| Sequences | `ab+c*`, `\d+\w*` | ✅ Supported (Phase 4) |
| Groups | `(abc)`, `(?:...)`, `(foo\|bar)` | ✅ Supported (Phase 5, basic) |
| Quantified groups | `(ab)+`, `(xyz)*` | ✅ Supported (Phase 5) |
| Bounded quantifiers | `{n}`, `{n,m}` | 🚧 Phase 2b |
| Capturing groups | `(group)` extraction | 🚧 Phase 5b |
| Word boundaries | `\b`, `\B` | ✅ Phase 6 |
| Lookahead/lookbehind | `(?=...)`, `(?<=...)` | 🚧 Phase 7 |
| Backreferences | `\1`, `\2` | 🚧 Phase 8 |
| Unicode properties | `\p{L}`, `\p{N}` | 🚧 Phase 9 |

## 🔄 Full Regex Engine Roadmap

ReXile is being built into a **complete regex engine** from scratch! We're taking a phased approach:

**✅ COMPLETED:**
- ✅ Phase 0: Literals, alternation, anchors
- ✅ Phase 1: Character classes with ASCII bitmap optimization
- ✅ Phase 2: Basic quantifiers with greedy backtracking

**🚀 IN PROGRESS:**
- 🔄 **Phase 3** - Escape sequences (`\d`, `\w`, `\s`, `\.`, `\\`, `\n`, `\t`)
- 🔄 **Phase 4** - Sequences and grouping (`ab+c*`, `(a|b)`, `(?:...)`)
- 🔄 **Phase 5** - Capturing groups with extraction API

**📋 PLANNED:**
- Phase 6: Word boundaries (`\b`, `\B`) ✅
- Phase 7: Assertions (lookahead/lookbehind)
- Phase 8: Backreferences
- Phase 9: Unicode support
- Phase 10: DFA compilation & optimizations

See [ROADMAP_FULL_REGEX.md](ROADMAP_FULL_REGEX.md) for the complete 4-8 week implementation plan.

**Why build from scratch?** Maximum performance, minimal dependencies, and full control over optimizations like literal-first matching and SIMD acceleration.

## 📊 Performance

ReXile is built on:

- **`memchr`** - SIMD-accelerated substring search (faster than naive loops)
- **`aho-corasick`** - Efficient multi-pattern matching (faster than multiple regex alternations)
- **ASCII bitmap optimization** - Fast character class matching

### Benchmark Results vs regex Crate

| Scenario | ReXile | regex | Winner |
|----------|--------|-------|--------|
| **Multi-pattern (2)** | 16ns | 21ns | ✅ ReXile **1.3x faster** |
| **Multi-pattern (4+)** | 18-25ns | 35-50ns | ✅ ReXile **~2x faster** |
| **Multi-pattern (10+)** | 40-80ns | 200-400ns | ✅ ReXile **5-10x faster** |
| Literal search | 14ns | 12ns | regex 1.2x faster |
| **Compilation (literal)** | <100ns | ~10µs | ✅ ReXile **100x faster** |
| **Compilation (multi)** | 1-2µs | 20-50µs | ✅ ReXile **20x faster** |

*Benchmarks on x86_64 Linux. See [BENCHMARK_COMPARISON.md](BENCHMARK_COMPARISON.md) for full analysis.*

### Key Takeaways

- ✅ **ReXile excels at multi-pattern matching** - advantage grows with more patterns
- ✅ **Dramatically faster compilation** - critical for dynamic pattern creation
- ✅ **Competitive single-literal performance** - within 20% of regex
- ✅ **Character classes with quantifiers work great** - `[0-9]+`, `[a-z]*`

**🔬 Run benchmarks yourself:**
```bash
# Quick comparison (3-5 minutes)
cargo bench --bench quick_comparison --manifest-path /path/to/rexile/Cargo.toml

# Full analysis (10-20 minutes)
cargo bench --bench comparison_benchmark --manifest-path /path/to/rexile/Cargo.toml
```

**📈 Full benchmark docs:** See [benches/README.md](benches/README.md) and [BENCHMARK_COMPARISON.md](BENCHMARK_COMPARISON.md)

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

**📚 More examples:** See [examples/](examples/) directory for:
- [`basic_usage.rs`](examples/basic_usage.rs) - Core API walkthrough
- [`parser_lexer.rs`](examples/parser_lexer.rs) - Using ReXile in parsers
- [`log_processing.rs`](examples/log_processing.rs) - Log analysis patterns
- [`performance.rs`](examples/performance.rs) - Performance comparison

Run examples with:
```bash
cargo run --example basic_usage
cargo run --example log_processing
```

## 🔧 Use Cases

ReXile is ideal for:

- **Parsers and lexers** - Fast token matching without regex overhead
- **Rule engines** - Simple pattern matching in business rules
- **Log processing** - Find keywords and patterns in logs
- **Configuration parsing** - Match simple patterns in config files
- **Migration from regex** - Gradual migration away from regex crate

## 🤝 Contributing

Contributions welcome! ReXile is actively being built into a full regex engine.

**Current priorities:**
- ✅ ~~Character classes~~ (DONE - Phase 1)
- ✅ ~~Basic quantifiers~~ (DONE - Phase 2)
- 🔄 **Escape sequences** (`\d`, `\w`, `\s`) - Phase 3 (HIGH PRIORITY)
- 🔄 **Sequences** (`ab+c*`) - Phase 4
- 🔄 **Grouping** (`(a|b)`, `(?:...)`) - Phase 4
- 📋 **Capturing groups** - Phase 5
- 📋 **Word boundaries** (`\b`) - Phase 6
- 📋 **Lookahead/lookbehind** - Phase 7

See [ROADMAP_FULL_REGEX.md](ROADMAP_FULL_REGEX.md) for full implementation plan.

## 📜 License

Licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## 🙏 Credits

Built on top of:
- [`memchr`](https://docs.rs/memchr) by Andrew Gallant
- [`aho-corasick`](https://docs.rs/aho-corasick) by Andrew Gallant

Inspired by the need for a lightweight regex alternative in the [rust-rule-engine](https://github.com/KSD-CO/rust-rule-engine) project.

---

**Status:** � Active Development - Building Full Regex Engine

- ✅ Phase 0-2 Complete: Literals, alternation, anchors, character classes, quantifiers
- 🔄 Phase 3+ In Progress: Escape sequences, grouping, captures, lookahead, etc.
- 📋 See [ROADMAP_FULL_REGEX.md](ROADMAP_FULL_REGEX.md) for complete 10-phase plan

**Goal:** Feature-complete regex engine with better performance than `regex` crate!
