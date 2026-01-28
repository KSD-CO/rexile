## [0.4.10] - 2026-01-27

### Added
- **Full lookaround support**: Complete implementation of lookahead and lookbehind with combined patterns
  - `foo(?=bar)` - Match 'foo' only if followed by 'bar' (positive lookahead with prefix)
  - `foo(?!bar)` - Match 'foo' only if NOT followed by 'bar' (negative lookahead with prefix)
  - `(?<=foo)bar` - Match 'bar' only if preceded by 'foo' (positive lookbehind with suffix)
  - `(?<!foo)bar` - Match 'bar' only if NOT preceded by 'foo' (negative lookbehind with suffix)
  - All combinations now work correctly in `is_match()`, `find()`, and `find_all()`
  - 10 comprehensive integration tests added for combined lookaround patterns

### Changed
- Enhanced AST structure with `LookbehindWithSuffix` variant for proper lookbehind+suffix handling
- Improved pattern parser to correctly route combined lookaround patterns
- All 138 tests passing (84 unit + 13 group + 9 capture + 10 combined lookaround + 8 lookaround + 6 boundary + 8 doc tests)

## [0.4.7] - 2025-01-26

### Fixed
- **Critical: Case-insensitive with uppercase patterns**: Fixed bug where `(?i)GET` failed to match "GET" (only worked with lowercase patterns)
  - Root cause: CaseInsensitive wrapper only lowercased text, not the pattern itself
  - Solution: Created `lowercase_ast()` function to recursively lowercase all literals in the AST before compilation
  - Patterns like `(?i)(GET|POST)` now correctly match "GET", "get", "Post", etc.

- **Critical: Range quantifiers in sequences**: Fixed bug where range quantifiers `{n}`, `{n,}`, `{n,m}` were parsed as literal characters in sequences
  - Root cause: `parse_quantifier_with_lazy()` only handled `*`, `+`, `?` quantifiers
  - Solution: Extended parser to recognize and parse range quantifiers
  - Patterns like `\d{1,3}\.` and `\b\d{4}\b` now work correctly

- **Critical: Position calculation bug**: Fixed incorrect end position returned by `find()` when using word boundaries with quantifiers
  - Root cause: `find()` incorrectly treated absolute final position as consumed bytes
  - Solution: Changed position calculation to use final_pos directly instead of adding to start_pos
  - Patterns like `\b\d{4}\b` now return correct match positions

### Added
- **Range quantifiers**: Full support for `{n}`, `{n,}`, and `{n,m}` patterns
  - `\d{4}` matches exactly 4 digits
  - `\d{1,3}` matches 1 to 3 digits
  - `\w{2,}` matches 2 or more word characters
  - Works correctly in sequences with other elements

- **Case-insensitive flag**: Full support for `(?i)` flag
  - `(?i)test` matches "test", "TEST", "Test", etc.
  - `(?i)(GET|POST)` matches any case variation
  - Works with capturing groups and complex patterns

### Testing
- All 52/52 production-ready tests pass (100% success rate)
- All 137 unit tests pass (84 unit + 13 group + 10 captures + 8 lookaround + 8 boundaries + 8 doc-tests + 6 word-boundaries)
- Known limitations updated: range quantifiers bug removed

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
