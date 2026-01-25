# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.8] - 2025-01-25

### Added
- **Case-insensitive flag `(?i)` support**: Full implementation of case-insensitive matching
  - Pattern `(?i)hello` now matches "HELLO", "Hello", "hello", etc.
  - Applied `CaseInsensitive` wrapper when `flags.case_insensitive` is set
  - Disabled prefilter optimization when flags are set to ensure correct matching
  - Returns original text positions (not lowercased positions)

### Changed
- Fast path and prefilter optimizations now disabled when regex flags are present
- Ensures correct behavior for case-insensitive, multiline, and DOTALL patterns

### Technical
- `(?s)` DOTALL flag was already working (. matches newlines)
- `(?m)` multiline flag parsing exists but not yet implemented for matching
- Flags framework ready for additional flag implementations

## [0.2.7] - 2025-01-25

### Added
- **Full support for quantified non-capturing groups**: Non-capturing groups can now be quantified
  - Pattern `a(?:b)*c` now works correctly
  - Added quantifier detection after `(?:...)` in `parse_pattern_with_captures_inner()`
  - Wraps quantified non-capturing groups in `Ast::QuantifiedCapture`
  - Supports all quantifiers: `*`, `+`, `?`, `{m,n}`

### Fixed
- **Critical: Zero-width matches with quantifiers**: Fixed quantified patterns to match zero occurrences
  - Pattern `a(b)*c` now correctly matches "ac" (when `(b)*` matches 0 times)
  - Pattern `a(?:b)*c` now correctly matches "ac" (when `(?:b)*` matches 0 times)
  - Pattern `a(?:bc)*d` now correctly matches "ad" (when `(?:bc)*` matches 0 times)
  - Fixed `quantified_find()` to handle empty text when `min=0`
  - Fixed `match_elements_with_backtrack()` to try zero-width matches by changing loop from `(1..=remaining_len)` to `(0..=remaining_len)`
  - Fixed `match_elements_with_backtrack_and_captures()` to try zero-width matches with proper capture handling
  - Special handling for `try_len == 0` case to verify quantifier allows min=0

### Changed
- Quantified patterns now fully support min=0 semantics for `*` and `?` quantifiers
- Zero-width matches are now properly detected and handled in backtracking logic
- All quantified group tests now pass including edge cases with zero occurrences

## [0.2.6] - 2025-01-25

### Fixed
- **Critical: Character classes in parenthesis matching**: Fixed parser to properly skip character classes when matching parentheses
  - Pattern `([^)])` now parses correctly - the `)` inside `[^)]` is no longer counted as closing paren
  - Fixed `find_matching_paren()` to skip over `[...]` character classes including negated classes `[^...]`
  - Fixed `contains_unescaped_paren()` to ignore parentheses inside character classes
  - Fixed next-paren search in `parse_pattern_with_captures_inner()` to skip character classes
  - Prevents infinite recursion when patterns contain character classes with parentheses inside them

- **Critical: Escaped parentheses handling**: Fixed parser to properly handle escaped parentheses `\(` and `\)`
  - Pattern `\(a\)` now parses correctly without infinite recursion
  - Added `contains_unescaped_paren()` helper to distinguish between `(` and `\(`
  - Updated `parse_pattern()` to only call `parse_pattern_with_captures()` for patterns with actual unescaped parens
  - Prevents stack overflow when parsing patterns with escaped parentheses mixed with captures

- **Critical: Alternation in character class context**: Fixed `split_by_alternation()` to skip character classes
  - Pattern `([^)])(x|y)` now parses correctly
  - Depth tracking no longer gets confused by `)` inside `[^)]`
  - Prevents "Unmatched parenthesis" errors for valid patterns with character classes

- **Critical: Alternation with sequence matching**: Fixed alternation matching when used in sequences
  - Pattern `(http|https)://` now correctly matches both "http://" and "https://"
  - Added special handling in `match_elements_with_backtrack()` to try all branches of `AlternationWithCaptures`
  - When first branch doesn't lead to complete match, tries subsequent branches instead of failing
  - Test `test_complex_real_world_patterns` now passing

### Changed
- All GRL regex patterns now compile successfully including complex ones like `([a-zA-Z_]\w*)\s*\(([^)]*)\)\s*(>=|<=|==|!=|>|<|contains|matches)\s*(.+)`
- Character class handling is now consistent across all parser functions
- Escaped character handling is now consistent across all parser functions

## [0.2.5] - 2025-01-25

### Fixed
- **Critical: Exact match requirement in backtracking**: Fixed backtracking logic to require exact match instead of substring match
  - Backtracking now checks `rel_end == substring.len()` to ensure quantified element matches EXACTLY the substring
  - Prevents greedy patterns from over-matching (e.g., `rule\s+` no longer matches entire text)
  - Pattern `rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))` now correctly matches only the rule name, not the entire text

- **Critical: Nested capture extraction**: Implemented proper extraction of captures from nested matchers
  - Added `match_elements_with_backtrack_and_captures()` to extract captures with backtracking support
  - Updated `captures()` method to use backtracking with capture extraction
  - Added `AlternationWithCaptures` case in `extract_nested_captures()` to extract captures from alternation branches
  - Pattern `(?:"([^"]+)"|([a-z]+))` now correctly captures matched branch content

- **Critical: is_match with backtracking**: Updated `is_match` implementation to use backtracking logic
  - `is_match` now calls `match_elements_with_backtrack` for PatternWithCaptures
  - Ensures consistent behavior between `is_match`, `find`, and `captures` methods

- **Optimization: AlternationWithCaptures not treated as quantified**: Fixed `contains_quantified` to not recurse into alternations
  - Alternations are fixed-length choices, not variable-length quantified patterns
  - Prevents unnecessary backtracking for alternation patterns
  - Improves matching performance for patterns with alternation

### Changed
- Complex patterns with alternation and captures now work correctly with full GRL support
- All capture groups properly extracted including nested captures from alternations

### Known Issues Resolved
- ✅ Backtracking with greedy quantifiers - FIXED
- ✅ Alternation with captures extraction - FIXED
- ✅ Complex GRL patterns - NOW WORKING

## [0.2.4] - 2025-01-24

### Fixed
- **Critical: Backtracking with greedy quantifiers**: Implemented proper backtracking for patterns with greedy quantifiers followed by literals
  - Pattern `a(.+)b` now correctly matches `"axxxb"` (backtracks from greedy `.+`)
  - Pattern `\{(.+)\}` now correctly matches `"{ abc }"` (backtracks to leave `}` for literal match)
  - Pattern `([^}]*)\{` now correctly matches `"salience 10 {"` (backtracks character class quantifier)
  - Pattern `start(.+)end` now correctly matches `"start123end"`
  - Added `match_elements_with_backtrack()` with smart strategy: try matching remaining elements first, then verify quantified element can match the substring
  - Added `contains_quantified()` helper to recursively detect patterns needing backtracking
  - All backtracking test cases now passing

### Changed
- Improved matching performance for complex patterns with captures by using intelligent backtracking instead of exhaustive search

## [0.2.3] - 2025-01-24

### Fixed
- **Critical: Alternation with captures**: Fixed patterns with alternation branches containing capture groups
  - Pattern `(?:(a)|(b))` now correctly matches `"a"` and `"b"` with proper capture groups
  - Pattern `(?:"([^"]+)"|([a-zA-Z_]\w*))` now works for matching quoted or unquoted identifiers
  - Pattern `(?:a|b)c` now correctly matches `"ac"` and `"bc"`
  - Added new AST variant `AlternationWithCaptures` to properly handle alternation with captures
  - Implemented `split_by_alternation()` to detect top-level `|` operators
  - All alternation patterns in test suite now passing

### Known Issues
- Backtracking with greedy quantifiers in captures: Patterns like `\{(.+)\}` fail because `.+` greedily consumes everything including the final `}`, and the engine doesn't backtrack. This affects complex patterns with greedy captures followed by literals. Consider using character classes like `[^}]+` instead of `.+` as a workaround.

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
