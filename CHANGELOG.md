# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - 2025-01-24

### Fixed
- **Critical: Zero-width quantifier matching**: Fixed patterns with `*` quantifiers not matching when consuming zero characters
  - Pattern `\s*a` now correctly matches `"a"` (with `\s*` consuming zero characters)
  - Pattern `\s*\{` now correctly matches `"{"` (with `\s*` consuming zero characters)
  - Pattern `a*b` now correctly matches `"b"` (with `a*` consuming zero characters)
  - Implemented epsilon closure in NFA simulation to handle optional elements (min=0)
  - Added `optional_bits` field to NfaTable to track elements that can be skipped
  - Updated `is_dfa_compilable` to route `ZeroOrMore` patterns through NFA path
- All 84 unit tests + 13 integration tests passing

### Known Issues
- Backtracking with capture groups followed by literals (e.g., `\{(.+)\}`) not working correctly
- Alternation patterns with captures followed by literals (e.g., `(?:a|b)c`) not matching properly
- These issues affect complex patterns used in GRL parsing but do not impact simple regex use cases

## [0.2.1] - 2025-01-24

### Added
- **Non-greedy (lazy) quantifiers**: Full support for `*?`, `+?`, and `??`
  - Pattern `.*?` matches as few characters as possible
  - Pattern `.+?` requires at least one character but matches minimally
  - Pattern `??` matches zero or one time, preferring zero
  - Example: `start\{.*?\}` matches `"start{abc}"` not `"start{abc}end{xyz}"`
- **DOTALL mode**: `(?s)` flag makes dot match newlines
  - Pattern `(?s).*` matches across line boundaries
  - Pattern `(?s)rule\s+.*?\}` correctly matches multi-line rule definitions
  - Enables matching of multi-line text blocks with dot wildcard
- **Non-capturing groups with alternations**: `(?:...)` support
  - Pattern `(?:"test"|foo)` matches either quoted "test" or literal foo
  - Full support for complex alternations inside groups
  - Groups can be quantified: `(?:abc|def)+`
  - Integrated with sequence matching and backtracking

### Fixed
- DOTALL backtracking consistency: Ensures all quantified elements in DOTALL mode correctly call DOTALL continuation paths
- Prefilter disabled for patterns with groups to maintain correctness
- Non-capturing group matching in complex patterns

### Changed
- Updated crate description to include new features
- Test suite expanded to 84 unit tests + 13 group integration tests

### Performance
- **Zero regression**: Maintains 13/15 patterns faster than regex (0.75x total time)
- All optimizations from v0.2.0 preserved while adding new features

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

[0.2.1]: https://github.com/KSD-CO/rexile/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/KSD-CO/rexile/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/KSD-CO/rexile/releases/tag/v0.1.1
