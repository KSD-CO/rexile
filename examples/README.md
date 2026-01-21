# ReXile Examples

This directory contains practical examples demonstrating ReXile's capabilities.

## Running Examples

```bash
# Run all examples
cargo run --example basic_usage
cargo run --example parser_lexer
cargo run --example log_processing
cargo run --example performance

# Or run a specific example
cargo run --example basic_usage
```

## Available Examples

### 1. `basic_usage.rs`
**Purpose:** Introduction to ReXile's core API

Demonstrates:
- Literal pattern matching
- Multi-pattern alternation (`foo|bar|baz`)
- Anchors (`^start`, `end$`, `^exact$`)
- Finding matches with `find()` and `find_all()`
- Cached API usage

**Best for:** Getting started with ReXile

```bash
cargo run --example basic_usage
```

### 2. `parser_lexer.rs`
**Purpose:** Using ReXile in parsers and lexers

Demonstrates:
- Keyword detection in code
- Token classification
- Rule engine pattern matching
- Finding clause positions

**Best for:** Parser and compiler developers

```bash
cargo run --example parser_lexer
```

### 3. `log_processing.rs`
**Purpose:** Processing log files and searching patterns

Demonstrates:
- Log level filtering (ERROR, WARN, INFO)
- Keyword extraction from logs
- Pattern counting and statistics
- Multi-pattern keyword search

**Best for:** Log analysis and monitoring tools

```bash
cargo run --example log_processing
```

### 4. `performance.rs`
**Purpose:** Performance comparison of caching strategies

Demonstrates:
- Uncached pattern compilation (compile every time)
- Pre-compiled patterns (compile once, reuse)
- Global cache API (automatic caching)
- Performance metrics and speedup calculations

**Best for:** Understanding ReXile's performance characteristics

```bash
cargo run --example performance --release
```

## Common Patterns

### Literal Search
```rust
let pattern = Pattern::new("hello").unwrap();
assert!(pattern.is_match("hello world"));
```

### Multi-Pattern Matching
```rust
let keywords = Pattern::new("foo|bar|baz").unwrap();
assert!(keywords.is_match("the bar is open"));
```

### Anchored Matching
```rust
// Start anchor
let starts = Pattern::new("^Hello").unwrap();
assert!(starts.is_match("Hello World"));

// End anchor
let ends = Pattern::new("World$").unwrap();
assert!(ends.is_match("Hello World"));

// Exact match
let exact = Pattern::new("^exact$").unwrap();
assert!(exact.is_match("exact"));
```

### Finding Positions
```rust
let pattern = Pattern::new("needle").unwrap();

// Find first occurrence
if let Some((start, end)) = pattern.find("needle in haystack") {
    println!("Found at: {}-{}", start, end);
}

// Find all occurrences
let matches = pattern.find_all("needle and needle");
// Returns: [(0, 6), (11, 17)]
```

### Cached API (Recommended)
```rust
// Automatically cached - compile once, reuse forever
rexile::is_match("test", "this is a test").unwrap();
rexile::find("world", "hello world").unwrap();
```

## Performance Tips

1. **Pre-compile patterns** when possible:
   ```rust
   let pattern = Pattern::new("foo").unwrap();
   for line in lines {
       pattern.is_match(line);  // ✅ Fast - no recompilation
   }
   ```

2. **Use the cached API** for patterns used across your codebase:
   ```rust
   rexile::is_match("keyword", text).unwrap();  // ✅ Auto-cached
   ```

3. **Avoid compiling in hot loops**:
   ```rust
   // ❌ Bad - recompiles every iteration
   for line in lines {
       Pattern::new("foo").unwrap().is_match(line);
   }
   
   // ✅ Good - compile once
   let pattern = Pattern::new("foo").unwrap();
   for line in lines {
       pattern.is_match(line);
   }
   ```

## Next Steps

- Check out the [benchmarks](../benches/README.md) for detailed performance analysis
- Read the [main README](../README.md) for feature documentation
- Explore the [API documentation](https://docs.rs/rexile)
