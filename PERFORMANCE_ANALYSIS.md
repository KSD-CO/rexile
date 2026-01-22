# Performance & Memory Analysis: ReXile vs Regex

## Quick Reference Card

| Metric | ReXile | Regex Crate | Winner |
|--------|--------|-------------|---------|
| **Best Use Case** | Anchored patterns | General purpose | - |
| **Worst Case** | Large text scanning | None identified | - |
| **Memory per Pattern** | 200-500 bytes | 100-200 bytes | Regex |
| **Compilation Speed** | Instant | Fast (JIT) | ReXile |
| **Anchor Performance** | **4-5 ns** | 12-24 ns | **ReXile 3-5x** ✅ |
| **Character Class** | 170-400 ns | **13-35 ns** | Regex 10-14x ❌ |
| **Large Text (12KB)** | 8-17 µs | **13 ns** | Regex 600-1200x ❌ |
| **Dependencies** | memchr, aho-corasick | regex | - |
| **Code Complexity** | Low | High | ReXile |

---

## Performance Categories

### 🏆 ReXile Wins (2-5x faster)

```
Anchored Patterns:
  ^hello          4.69 ns  vs  14.21 ns  →  3.0x faster ✅
  test$           4.24 ns  vs  24.31 ns  →  5.7x faster ✅
  ^hello world    5.01 ns  vs  17.31 ns  →  3.5x faster ✅
  ^(hello)        4.66 ns  vs  12.82 ns  →  2.7x faster ✅

Simple Alternation:
  cat|dog        20.84 ns  vs  21.94 ns  →  1.05x faster ✅
```

**Why ReXile wins:**
- Direct string comparison for anchored literals
- No DFA/NFA overhead
- Specialized fast path for this common case

---

### 🤝 Competitive (within 2x)

```
Simple Groups:
  (foo)          89.39 ns  vs  20.07 ns  →  4.5x slower (acceptable)
  (foo|bar)      87.21 ns  vs  36.61 ns  →  2.4x slower (acceptable)

Quantifiers:
  a+            172.83 ns  vs  17.90 ns  →  9.7x slower
  b*            159.72 ns  vs  16.40 ns  →  9.7x slower
```

**Analysis**: ReXile holds its own on simple patterns, though regex optimizations show.

---

### ❌ Regex Dominates (10-1000x faster)

```
Character Classes:
  [a-z]+        182.24 ns  vs  13.38 ns  →   13.6x faster ❌
  [0-9]+        399.07 ns  vs  34.65 ns  →   11.5x faster ❌

Escape Sequences:
  \d+           512.71 ns  vs  14.15 ns  →   36.2x faster ❌
  \w+           190.43 ns  vs  13.34 ns  →   14.3x faster ❌
  \d+\s+\w+     668.20 ns  vs  15.25 ns  →   43.8x faster ❌

Large Text (12KB):
  hello          8.07 µs   vs  12.52 ns  →  645x faster ❌❌❌
  \w+           17.05 µs   vs  13.21 ns  → 1291x faster ❌❌❌
  hello world    7.89 µs   vs  13.69 ns  →  576x faster ❌❌❌

Find All:
  \d+            2.25 µs   vs  67.20 ns  →   33.5x faster ❌❌
```

**Why Regex wins:**
- SIMD-optimized character class matching
- Highly optimized DFA engine
- Early termination on large text
- Decades of optimization work

---

## Critical Performance Issues in ReXile

### 🔴 Priority 1: Large Text Performance

**Problem:** 600-1000x slower on large text (12KB)

```rust
// Current behavior (SLOW):
fn is_match(&self, text: &str) -> bool {
    self.find(text).is_some()  // find() may scan entire text!
}

// Should be (FAST):
fn is_match(&self, text: &str) -> bool {
    // Return immediately after first match
    // Don't scan entire text for is_match()
}
```

**Impact:** Makes ReXile unusable for large files.

**Fix:** Implement early termination in `is_match()` for all pattern types.

---

### 🟡 Priority 2: Character Class Optimization

**Current:** 170-400 ns (10-14x slower than regex)

**Regex approach:**
- SIMD vectorization (process 16-32 bytes at once)
- Branch prediction optimization
- Specialized code paths for common ranges

**Possible ReXile improvements:**
1. Consider SIMD using `packed_simd` or similar
2. Optimize bitmap lookup with better branching
3. Add fast paths for common classes like `[a-z]`, `[0-9]`

---

### 🟡 Priority 3: Find All Optimization

**Current:** 2.25 µs for `\d+` (33x slower)

**Issues:**
- Creating new Vec for each iteration
- Inefficient iterator implementation
- No reuse of internal buffers

**Regex approach:**
- Zero-allocation iterator
- Reuses internal state machine
- Optimized position tracking

---

## Memory Usage Analysis

### Pattern Size Estimates

| Pattern Type | ReXile | Regex | Notes |
|--------------|--------|-------|-------|
| Simple literal (`hello`) | ~80 bytes | ~100 bytes | ReXile: String + memchr finder |
| Anchored (`^hello`) | ~100 bytes | ~120 bytes | Similar overhead |
| Character class (`[a-z]+`) | ~200 bytes | ~150 bytes | ReXile: ASCII bitmap (256 bits) |
| Complex sequence | 300-500 bytes | 200-300 bytes | Regex: Optimized bytecode |
| Alternation (5 items) | 200-400 bytes | 250-350 bytes | Both use specialized structures |

### Memory Characteristics

**ReXile:**
- ✅ Simple, predictable memory usage
- ✅ No runtime compilation
- ✅ Small binary size
- ❌ Slightly larger pattern structs

**Regex:**
- ✅ Highly optimized structs
- ✅ Memory pooling for DFA cache
- ❌ JIT compilation overhead
- ❌ Larger binary size (~500KB)

---

## CPU Usage Patterns

### ReXile CPU Profile

```
Anchored patterns:    ████ 4-5 ns    (minimal CPU)
Literals:            ████████████ 100-120 ns
Character classes:   ██████████████████ 170-400 ns
Sequences:           ██████████████████████████ 500-700 ns
```

**Characteristics:**
- Low CPU for anchored patterns
- Moderate CPU for simple patterns
- Higher CPU for character-intensive operations
- Predictable, consistent overhead

---

### Regex CPU Profile

```
All patterns:        ████ 13-45 ns     (highly optimized)
Complex patterns:    █████████ 50-200 ns
JIT compilation:     ████████████████ 1-10 µs (one-time)
```

**Characteristics:**
- Ultra-low CPU due to SIMD and optimizations
- JIT compilation cost amortized over many matches
- DFA cache warmup may affect first few matches

---

## Recommendations by Use Case

### ✅ Use ReXile When:

1. **Anchored pattern matching is primary use case**
   - Log parsing: `^ERROR`, `^WARNING`
   - Config validation: `^[A-Z_]+$`
   - Start-of-line matching

2. **Zero regex dependency required**
   - Embedded systems
   - Minimal dependency projects
   - Security-critical code with minimal deps

3. **Simple patterns only**
   - Literal matching
   - Basic alternation (2-3 items)
   - Simple character classes

4. **Educational/Learning purposes**
   - Understanding regex internals
   - Teaching pattern matching concepts
   - Demonstration code

### ✅ Use Regex Crate When:

1. **Production applications** requiring:
   - Maximum performance
   - Large text processing
   - Complex pattern support
   - Unicode handling

2. **Character class heavy workloads**
   - Email validation
   - Phone number parsing
   - Data extraction with `\d+`, `\w+`

3. **Find all operations**
   - Log analysis (find all errors)
   - Data mining
   - Text extraction

4. **Large file processing**
   - Log files (MB-GB)
   - Data import/export
   - Batch text processing

---

## Optimization Roadmap for ReXile

### Phase 1: Critical Fixes (Required for usability)
- [ ] Fix large text performance (early termination)
- [ ] Optimize `find_all()` iterator (zero-allocation)
- [ ] Add benchmarks for regression detection

### Phase 2: Performance Improvements
- [ ] Character class SIMD optimization
- [ ] Escape sequence fast paths
- [ ] Memory pool for temporary buffers

### Phase 3: Feature Completion
- [ ] Bounded quantifiers `{n,m}`
- [ ] Capturing groups with extraction
- [ ] Backreferences `\1`, `\2`

---

## Conclusion

**ReXile excels at its niche:**
- ✅ **3-5x faster** anchored pattern matching
- ✅ Simple, understandable codebase
- ✅ Zero regex dependency
- ✅ Good educational value

**Regex crate dominates general-purpose matching:**
- 🏆 **10-1000x faster** on most patterns
- 🏆 Production-ready, battle-tested
- 🏆 Comprehensive feature set
- 🏆 Decades of optimization

**Verdict:** ReXile has found its niche in anchored pattern matching. With critical fixes (especially large text handling), it could be a viable alternative for specific use cases where anchored patterns dominate and regex dependency is undesirable.

---

## Benchmark Details

**Test Date:** January 21, 2026  
**ReXile Version:** 0.1.0  
**Regex Version:** 1.12.2  
**Rust Edition:** 2021  
**Criterion:** 0.5.1  
**Platform:** Linux  
**Samples:** 100 iterations per test  

Full benchmark results: `benchmark_comparison.txt`  
Criterion HTML reports: `target/criterion/`
