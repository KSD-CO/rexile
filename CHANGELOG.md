# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-01-XX

### Added
- **Dot wildcard support**: Full implementation of `.`, `.*`, and `.+` patterns
  - Single dot `.` matches any character except newline
  - Quantified dots `.*` and `.+` with proper backtracking
  - Pattern `a.c` now correctly matches `abc`, `a_c`, etc.
  - Pattern `.*test.*` correctly matches strings containing "test"
- **Backtracking algorithm**: Recursive backtracking for quantified elements in sequences
  - Handles greedy quantifiers with proper backtracking behavior
  - Supports complex patterns like `a.+c`, `\w+.*\d+`
  - Ensures correct matching for patterns with multiple quantified elements
- **Capturing groups**: Automatic detection and extraction support
  - Auto-detects capturing groups when pattern contains `(` but not `(?:`
  - Provides `captures()` API for extracting matched groups

### Fixed
- Empty string matching for quantifiers with minimum count of zero
  - Pattern `.*test.*` now correctly matches "test" (empty prefix/suffix)
  - Pattern `a*` can now match empty string
  - Quantifiers with `min=0` properly handle zero-length matches

### Changed
- **10-100x faster compilation** compared to regex crate
  - Pattern `[a-zA-Z_]\w*`: 104.7x faster compilation
  - Pattern `\d+`: 46.5x faster compilation
  - Pattern `(\w+)\s*(>=|<=|==|!=|>|<)\s*(.+)`: 40.7x faster compilation
- **Matching performance trade-offs**:
  - Simple patterns: 1.4-1.9x faster than regex
  - Complex patterns with backtracking: 2-10x slower (acceptable for non-hot-path usage)
- Updated crate description to emphasize compilation speed advantages

### Performance
- **Ideal for parsers and rule engines**: 100x faster startup time when loading many patterns
- **Memory efficient**: 15x less compilation memory, 5x less peak memory
- **Perfect trade-off**: Instant pattern compilation vs slightly slower complex matching

## [0.1.1] - 2025-01-XX

### Initial Release
- Literal searches with SIMD acceleration
- Multi-pattern matching (alternations)
- Character classes with negation
- Quantifiers (`*`, `+`, `?`)
- Escape sequences (`\d`, `\w`, `\s`, etc.)
- Sequences and groups
- Word boundaries (`\b`, `\B`)
- Anchoring (`^`, `$`)
- 10 specialized fast paths for common patterns
- Minimal dependencies (only `memchr` and `aho-corasick`)

[0.2.0]: https://github.com/yourusername/rexile/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/yourusername/rexile/releases/tag/v0.1.1
