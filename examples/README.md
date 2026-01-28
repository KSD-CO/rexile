# ReXile Examples

Streamlined examples demonstrating ReXile's features and performance.

## 📚 Examples

### 1. [comprehensive.rs](comprehensive.rs) - **All-in-One Demo** ⭐
Complete showcase of all ReXile features in a single file with menu-driven demos:

**Available demos:**
- `basic` - Basic pattern matching (literals, word chars, digits, anchors)
- `advanced` - Advanced features (captures, lookaround, quantifiers, character classes)
- `performance` - Performance comparison (uncached, pre-compiled, global cache)
- `benchmark` - **36 detailed benchmarks** for common patterns
- `production` - **12 production-ready use cases** (logs, URLs, versions, validation)
- `all` - Run all demos (default)

**Run:**
```bash
cargo run --example comprehensive           # All demos
cargo run --example comprehensive basic     # Basic demo only
cargo run --example comprehensive advanced  # Advanced features
cargo run --example comprehensive benchmark # 36 pattern benchmarks
cargo run --example comprehensive production # 12 real-world use cases
```

**Features covered:**
- ✅ **36 benchmark patterns** - comprehensive performance testing
- ✅ **12 production use cases** - real-world patterns from rust-rule-engine
- ✅ Literals, word chars, digits, anchors, boundaries
- ✅ Capture groups with extraction
- ✅ Lookahead/lookbehind assertions
- ✅ Quantifiers (greedy, lazy, bounded, min/max)
- ✅ Character classes (ranges, negation, Unicode)
- ✅ Alternation (2-4 alternatives)
- ✅ Rule engine patterns (GRL parser, conditions, variables)
- ✅ Network patterns (IP, URLs, protocols)
- ✅ Data extraction (prices, dates, versions)
- ✅ Security validation (usernames, phones, hashes)
- ✅ Configuration parsing (key=value, types)

### 2. [perf_compare.rs](perf_compare.rs) - **Performance Benchmarks**
Detailed performance comparison with regex crate:
- 22 test cases covering all pattern types
- Memory usage comparison
- Compile time benchmarks
- ReXile-only features (lookaround, backreferences)

**Run:**
```bash
cargo run --release --example perf_compare
```

---

## 🚀 Quick Start

```rust
use rexile::{ReXile, Pattern};

// Basic matching
let pattern = ReXile::new(r"\w+@\w+").unwrap();
assert!(pattern.is_match("user@domain"));

// Capture groups
let pattern = Pattern::new(r"(\w+)@(\w+)").unwrap();
if let Some(caps) = pattern.captures("admin@example") {
    println!("User: {:?}", caps.get(1));    // Some("admin")
    println!("Domain: {:?}", caps.get(2));  // Some("example")
}

// Find positions
if let Some((start, end)) = pattern.find("Contact: admin@example") {
    println!("Found at: {}-{}", start, end);
}
```

---

## 📊 Performance Highlights

From `comprehensive.rs` benchmarks (100k iterations):

| Pattern | Time/iter | Notes |
|---------|-----------|-------|
| Literal (`ERROR`) | ~29ns | Fastest with memchr |
| Word run (`\w+`) | ~22ns | Optimized fast path |
| Character class (`[a-z]+`) | ~11ns | Bitmap optimization |
| Email-like (`\w+@\w+`) | ~35ns | Literal prefilter |
| Quantifier (`\d{2,4}`) | ~52ns | Bounded matching |
| Alternation (`ERROR\|WARN`) | ~64ns | Multi-pattern |

**Pre-compiled vs uncached: ~116x faster!**

---

## 💡 Tips

1. **Pre-compile patterns** for best performance:
   ```rust
   let pattern = ReXile::new(r"\w+@\w+").unwrap();  // Compile once
   for line in lines {
       pattern.is_match(line);  // Reuse many times
   }
   ```

2. **Use global cache API** for convenience:
   ```rust
   rexile::is_match(r"\w+@\w+", text).unwrap();  // Auto-cached
   ```

3. **Literal patterns are fastest** - memchr optimization kicks in
4. **Character classes** use bitmap for O(1) lookup
5. **Anchored patterns** (^, $) are optimized

---

## 🎯 Next Steps

- See [FEATURE_STATUS.md](../FEATURE_STATUS.md) for supported features
- Check [PERFORMANCE_RESULTS.md](../PERFORMANCE_RESULTS.md) for detailed benchmarks  
- Read [ROADMAP_FULL_REGEX.md](../ROADMAP_FULL_REGEX.md) for future plans
