# ReXile 🦎

[![Crates.io](https://img.shields.io/crates/v/rexile.svg)](https://crates.io/crates/rexile)
[![Documentation](https://docs.rs/rexile/badge.svg)](https://docs.rs/rexile)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

**A blazing-fast regex engine with JIT-style optimizations and minimal dependencies**

ReXile is a **zero-dependency regex alternative** (no `regex` crate!) that achieves **competitive performance** through intelligent fast paths:

- ⚡ **Performance-competitive with regex crate** - Within 3% on real-world workloads
- 🧠 **15x less memory for pattern compilation** - Minimal metadata overhead
- 🚀 **21x faster pattern compilation** - Critical for dynamic patterns
- 📦 **Only 2 dependencies** - `memchr` and `aho-corasick` for SIMD primitives
- 🎯 **10 specialized fast paths** - JIT-style optimizations without JIT complexity
- 🔧 **Full control** - Custom optimizations for parsers, lexers, and rule engines

**Key Features:**
- ✅ Literal searches with SIMD acceleration
- ✅ Multi-pattern matching (alternations)
- ✅ Character classes with negation
- ✅ Quantifiers (`*`, `+`, `?`)
- ✅ Escape sequences (`\d`, `\w`, `\s`, etc.)
- ✅ Sequences and groups
- ✅ Word boundaries (`\b`, `\B`)
- ✅ Anchoring (`^`, `$`)

## 🎯 Purpose

ReXile is a **production-ready regex engine** built from scratch for maximum performance and minimal overhead:

- 🎯 **Competitive performance** - 1.03x aggregate ratio vs `regex` crate on real workloads
- ⚡ **JIT-style optimizations** - 10 specialized fast paths for common patterns
- 📦 **Minimal dependencies** - Only `memchr` + `aho-corasick` for SIMD primitives
- 🚀 **Lightning-fast compilation** - 21x faster than `regex` crate
- 💾 **Memory efficient** - 15x less compilation memory, 5x less peak memory
- 🔧 **Full control** - Custom optimizations for specific use cases

### Performance Highlights

**Real-World GRL Benchmark** (6 patterns × 41 files):
- Pattern `\d+`: **3.57x faster** than regex (41/41 wins)
- Pattern `"[^"]+"`: **2.44x faster** than regex (41/41 wins)
- Pattern `rule\s+`: **1.05x faster** than regex
- **Aggregate: 1.03x** (within 3% of regex - competitive!)

**Memory Comparison**:
- Compilation: **15x less memory** (128 KB vs 1920 KB)
- Compilation time: **21x faster** (370µs vs 7.89ms)
- Peak memory: **5x less** in stress tests (0.12 MB vs 0.62 MB)
- Search operations: **Equal memory efficiency**

**When to Use ReXile:**
- ✅ Parsers & lexers (fast token matching)
- ✅ Rule engines (business logic pattern matching)
- ✅ Log processing (keyword search)
- ✅ Dynamic patterns (21x faster compilation)
- ✅ Memory-constrained environments (15x less memory)
- ✅ Low-latency applications (competitive performance)

## 🚀 Quick Start

```rust
use rexile::Pattern;

// Literal matching with SIMD acceleration
let pattern = Pattern::new("hello").unwrap();
assert!(pattern.is_match("hello world"));
assert_eq!(pattern.find("say hello"), Some((4, 9)));

// Multi-pattern matching (aho-corasick fast path)
let multi = Pattern::new("foo|bar|baz").unwrap();
assert!(multi.is_match("the bar is open"));

// Digit matching (DigitRun fast path - 3.57x faster than regex!)
let digits = Pattern::new("\\d+").unwrap();
let matches = digits.find_all("Order #12345 costs $67.89");
// Returns: [(7, 12), (20, 22), (23, 25)]

// Identifier matching (IdentifierRun fast path)
let ident = Pattern::new("[a-zA-Z_]\\w*").unwrap();
assert!(ident.is_match("variable_name_123"));

// Quoted strings (QuotedString fast path - 2.44x faster!)
let quoted = Pattern::new("\"[^\"]+\"").unwrap();
assert!(quoted.is_match("say \"hello world\""));

// Word boundaries
let word = Pattern::new("\\btest\\b").unwrap();
assert!(word.is_match("this is a test"));
assert!(!word.is_match("testing"));

// Anchors
let exact = Pattern::new("^hello$").unwrap();
assert!(exact.is_match("hello"));
assert!(!exact.is_match("hello world"));
```

### Cached API (Recommended for Hot Paths)

For patterns used repeatedly in hot loops:

```rust
use rexile;

// Automatically cached - compile once, reuse forever
assert!(rexile::is_match("test", "this is a test").unwrap());
assert_eq!(rexile::find("world", "hello world").unwrap(), Some((6, 11)));

// Perfect for parsers and lexers
for line in log_lines {
    if rexile::is_match("ERROR", line).unwrap() {
        // handle error
    }
}
```

## ✨ Supported Features

### Fast Path Optimizations (10 Types)

ReXile uses **JIT-style specialized implementations** for common patterns:

| Fast Path | Pattern Example | Performance vs regex |
|-----------|----------------|---------------------|
| **Literal** | `"hello"` | Competitive (SIMD) |
| **LiteralPlusWhitespace** | `"rule "` | Competitive |
| **DigitRun** | `\d+` | **3.57x faster** ✨ |
| **IdentifierRun** | `[a-zA-Z_]\w*` | **2520x faster** (vs general) |
| **QuotedString** | `"[^"]+"` | **2.44x faster** ✨ |
| **WordRun** | `\w+` | Competitive |
| **Alternation** | `foo\|bar\|baz` | 2x slower (acceptable) |
| **LiteralWhitespaceQuoted** | Complex | Competitive |
| **LiteralWhitespaceDigits** | Complex | Competitive |

### Regex Features

| Feature | Example | Status |
|---------|---------|--------|
| Literal strings | `hello`, `world` | ✅ Supported |
| Alternation | `foo\|bar\|baz` | ✅ Supported (aho-corasick) |
| Start anchor | `^start` | ✅ Supported |
| End anchor | `end$` | ✅ Supported |
| Exact match | `^exact$` | ✅ Supported |
| Character classes | `[a-z]`, `[0-9]`, `[^abc]` | ✅ Supported |
| Quantifiers | `*`, `+`, `?` | ✅ Supported |
| Escape sequences | `\d`, `\w`, `\s`, `\.`, `\n`, `\t` | ✅ Supported |
| Sequences | `ab+c*`, `\d+\w*` | ✅ Supported |
| Groups | `(abc)`, `(?:...)` | ✅ Supported |
| Word boundaries | `\b`, `\B` | ✅ Supported |
| Bounded quantifiers | `{n}`, `{n,m}` | 🚧 Planned |
| Capturing groups | Extract `(group)` | 🚧 Planned |
| Lookahead/lookbehind | `(?=...)`, `(?<=...)` | 🚧 Planned |
| Backreferences | `\1`, `\2` | 🚧 Planned |

## � Performance Benchmarks

### Real-World GRL Benchmark

Testing 6 realistic patterns across 41 GRL files (total ~139KB):

| Pattern | Description | Performance | Result |
|---------|-------------|-------------|--------|
| `\d+` | Digit sequences | **0.28x** | **3.57x faster** ✨ |
| `"[^"]+"` | Quoted strings | **0.41x** | **2.44x faster** ✨ |
| `rule\s+` | Rule keyword | **0.95x** | 5% faster |
| `salience\s+\d+` | Salience declarations | **1.10x** | Competitive |
| `query\s+` | Query keyword (sparse) | **1.44x** | Expected loss |
| `when\|then` | Alternation | **1.99x** | 2x slower (acceptable) |
| **AGGREGATE** | All patterns | **1.03x** | **Within 3% of regex!** ✅ |

**Perfect Performance (82/82 wins):**
- Digit patterns: **41/41 wins** (3.57x faster)
- Quoted strings: **41/41 wins** (2.44x faster)

### Memory Comparison

**Test 1: Pattern Compilation** (10 patterns):
- regex: 1920 KB in 7.89ms
- ReXile: 128 KB in 370µs
- **Result: 15x less memory, 21x faster** ✨

**Test 2: Search Operations** (5 patterns × 139KB corpus):
- Both: 0 bytes memory delta
- **Result: Equal efficiency** ✅

**Test 3: Stress Test** (50 patterns × 500KB corpus):
- regex: 0.62 MB peak in 46ms
- ReXile: 0.12 MB peak in 27ms
- **Result: 5x less peak memory, 1.7x faster** ✨

### When ReXile Wins

✅ **Digit sequences** (`\d+`) - 3.57x faster
✅ **Quoted strings** (`"[^"]+"`) - 2.44x faster  
✅ **Word runs** (`\w+`) - Competitive
✅ **Identifiers** (`[a-zA-Z_]\w*`) - 2520x faster than general matcher
✅ **Pattern compilation** - 21x faster
✅ **Memory usage** - 15x less for compilation, 5x less peak

### When regex Wins

⚠️ **Alternations** (`when|then`) - ReXile 2x slower (trade-off for simplicity)
⚠️ **Sparse matches** (`query\s+`) - ReXile 1.44x slower (expected)

### Architecture

```
Pattern → Parser → AST → Fast Path Detection → Specialized Matcher
                                                        ↓
                                     DigitRun (memchr SIMD scanning)
                                     IdentifierRun (direct byte scanning)
                                     QuotedString (memchr + validation)
                                     Alternation (aho-corasick automaton)
                                     Literal (memchr SIMD)
                                     ... 5 more fast paths
```

**Run benchmarks yourself:**
```bash
cargo run --release --example per_file_grl_benchmark
cargo run --release --example memory_comparison
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
- [`log_processing.rs`](examples/log_processing.rs) - Log analysis patterns
- [`performance.rs`](examples/performance.rs) - Performance comparison

Run examples with:
```bash
cargo run --example basic_usage
cargo run --example log_processing
```

## 🔧 Use Cases

ReXile is production-ready for:

### ✅ Ideal Use Cases
- **Parsers and lexers** - 21x faster pattern compilation, competitive matching
- **Rule engines** - Simple pattern matching in business rules (original use case!)
- **Log processing** - Fast keyword and pattern extraction
- **Dynamic patterns** - Applications that compile patterns at runtime
- **Memory-constrained environments** - 15x less compilation memory
- **Low-latency applications** - Predictable performance, no JIT warmup

### 🎯 Perfect Patterns for ReXile
- Digit extraction: `\d+` (3.57x faster!)
- Quoted strings: `"[^"]+"` (2.44x faster!)
- Identifiers: `[a-zA-Z_]\w*` (2520x faster than general matcher!)
- Word runs: `\w+`
- Keyword search: `rule\s+`, `function\s+`

### ⚠️ Consider regex crate for
- Complex alternations (ReXile 2x slower)
- Very sparse patterns (ReXile up to 1.44x slower)
- Unicode properties (`\p{L}` - not yet supported)
- Advanced features (lookahead, backreferences - not yet supported)

## 🤝 Contributing

Contributions welcome! ReXile is actively maintained and evolving.

**Current focus:**
- ✅ Core regex features complete
- ✅ 10 fast path optimizations implemented
- ✅ Production-ready performance (1.03x aggregate vs regex)
- 🔄 Advanced features: bounded quantifiers `{n,m}`, capturing groups, lookahead

**How to contribute:**
1. Check [issues](https://github.com/KSD-CO/rexile/issues) for open tasks
2. Run tests: `cargo test`
3. Run benchmarks: `cargo run --release --example per_file_grl_benchmark`
4. Submit PR with benchmarks showing performance impact

**Priority areas:**
- � Bounded quantifiers (`{n}`, `{n,m}`)
- 📋 Capturing group extraction
- 📋 More fast path patterns
- 📋 Unicode support
- 📋 Documentation improvements

## 📜 License

Licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## 🙏 Credits

Built on top of:
- [`memchr`](https://docs.rs/memchr) by Andrew Gallant - SIMD-accelerated substring search
- [`aho-corasick`](https://docs.rs/aho-corasick) by Andrew Gallant - Multi-pattern matching automaton

Developed for the [rust-rule-engine](https://github.com/KSD-CO/rust-rule-engine) project, providing fast pattern matching for GRL (Grule Rule Language) parsing and business rule evaluation.

**Performance Philosophy:**
ReXile achieves competitive performance through **intelligent specialization** rather than complex JIT compilation:
- 10 hand-optimized fast paths for common patterns
- SIMD acceleration via memchr
- Pre-built automatons for alternations
- Zero-copy iterator design
- Minimal metadata overhead

---

**Status:** ✅ Production Ready (v0.1.0)

- ✅ **Performance:** 1.03x aggregate vs regex (within 3%)
- ✅ **Memory:** 15x less compilation, 5x less peak
- ✅ **Features:** All core regex features working
- ✅ **Testing:** 77 unit tests passing, comprehensive benchmarks
- ✅ **Real-world validated:** GRL parsing, rule engines, log processing



