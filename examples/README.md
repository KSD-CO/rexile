# Rexile Examples

Essential examples demonstrating rexile's features and use cases.

## Core Examples

### 1. [basic_usage.rs](basic_usage.rs)
Quick start guide showing core API:
- Pattern compilation
- Basic matching (is_match, find, find_all)
- Character classes
- Quantifiers
- Word boundaries

Run: `cargo run --example basic_usage`

### 2. [production_ready_test.rs](production_ready_test.rs)
Comprehensive feature test validating all supported patterns:
- ✅ 70+ test cases covering all features
- Character classes, quantifiers, anchors
- Lookahead assertions
- Case-insensitive matching
- Word boundaries
- Practical patterns (email, IP, HTTP methods)

Run: `cargo run --example production_ready_test`

### 3. [log_processing.rs](log_processing.rs)
Real-world use case: log file parsing and analysis
- ERROR/WARN level detection
- IP address extraction
- Timestamp matching
- Multi-pattern search

Run: `cargo run --example log_processing`

## Performance Examples

### 4. [performance.rs](performance.rs)
Performance comparison between rexile and regex crate
- Compilation speed benchmarks
- Matching performance tests
- Memory usage comparison

Run: `cargo run --release --example performance`

### 5. [perf_compare.rs](perf_compare.rs)
Side-by-side performance comparison for common patterns

Run: `cargo run --release --example perf_compare`

### 6. [perf_micro.rs](perf_micro.rs)
Micro-benchmarks for specific pattern types

Run: `cargo run --release --example perf_micro`

## Utilities

### 7. [check_patterns.rs](check_patterns.rs)
Pattern validation and testing utility
- Validate pattern syntax
- Test pattern matching
- Debug pattern compilation

Run: `cargo run --example check_patterns`

---

## Quick Start

```rust
use rexile::Pattern;

// Basic matching
let pattern = Pattern::new(r"\d+").unwrap();
assert!(pattern.is_match("Order #12345"));

// Find matches
if let Some((start, end)) = pattern.find("Item 123") {
    println!("Found at {}..{}", start, end);
}

// Find all matches
let matches = pattern.find_all("123 and 456");
// Returns: [(0, 3), (8, 11)]
```

## Features Demonstrated

- ⚡ 10-100x faster compilation than regex crate
- 🎯 Competitive matching performance
- 📦 Minimal dependencies (memchr + aho-corasick)
- 🔧 Perfect for parsers, DSLs, rule engines

See [README.md](../README.md) for full feature list and documentation.
