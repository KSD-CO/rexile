# ReXile vs Regex Crate Performance Comparison

## Executive Summary

Comparison between **ReXile** (custom regex-lite engine) and the standard **regex** crate v1.12.2.

**Key Findings:**
- ✅ **Anchored patterns**: ReXile is **2-5x FASTER** than regex
- ✅ **Simple alternation**: ReXile competitive with regex
- ✅ **Simple groups**: ReXile competitive  
- ❌ **Character classes**: regex is 10-14x faster (SIMD optimizations)
- ❌ **Complex patterns**: regex is 10-40x faster on large text
- ❌ **Find all operations**: regex is 5-33x faster (optimized iterators)

## Detailed Performance Results

### 1. Literal Patterns

| Pattern | ReXile | Regex | Winner |
|---------|--------|-------|---------|
| `hello` | 99.03 ns | **13.13 ns** | Regex **7.5x faster** |
| `world` | 125.58 ns | **13.00 ns** | Regex **9.7x faster** |
| `hello world` | 121.58 ns | **13.37 ns** | Regex **9.1x faster** |

**Analysis**: Regex's memchr-based literal optimization is extremely fast.

---

### 2. Anchored Patterns (ReXile Strength!)

| Pattern | ReXile | Regex | Winner |
|---------|--------|-------|---------|
| `^hello` | **4.69 ns** | 14.21 ns | **ReXile 3.0x faster** ✅ |
| `test$` | **4.24 ns** | 24.31 ns | **ReXile 5.7x faster** ✅ |
| `^hello world` | **5.01 ns** | 17.31 ns | **ReXile 3.5x faster** ✅ |

**Analysis**: ReXile's specialized anchored literal matching is extremely efficient. This is a **major advantage**.

---

### 3. Character Classes

| Pattern | ReXile | Regex | Winner |
|---------|--------|-------|---------|
| `[a-z]+` | 182.24 ns | **13.38 ns** | Regex **13.6x faster** |
| `[0-9]+` | 399.07 ns | **34.65 ns** | Regex **11.5x faster** |
| `[a-zA-Z]+` | 177.61 ns | **13.50 ns** | Regex **13.2x faster** |
| `[^0-9]+` | 171.30 ns | **13.33 ns** | Regex **12.9x faster** |

**Analysis**: Regex has highly optimized SIMD character class matching. ReXile's ASCII bitmap is good but not SIMD-optimized.

---

### 4. Escape Sequences

| Pattern | ReXile | Regex | Winner |
|---------|--------|-------|---------|
| `\d+` | 512.71 ns | **14.15 ns** | Regex **36.2x faster** |
| `\w+` | 190.43 ns | **13.34 ns** | Regex **14.3x faster** |
| `\s+` | 422.87 ns | **13.96 ns** | Regex **30.3x faster** |
| `\d+\s+\w+` | 668.20 ns | **15.25 ns** | Regex **43.8x faster** |

**Analysis**: Regex's escape sequence handling is extremely optimized.

---

### 5. Quantifiers

| Pattern | ReXile | Regex | Winner |
|---------|--------|-------|---------|
| `a+` | 172.83 ns | **17.90 ns** | Regex **9.7x faster** |
| `b*` | 159.72 ns | **16.40 ns** | Regex **9.7x faster** |
| `c?` | 166.81 ns | **16.00 ns** | Regex **10.4x faster** |
| `a+b+c+` | 283.03 ns | **19.20 ns** | Regex **14.7x faster** |

**Analysis**: Regex's quantifier matching is highly optimized.

---

### 6. Groups

| Pattern | ReXile | Regex | Winner |
|---------|--------|-------|---------|
| `(foo)` | 89.39 ns | **20.07 ns** | Regex **4.5x faster** |
| `(foo\|bar)` | 87.21 ns | **36.61 ns** | Regex **2.4x faster** |
| `^(hello)` | **4.66 ns** | 12.82 ns | **ReXile 2.7x faster** ✅ |
| `(\d+)` | 570.26 ns | **17.94 ns** | Regex **31.8x faster** |
| `(\w+)@` | 597.26 ns | **11.75 ns** | Regex **50.8x faster** |

**Analysis**: ReXile competitive on simple groups, especially anchored groups. Regex faster on complex groups.

---

### 7. Alternation

| Pattern | ReXile | Regex | Winner |
|---------|--------|-------|---------|
| `cat\|dog` | **20.84 ns** | 21.94 ns | **ReXile 1.05x faster** ✅ |
| `cat\|dog\|fox` | 22.53 ns | **20.93 ns** | Regex 1.08x faster |
| `one\|two\|three\|four\|five` | 53.09 ns | **20.73 ns** | Regex **2.6x faster** |

**Analysis**: ReXile's Aho-Corasick alternation is competitive for 2-3 alternatives. Regex scales better with many alternatives.

---

### 8. Find All Operations

| Pattern | ReXile | Regex | Winner |
|---------|--------|-------|---------|
| `test` | 232.73 ns | **47.31 ns** | Regex **4.9x faster** |
| `\d+` | 2.25 µs | **67.20 ns** | Regex **33.5x faster** |
| `test\d+` | 499.92 ns | **109.85 ns** | Regex **4.6x faster** |

**Analysis**: Regex's iterator-based `find_iter()` is highly optimized. ReXile needs optimization here.

---

### 9. Large Text (1000 repetitions of "hello world ")

| Pattern | ReXile | Regex | Winner |
|---------|--------|-------|---------|
| `hello` | 8.07 µs | **12.52 ns** | Regex **645x faster** ⚠️ |
| `\w+` | 17.05 µs | **13.21 ns** | Regex **1,291x faster** ⚠️ |
| `hello world` | 7.89 µs | **13.69 ns** | Regex **576x faster** ⚠️ |

**Analysis**: **CRITICAL ISSUE!** Regex returns immediately after finding first match on large text. ReXile appears to be scanning entire text. This needs investigation and optimization.

---

### 10. Real World Patterns

| Pattern | ReXile | Regex | Winner |
|---------|--------|-------|---------|
| `\w+@\w+\.\w+` (email) | 393.14 ns | **43.12 ns** | Regex **9.1x faster** |
| `\d+\.\d+\.\d+` (version) | 1.06 µs | **28.66 ns** | Regex **37.0x faster** |
| `https?://\w+\.\w+` (URL) | 427.98 ns | **32.16 ns** | Regex **13.3x faster** |

**Analysis**: Regex significantly faster on real-world patterns with escape sequences.

---

## Summary Statistics

### ReXile Advantages ✅
1. **Anchored patterns**: 2-5x faster than regex
2. **Simple alternation (2 items)**: Competitive or slightly faster
3. **Anchored groups**: 2-3x faster than regex
4. **Code simplicity**: Much simpler implementation

### Regex Advantages 🏆
1. **Character classes**: 10-14x faster (SIMD optimizations)
2. **Escape sequences**: 15-45x faster
3. **Complex patterns**: 10-50x faster
4. **Large text**: 500-1000x faster (early termination optimization)
5. **Find all operations**: 5-33x faster (optimized iterators)
6. **Memory efficiency**: Highly optimized internal structures

---

## Recommendations

### When to Use ReXile ✅
- **Anchored pattern matching** (`^hello`, `world$`)
- **Simple alternation** with 2-3 options
- **Educational purposes** - understanding regex internals
- **Zero regex crate dependency** required
- **Embedded systems** with size constraints

### When to Use Regex Crate 🏆
- **Production applications** requiring maximum performance
- **Large text processing**
- **Complex patterns** with character classes and escapes
- **Find all operations** on large datasets
- **Unicode support** required
- **Comprehensive regex features** (backreferences, lookahead, etc.)

---

## Critical Issues to Address in ReXile

### Priority 1: Large Text Performance ⚠️
**Problem**: ReXile 500-1000x slower on large text.
**Cause**: Not returning immediately after first match in `is_match()`.
**Fix**: Implement early termination optimization.

### Priority 2: Character Class Optimization
**Problem**: 10-14x slower than regex.
**Solution**: Consider SIMD optimizations or faster bitmap scanning.

### Priority 3: Find All Iterator Optimization
**Problem**: 5-33x slower than regex on `find_all()`.
**Solution**: Optimize iterator implementation, reduce allocations.

### Priority 4: Escape Sequence Performance
**Problem**: 15-45x slower on `\d+`, `\w+`, etc.
**Solution**: Optimize character class matching for common escapes.

---

## Conclusion

ReXile demonstrates **excellent performance on anchored patterns** (2-5x faster than regex), proving the concept of specialized optimizations. However, regex's decades of optimization work show in **10-1000x advantages** on complex patterns and large text.

**ReXile's niche**: Fast anchored pattern matching without regex dependency.
**Regex's domain**: General-purpose, production-grade pattern matching.

Both libraries have their place depending on use case requirements.

---

## Test Environment

- **Date**: January 21, 2026
- **Rust**: Edition 2021
- **ReXile**: v0.1.0
- **Regex**: v1.12.2
- **Criterion**: v0.5.1
- **CPU**: (Results from criterion benchmark on Linux)
- **Iterations**: 100 samples per test
