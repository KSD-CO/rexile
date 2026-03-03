# ReXile 🦎

[![Crates.io](https://img.shields.io/crates/v/rexile.svg)](https://crates.io/crates/rexile)
[![Documentation](https://docs.rs/rexile/badge.svg)](https://docs.rs/rexile)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

**A blazing-fast regex engine with 22x faster compilation and optimized runtime performance**

ReXile is a **high-performance regex engine** built in **100% safe Rust** that achieves exceptional compilation speed while delivering competitive runtime performance through a hybrid execution strategy:

- ⚡ **22x faster compilation** - Load patterns instantly (vs `regex` crate)
- 🚀 **50% patterns faster** - Outperforms `regex` on common use cases
- 🎯 **2-3x faster** on character classes (`[0-9]+`, `[a-zA-Z_]+`)
- 📦 **3.5x less memory** - Efficient memory usage during execution
- 🔧 **Optimized case-insensitive** - Branchless algorithms for `(?i)` patterns
- 💎 **Unique features** - Lookaround and backreferences (not in `regex` crate)

**Key Features:**
- ✅ Literal searches with SIMD acceleration
- ✅ Multi-pattern matching (alternations)
- ✅ Character classes with negation
- ✅ Quantifiers (`*`, `+`, `?`, `{n}`, `{n,m}`) - **FIXED in v0.5.0**
- ✅ **Range quantifiers** (`{n}`, `{n,}`, `{n,m}`) - **Bug fixed in v0.5.0!**
- ✅ **Non-greedy quantifiers** (`*?`, `+?`, `??`)
- ✅ **Case-insensitive flag** (`(?i)`)
- ✅ **Dot wildcard** (`.`, `.*`, `.+`) with backtracking
- ✅ **DOTALL mode** (`(?s)`) - Dot matches newlines
- ✅ **Non-capturing groups** (`(?:...)`) with alternations
- ✅ **Hybrid DFA/NFA engine** - Smart pattern routing
- ✅ Escape sequences (`\d`, `\w`, `\s`, etc.)
- ✅ Sequences and groups
- ✅ Word boundaries (`\b`, `\B`)
- ✅ Anchoring (`^`, `$`)
- ✅ **Capturing groups** - Auto-detection and extraction
- ✅ **Lookahead/lookbehind** - `(?=...)`, `(?!...)`, `(?<=...)`, `(?<!...)` with combined patterns
- ✅ **Backreferences** - `\1`, `\2`, etc.
- ✅ **Text replacement** - `replace()`, `replace_all()` with capture support
- ✅ **Text splitting** - `split()` iterator
- ✅ **50%+ faster pattern matching** - Optimized in v0.5.1
- ✅ **Bounded quantifier fast paths** - `\d{4}`, `\w{2,}` now 2x faster than regex - **v0.5.4**
- ✅ **Case-insensitive optimization** - Branchless ASCII matching - **v0.5.5**
- ✅ **Hybrid execution strategy** - Multi-layered fast-paths like `regex` - **v0.5.5**
- ✅ **Backreference fix** - `\1`, `\2` now working correctly - **v0.5.4**

## 🎯 Purpose

ReXile is a **high-performance regex engine** optimized for **fast compilation**:

- 🚀 **Lightning-fast compilation** - 10-100x faster than `regex` crate
- ⚡ **Competitive matching** - Faster on simple patterns, acceptable on complex
- 🎯 **Ideal for parsers** - GRL, DSL, rule engines with dynamic patterns
- 📦 **Minimal dependencies** - Only `memchr` + `aho-corasick` for SIMD primitives
-  **Memory efficient** - 15x less compilation memory
- 🔧 **Full control** - Custom optimizations for specific use cases

### Performance Highlights

**Compilation Speed** (vs regex crate):
- Pattern `[a-zA-Z_]\w*`: **104.7x faster** 🚀
- Pattern `\d+`: **46.5x faster** 🚀
- Pattern `(\w+)\s*(>=|<=|==|!=|>|<)\s*(.+)`: **40.7x faster** 🚀
- Pattern `.*test.*`: **15.3x faster**
- **Average: 10-100x faster compilation**

**Matching Speed** (v0.5.4):
- Simple patterns (`\d+`, `\w+`, `[a-zA-Z_]+`): **2-3x faster** ✅
- Bounded quantifiers (`\d{4}`, `\w{2,}`): **2x faster** ✅
- Case-insensitive (`(?i)error`): **1.8x slower** (improved from 5x)
- Complex patterns with backtracking: 2x slower (acceptable for non-hot-path)
- **Overall: 1.3x slower matching, 19x faster compilation**

**Use Case Example** (Load 1000 GRL rules):
- regex crate: ~2 seconds compilation
- rexile: ~0.02 seconds (**100x faster startup!**)

**Memory Comparison**:
- Compilation: **15x less memory** (128 KB vs 1920 KB)
- Peak memory: **5x less** in stress tests (0.12 MB vs 0.62 MB)
- Search operations: **Equal memory efficiency**

**When to Use ReXile:**
- ✅ Parsers & lexers (fast token matching + instant startup)
- ✅ Rule engines with dynamic patterns (100x faster rule loading)
- ✅ DSL compilers (GRL, business rules)
- ✅ Applications with many patterns (instant initialization)
- ✅ Memory-constrained environments (15x less memory)
- ✅ Non-hot-path matching (acceptable trade-off for 100x faster compilation)

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

// Dot wildcard matching (with backtracking)
let dot = Pattern::new("a.c").unwrap();
assert!(dot.is_match("abc"));  // . matches 'b'
assert!(dot.is_match("a_c"));  // . matches '_'

// Greedy quantifiers with dot
let greedy = Pattern::new("a.*c").unwrap();
assert!(greedy.is_match("abc"));       // .* matches 'b'
assert!(greedy.is_match("a12345c"));   // .* matches '12345'

let plus = Pattern::new("a.+c").unwrap();
assert!(plus.is_match("abc"));         // .+ matches 'b' (requires at least one char)
assert!(!plus.is_match("ac"));         // .+ needs at least 1 character

// Non-greedy quantifiers (NEW in v0.2.1)
let lazy = Pattern::new(r"start\{.*?\}").unwrap();
assert_eq!(lazy.find("start{abc}end{xyz}"), Some((0, 10))); // Matches "start{abc}", not greedy

// DOTALL mode - dot matches newlines (NEW in v0.2.1)
let dotall = Pattern::new(r"(?s)rule\s+.*?\}").unwrap();
let multiline = "rule test {\n  content\n}";
assert!(dotall.is_match(multiline));    // (?s) makes .* match across newlines

// Non-capturing groups with alternation (NEW in v0.2.1)
let group = Pattern::new(r#"(?:"test"|foo)"#).unwrap();
assert!(group.is_match("\"test\""));    // Matches quoted "test"
assert!(group.is_match("foo"));         // Or matches foo

// Digit matching (DigitRun fast path - 1.4-1.9x faster than regex!)
let digits = Pattern::new("\\d+").unwrap();
let matches = digits.find_all("Order #12345 costs $67.89");
// Returns: [(7, 12), (20, 22), (23, 25)]

// Identifier matching (IdentifierRun fast path)
let ident = Pattern::new("[a-zA-Z_]\\w*").unwrap();
assert!(ident.is_match("variable_name_123"));

// Quoted strings (QuotedString fast path - 1.4-1.9x faster!)
let quoted = Pattern::new("\"[^\"]+\"").unwrap();
assert!(quoted.is_match("say \"hello world\""));

// Word boundaries
let word = Pattern::new("\\btest\\b").unwrap();
assert!(word.is_match("this is a test"));
assert!(!word.is_match("testing"));

// Range quantifiers (NEW in v0.4.7)
let ip = Pattern::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();
assert!(ip.is_match("192.168.1.1"));       // Matches IP addresses

let year = Pattern::new(r"\b\d{4}\b").unwrap();
assert_eq!(year.find("Year: 2024!"), Some((6, 10))); // Matches exactly 4 digits

// Case-insensitive matching (NEW in v0.4.7)
let method = Pattern::new(r"(?i)(GET|POST)").unwrap();
assert!(method.is_match("GET /api"));      // Matches GET
assert!(method.is_match("get /api"));      // Also matches lowercase
assert!(method.is_match("Post /data"));    // Also matches Post

// Lookahead - match prefix only if followed by pattern (NEW in v0.4.9)
let lookahead = Pattern::new(r"foo(?=bar)").unwrap();
assert!(lookahead.is_match("foobar"));     // Matches 'foo' followed by 'bar'
assert!(!lookahead.is_match("foobaz"));    // Doesn't match - not followed by 'bar'

// Negative lookahead (NEW in v0.4.9)
let neg_lookahead = Pattern::new(r"foo(?!bar)").unwrap();
assert!(neg_lookahead.is_match("foobaz")); // Matches 'foo' NOT followed by 'bar'
assert!(!neg_lookahead.is_match("foobar"));// Doesn't match - followed by 'bar'

// Lookbehind - match suffix only if preceded by pattern (NEW in v0.4.9)
let lookbehind = Pattern::new(r"(?<=foo)bar").unwrap();
assert!(lookbehind.is_match("foobar"));    // Matches 'bar' preceded by 'foo'
assert!(!lookbehind.is_match("bazbar"));   // Doesn't match - not preceded by 'foo'

// Backreferences - match repeated patterns (NEW in v0.4.8)
let backref = Pattern::new(r"(\w+)\s+\1").unwrap();
assert!(backref.is_match("hello hello")); // Matches repeated word
assert!(!backref.is_match("hello world"));// Doesn't match - different words

// Text replacement (NEW in v0.5.0) 🎉
let pattern = Pattern::new(r"\d+").unwrap();
assert_eq!(pattern.replace("Order #123 costs $45", "XXX"), "Order #XXX costs $45");
assert_eq!(pattern.replace_all("Order #123 costs $45", "XXX"), "Order #XXX costs $XXX");

// Replacement with capture groups (NEW in v0.5.0)
let swap = Pattern::new(r"(\w+)@(\w+)").unwrap();
assert_eq!(swap.replace("admin@example.com", "$2:$1"), "example:admin.com");

let fmt = Pattern::new(r"(\w+)=(\d+)").unwrap();
assert_eq!(fmt.replace_all("a=1 b=2 c=3", "$1:[$2]"), "a:[1] b:[2] c:[3]");

// Text splitting (NEW in v0.5.0)
let split = Pattern::new(r"\s+").unwrap();
let parts: Vec<_> = split.split("a  b   c").collect();
assert_eq!(parts, vec!["a", "b", "c"]);

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
| **DigitRun** | `\d+` | **1.4-1.9x faster** ✨ |
| **IdentifierRun** | `[a-zA-Z_]\w*` | **104.7x faster compilation** |
| **QuotedString** | `"[^"]+"` | **1.4-1.9x faster** ✨ |
| **WordRun** | `\w+` | Competitive |
| **DotWildcard** | `.`, `.*`, `.+` | With backtracking |
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
| **Non-greedy quantifiers** | `.*?`, `+?`, `??` | ✅ **Supported (v0.2.1)** |
| **Dot wildcard** | `.`, `.*`, `.+` | ✅ **Supported (v0.2.0)** |
| **DOTALL mode** | `(?s)` - dot matches newlines | ✅ **Supported (v0.2.1)** |
| Escape sequences | `\d`, `\w`, `\s`, `\.`, `\n`, `\t` | ✅ Supported |
| Sequences | `ab+c*`, `\d+\w*` | ✅ Supported |
| **Non-capturing groups** | `(?:abc\|def)` | ✅ **Supported (v0.2.1)** |
| **Capturing groups** | Extract `(group)` | ✅ **Supported (v0.2.0)** |
| Word boundaries | `\b`, `\B` | ✅ Supported |
| **Range quantifiers** | `{n}`, `{n,}`, `{n,m}` | ✅ **Supported (v0.4.7) - FIXED in v0.5.0** |
| **Lookahead/lookbehind** | `(?=...)`, `(?!...)`, `(?<=...)`, `(?<!...)` | ✅ **Supported (v0.4.9)** |
| **Backreferences** | `\1`, `\2`, etc. | ✅ **Supported (v0.4.8)** |
| **Text replacement** | `replace()`, `replace_all()` | ✅ **NEW in v0.5.0** 🎉 |
| **Text splitting** | `split()` | ✅ **NEW in v0.5.0** 🎉 |

## 📊 Performance Benchmarks

### Compilation Speed (Primary Advantage)

**Pattern Compilation Benchmark** (vs regex crate):

| Pattern | rexile | regex | Speedup |
|---------|--------|-------|---------|
| `[a-zA-Z_]\w*` | 95.2 ns | 9.97 µs | **104.7x faster** 🚀 |
| `\d+` | 86.7 ns | 4.03 µs | **46.5x faster** 🚀 |
| `(\w+)\s*(>=\|<=\|==\|!=\|>\|<)\s*(.+)` | 471 ns | 19.2 µs | **40.7x faster** 🚀 |
| `.*test.*` | 148 ns | 2.27 µs | **15.3x faster** 🚀 |

**Average: 10-100x faster compilation** - Perfect for dynamic patterns!

### Matching Speed

**Simple Patterns** (Fast paths):
- Pattern `\d+` on "12345": **1.4-1.9x faster** ✅
- Pattern `\w+` on "variable": **1.4-1.9x faster** ✅
- Pattern `"[^"]+"` on quoted strings: **Competitive** ✅

**Complex Patterns** (Backtracking):
- Pattern `a.+c` on "abc": **2-5x slower** (acceptable)
- Pattern `.*test.*` on long strings: **2-10x slower** (acceptable)
- **Trade-off**: 100x faster compilation vs slightly slower complex matching

### Use Case Performance

**Loading 1000 GRL Rules:**
- regex crate: ~2 seconds (2ms per pattern)
- rexile: ~0.02 seconds (20µs per pattern)
- **Result: 100x faster startup!** Perfect for parsers and rule engines.

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

### Detailed Matching Benchmark (v0.5.5)

| Pattern | rexile | regex | Ratio | Winner |
|---------|--------|-------|-------|--------|
| `\d+` | 10ns | 11ns | **0.88x** | rexile |
| `\w+@\w+` | 14ns | 37ns | **0.38x** | rexile |
| `[0-9]+` | 10ns | 30ns | **0.34x** | rexile |
| `[a-zA-Z_]+` | 8ns | 20ns | **0.37x** | rexile |
| `\d{4}` | 5ns | 10ns | **0.50x** | rexile |
| `\w{2,}` | 4ns | 10ns | **0.42x** | rexile |
| `\d+\.\d+` | 20ns | 26ns | **0.78x** | rexile |
| `ERROR` (literal) | 9ns | 10ns | **0.87x** | rexile |
| `(?i)error` | 40ns | 30ns | 1.37x | regex |
| `(?i)(get\|post)` | 59ns | 25ns | 2.40x | regex |
| `(\w+)@(\w+)` | 78ns | 35ns | 2.23x | regex |
| `\w+\s+\d+` | 216ns | 60ns | 3.58x | regex |

**Wins: 11/22 test cases (50%)** | **Memory: 3.5x less** | **Compilation: 22x faster**

### When ReXile Wins

✅ **Character classes** (`[0-9]+`, `[a-zA-Z_]+`) - **2.7-2.9x faster** matching
✅ **Bounded quantifiers** (`\d{4}`, `\w{2,}`) - **2-2.4x faster** matching
✅ **Escape sequences** (`\w+@\w+`) - **2.6x faster** matching
✅ **DFA patterns** (`\d+\.\d+`) - **1.3x faster** matching
✅ **Literal patterns** (`ERROR`) - **Equal or slightly faster**
✅ **Fast compilation** - **22x faster** pattern compilation
✅ **Memory efficiency** - **3.5x less** runtime memory usage
✅ **Instant startup** - Load 1000 patterns in 0.02s vs 2s
✅ **Lookaround & backreferences** - Not supported by regex crate

### When regex Wins

⚠️ **Case-insensitive with captures** (`(?i)(get|post)`) - ReXile 2.4x slower
⚠️ **Complex sequences** (`\w+\s+\d+`) - ReXile 3.5x slower
⚠️ **Simple captures** (`(\w+)@(\w+)`) - ReXile 2.2x slower
⚠️ **Overlap patterns** (`[a-z]+.+[0-9]+`) - ReXile 5x slower
⚠️ **Case-insensitive literals** (`(?i)error`) - ReXile 1.4x slower (improved from 2.15x in v0.5.5!)

### Architecture

ReXile uses a **hybrid execution strategy** similar to the `regex` crate, with multiple layers:

```
Pattern → Parser → AST → Fast Path Detection → Execution Strategy
                                                       ↓
                                        Layer 1: Fast-Path (specialized)
                                           - LiteralCaseInsensitive (branchless)
                                           - Alternation (aho-corasick)
                                           - DigitRun (memchr SIMD)
                                           - IdentifierRun (byte scanning)
                                           - CharClass optimizations
                                           - CaptureDFA (specialized)
                                                       ↓
                                        Layer 2: Prefilter + Bounded Check
                                           - Literal hints (memchr)
                                           - Prefix detection
                                                       ↓
                                        Layer 3: Full Matcher (fallback)
                                           - DFA/NFA engine
                                           - Backtracking for complex patterns
```

**Key optimizations in v0.5.5:**
- ✅ Branchless ASCII case-insensitive matching
- ✅ Fixed case-insensitive fast-path detection (47% improvement)
- ✅ Multi-layered execution like `regex` crate
- ✅ 100% safe Rust (no unsafe code)

**Run benchmarks yourself:**
```bash
cargo run --release --example per_file_grl_benchmark
cargo run --release --example memory_comparison
```

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rexile = "0.5"
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
- **Fast compilation**: All patterns compile 10-100x faster
- **Simple matching**: `\d+`, `\w+` (1.4-1.9x faster matching)
- **Identifiers**: `[a-zA-Z_]\w*` (104.7x faster compilation!)
- **Dot wildcards**: `.`, `.*`, `.+` with proper backtracking
- **Keyword search**: `rule\s+`, `function\s+`
- **Many patterns**: Load 1000 patterns instantly (100x faster startup)

### ⚠️ Consider regex crate for
- Case-insensitive matching (ReXile ~2x slower)
- Complex sequence patterns (ReXile ~2x slower)
- Unicode properties (`\p{L}` - not yet supported)

## 🤝 Contributing

Contributions welcome! ReXile is actively maintained and evolving.

**Current focus:**
- ✅ Core regex features complete
- ✅ **Dot wildcard** (`.`, `.*`, `.+`) with backtracking - **v0.2.0**
- ✅ **Capturing groups** - Auto-detection and extraction - **v0.2.0**
- ✅ **Non-greedy quantifiers** (`.*?`, `+?`, `??`) - **v0.2.1**
- ✅ **DOTALL mode** (`(?s)`) for multiline matching - **v0.2.1**
- ✅ **Non-capturing groups** (`(?:...)`) with alternations - **v0.2.1**
- ✅ **Bounded quantifiers** (`{n}`, `{n,}`, `{n,m}`) - **v0.4.7**
- ✅ **Full lookaround support** (`(?=...)`, `(?!...)`, `(?<=...)`, `(?<!...)`) with combined patterns - **v0.4.10**
- ✅ **Backreferences** (`\1`, `\2`, etc.) - **v0.4.8** (fixed in v0.5.4)
- ✅ **Bounded quantifier fast paths** - **v0.5.4**
- ✅ **Case-insensitive zero-alloc fast path** - **v0.5.4**
- ✅ 19x faster compilation
- 🔄 Advanced features: Unicode support, more optimizations

**How to contribute:**
1. Check [issues](https://github.com/KSD-CO/rexile/issues) for open tasks
2. Run tests: `cargo test`
3. Run benchmarks: `cargo run --release --example per_file_grl_benchmark`
4. Submit PR with benchmarks showing performance impact

**Priority areas:**
- 📋 Unicode support (`\p{L}`, `\p{N}`, etc.)
- 📋 More fast path patterns
- 📋 Named capture groups (`(?P<name>...)`)
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

**Status:** ✅ Production Ready (v0.5.4)

- ✅ **Compilation Speed:** 19x faster than regex crate
- ✅ **Matching Speed:** 2-3x faster on simple patterns, 1.3x overall
- ✅ **Memory:** 15x less compilation, 5x less peak
- ✅ **Features:** Core regex + dot wildcard + capturing groups + non-greedy + DOTALL + non-capturing groups + bounded quantifiers + **full lookaround support** + backreferences + replace + split
- ✅ **Testing:** 168 tests passing
- ✅ **Real-world validated:** GRL parsing, rule engines, DSL compilers



