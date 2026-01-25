## [0.4.0] - 2025-01-25

### Fixed
- **Critical: Anchors with capturing groups**: Fixed bug where patterns with anchors (`^`, `$`) and capturing groups like `^(\w+)=(\d+)$` would fail to match
  - Root cause: Anchored patterns were incorrectly bypassing capture group parsing
  - Solution: Parse inner pattern with captures, then wrap with anchor constraints
  - All anchored patterns with captures now work correctly

- **Critical: Unicode/emoji panic**: Fixed panic when using `\s+` and other patterns on text containing Unicode multi-byte characters
  - Root cause: Fast path detection incorrectly matched `\s+` as `LiteralPlusWhitespace("")`
  - Solution: Added non-empty literal check in fast path detection
  - Patterns now safely handle emoji and other Unicode characters

### Changed
- `Ast::AnchoredPattern` and `Matcher::AnchoredPattern` added to properly handle anchored patterns with complex inner patterns
- Fast path detection now requires non-empty literals for `LiteralPlusWhitespace`, `LiteralWhitespaceQuoted`, `LiteralWhitespaceDigits`, and `LiteralWhitespaceWord`

### Testing
- All 129 tests pass (84 unit + 13 group integration + 10 captures + 8 lookaround + 8 boundaries + 8 doc-tests)
- Unicode handling verified with emoji-containing strings

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
