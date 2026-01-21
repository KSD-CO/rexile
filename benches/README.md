# ReXile Benchmarks

Comprehensive performance benchmarks for ReXile pattern matching engine.

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench rexile_benchmark
cargo bench --bench comparison_benchmark

# Run specific benchmark
cargo bench literal_search
cargo bench multi_pattern
```

## Benchmark Suites

### 1. `rexile_benchmark.rs`
**Purpose:** Comprehensive ReXile performance analysis

Benchmark groups:
- **literal_search**: Single literal pattern matching performance
- **multi_pattern**: Alternation patterns (`foo|bar|baz`)
- **anchors**: Start/end anchor performance
- **find_operations**: `find()` and `find_all()` performance
- **compilation**: Pattern compilation overhead
- **cached_api**: Global cache performance
- **text_size_scaling**: Performance across different text sizes

```bash
cargo bench --bench rexile_benchmark
```

### 2. `comparison_benchmark.rs`
**Purpose:** Compare ReXile vs regex crate and raw libraries

Benchmark groups:
- **literal_comparison**: ReXile vs regex for literal patterns
- **multi_pattern_comparison**: Alternation performance comparison
- **compilation_comparison**: Compilation speed comparison
- **memchr_raw_comparison**: ReXile vs raw memchr
- **aho_corasick_raw_comparison**: ReXile vs raw aho-corasick

```bash
cargo bench --bench comparison_benchmark
```

## Understanding Results

Criterion outputs results in this format:
```
literal_search/rexile/fox
                        time:   [12.345 ns 12.567 ns 12.789 ns]
```

- **time**: Average execution time with confidence interval
- Lower is better
- ns = nanoseconds, µs = microseconds, ms = milliseconds

## Expected Performance Characteristics

### Literal Search (memchr-based)
- **Small patterns (<10 chars)**: 10-50ns per match
- **Medium patterns (10-100 chars)**: 20-100ns per match
- **SIMD acceleration**: ~4-8x faster than naive search on modern CPUs

### Multi-Pattern (aho-corasick-based)
- **2-4 patterns**: 30-80ns per match
- **5-10 patterns**: 50-150ns per match
- **10+ patterns**: 100-300ns per match
- Scales much better than multiple regex alternations

### Anchors
- **Start anchor (^)**: Similar to literal + position check (~15-60ns)
- **End anchor ($)**: Slightly slower (~20-80ns)
- **Exact match (^$)**: Combined overhead (~25-100ns)

### Compilation
- **Literal**: <100ns
- **Multi-pattern**: 1-10µs (depends on number of patterns)
- **Much faster than regex compilation**: ~10-100x faster

### Cached API
- **First call**: Compilation + hash insert (~1-10µs)
- **Subsequent calls**: Hash lookup + match (~50-200ns)
- **Recommended for repeated patterns**

## Performance Tips

1. **Pre-compile patterns** for best performance:
   ```rust
   let pattern = Pattern::new("foo").unwrap();  // Compile once
   for line in lines {
       pattern.is_match(line);  // Fast repeated use
   }
   ```

2. **Use cached API** for patterns scattered across codebase:
   ```rust
   rexile::is_match("keyword", text).unwrap();  // Auto-cached
   ```

3. **Choose the right pattern type**:
   - 1 literal: Use `Pattern::new("foo")`
   - 2+ literals: Use `Pattern::new("foo|bar|baz")`
   - Position matters: Use anchors `^foo`, `foo$`

## Comparing Against regex Crate

To enable regex comparison benchmarks:

1. Add regex to dev-dependencies in `Cargo.toml`:
   ```toml
   [dev-dependencies]
   criterion = "0.5"
   regex = "1"  # Add this
   ```

2. Uncomment the regex sections in `comparison_benchmark.rs`

3. Run benchmarks:
   ```bash
   cargo bench --bench comparison_benchmark
   ```

Expected results:
- **Literal patterns**: ReXile ~2-5x faster than regex
- **Multi-patterns (2-4)**: ReXile ~1.5-3x faster
- **Multi-patterns (10+)**: ReXile ~5-20x faster
- **Compilation**: ReXile ~10-100x faster

## Optimization Guidelines

### When ReXile Wins
✅ Simple literal searches  
✅ Multi-pattern matching (alternations)  
✅ Fast compilation needed  
✅ Predictable performance required  

### When regex Wins
✅ Complex patterns (character classes, quantifiers)  
✅ Capture groups needed  
✅ Unicode property classes  
✅ Lookahead/lookbehind  

## Profiling

To profile benchmarks with flamegraph:

```bash
cargo install flamegraph
cargo bench --bench rexile_benchmark --profile-time 10
```

## CI Integration

Add to your CI pipeline:

```yaml
- name: Run benchmarks
  run: cargo bench --no-run  # Just compile, don't run
  
- name: Performance regression check
  run: cargo bench -- --save-baseline main
```

## Benchmark Data

Results are saved to `target/criterion/`:
- HTML reports: `target/criterion/report/index.html`
- Raw data: `target/criterion/*/new/estimates.json`

Open the HTML report in a browser for interactive charts:
```bash
open target/criterion/report/index.html
```

## Contributing

When adding new benchmarks:
1. Follow the existing pattern using Criterion
2. Use `black_box()` to prevent compiler optimizations
3. Test with realistic data sizes
4. Document expected performance characteristics
5. Add to the appropriate benchmark group

## Next Steps

- Review [examples](../examples/README.md) for practical usage
- Check [main README](../README.md) for feature documentation
- Read Criterion docs: https://bheisler.github.io/criterion.rs/
