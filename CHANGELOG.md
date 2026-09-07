## [Unreleased]

## [0.7.2] - 2026-09-07

### Fixed
- **Invalid pattern `(?` was accepted** - `check_balanced_parens` was bypassed
  whenever `has_captures` routed a pattern directly to
  `parse_pattern_with_captures_with_flags`, so `(?` silently dropped the `(`
  and matched `?` as a literal. Unbalanced parentheses are now rejected upfront
  in `Pattern::parse`, and a new `validate_group_syntax` check whitelists
  supported group extensions (`(?:`, `(?=`, `(?!`, `(?<=`, `(?<!`) while
  explicitly rejecting unsupported ones (named captures, comments, atomic
  groups, branch resets, conditionals) and malformed ones (`(?)`, `(?a`, `(??`,
  etc.) with a proper parse error. ([#13](https://github.com/KSD-CO/rexile/issues/13))
- **Valid pattern `(é)` panicked on multi-byte UTF-8** - byte-by-byte position
  advancement in `parse_pattern_with_captures_inner` and a char-count/byte-slice
  mismatch in `extract_alternation_prefix` could slice a string in the middle of
  a multi-byte UTF-8 character. Positions now advance by `char::len_utf8()`, and
  prefix lengths are mapped to byte offsets via `char_indices()`, fixing panics
  on 2-byte, 3-byte (CJK), and 4-byte (emoji) characters inside groups.
  ([#14](https://github.com/KSD-CO/rexile/issues/14))

### Testing
- Added `tests/test_issue_13_14_regression.rs` covering rejection of unclosed/
  malformed/unsupported group syntax and acceptance of multi-byte UTF-8
  captures, alternations, quantifiers, and classes.

### Credits
- Thanks to [@Viorel](https://github.com/Viorel) for reporting both issues and
  [@nghiaphamln](https://github.com/nghiaphamln) for the fix.
  ([#15](https://github.com/KSD-CO/rexile/pull/15))

### Added
- PatternSet with stable rule IDs, first/all match locations, numbered captures,
  independent overlaps, reusable caches, and cancellable capture visitors.
- Shared literal candidate indexing and deterministic capture programs, with
  general rexile matching retained where those optimizations are not applicable.
- A fixed differential corpus, allocation checks, examples, and a reproducible
  Criterion/heap acceptance runner against regex 1.13.1. Performance targets
  require passing the measured gates; no universal speedup is claimed.

### Fixed
- Empty-match iterators now progress at UTF-8 boundaries, include the end of
  the input, and suppress empty matches adjacent to the preceding match.
- Capture parsing no longer slices UTF-8 while scanning for backreferences.
- Unicode case-insensitive find results retain original source offsets.
- Lazy quantifiers preserve their minimum match length on non-ASCII input.
- Internal visibility no longer exposes private matcher types to Rust 1.70.

## [0.7.0] - 2026-09-04

### Fixed
- **Global flags were not semantically correct** - `(?i)`, `(?m)`, and `(?s)` were parsed but not consistently honored deeper in the AST (groups, alternations, captures, quantified non-capturing groups). `^`/`$` are now represented as true zero-width `Anchor` assertions evaluated against the full haystack instead of a fragile prefix/suffix string strip, fixing multiline anchors inside groups, alternations, captures, and lookarounds. DOTALL now applies consistently inside captures and quantified non-capturing groups.
- **Unsupported inline flag syntax was silently ignored** - `x`, `u`, `U`, `R`, flag-disabling (`(?-i)`), scoped flags (`(?i:...)`), and mid-pattern flag changes now return `PatternError::UnsupportedFeature` instead of being accepted and producing wrong results. Supported flags remain global and must appear at the beginning of a pattern; consecutive leading flag groups (e.g. `(?i)(?m)`) are merged.
- Added a safe line-start/literal prefilter so multiline anchors don't regress performance on long inputs.

### Testing
- Added `tests/test_flags.rs`, a differential suite covering multiline anchors in alternations/groups/captures/replacement/split, DOTALL in captures, consecutive flag groups, literal anchors, and unsupported flag syntax, all checked against the `regex` crate.

### Credits
- Thanks to [@nghiaphamln](https://github.com/nghiaphamln) for the fix. ([#11](https://github.com/KSD-CO/rexile/pull/11))

## [0.6.3] - 2026-09-04

### Fixed
- **Security: DoS via stack overflow** - `Pattern::new(")")` (and other patterns with an unmatched parenthesis) crashed the process with a stack overflow instead of returning a parse error. The unmatched `)` fell through to the capture-group segmentation logic, which re-parsed the identical segment forever without ever advancing the recursion-depth guard. Added `check_balanced_parens()` to reject unbalanced parentheses up front with a normal `PatternError`. ([#9](https://github.com/KSD-CO/rexile/issues/9))
- **`\|` matched any text** - The top-level alternation check used a naive `pattern.split('|')`, which also split on an escaped `\|`, producing a bogus empty alternative that matched anything. Now reuses the existing escape-aware `split_by_alternation()` helper so `\|` only matches a literal `|`. ([#7](https://github.com/KSD-CO/rexile/issues/7))

### Credits
- Thanks to [@Viorel](https://github.com/Viorel) for reporting both issues with clear repro steps.

## [0.5.3] - 2026-02-02

### Fixed
- **CRITICAL: Complete Unicode safety** - Fixed additional Unicode boundary issues in DOTALL mode and capture iteration
  - **Issue 1**: `match_elements_backtracking_dotall()` could use non-char-boundary positions with Unicode text
  - **Issue 2**: Capture group iterator (`find_iter`) iterated byte-by-byte instead of char-by-char
  - **Root cause**: DOTALL mode (`.` matches newlines) and capture matching didn't check char boundaries
  - **Solution**: 
    - Added char boundary check in `match_elements_backtracking_dotall()` 
    - Changed capture iteration to use `char_indices()` instead of byte positions
  - **Real-world impact**: Fixed panic in rust-rule-engine with GRL files containing Unicode arrows (→) in comments

### Testing
- ✅ All rust-rule-engine examples now work with Unicode
- ✅ `ecommerce_approval_demo` with Unicode → arrows and ✅ emoji - PASSES
- ✅ 11+ examples tested successfully
- ✅ 152 rust-rule-engine tests + 100 rexile tests all passing

### Impact
- **Backward chaining examples**: Now work correctly with Unicode in GRL comments
- **Production ready**: Complete Unicode safety for all regex operations
- **No more panic**: All char boundary violations fixed

## [0.5.2] - 2026-02-02

### Fixed
- **CRITICAL: Unicode safety** - Fixed false matches and potential panics with non-ASCII text
  - Root cause: Byte-based optimizations in v0.5.1 incorrectly handled multi-byte UTF-8 characters
  - Solution: Restrict byte-based scanning optimizations to ASCII-only text (`text.is_ascii()`)
  - For Unicode text, fallback to correct char-by-char matching
  - Added comprehensive Unicode test suite (8 tests covering emoji, CJK, math symbols, Vietnamese, etc.)
  
### Impact
- ✅ **ASCII text**: Full performance benefits of v0.5.1 optimizations (50%+ faster)
- ✅ **Unicode text**: Correct matching with acceptable performance (uses fallback path)
- ✅ **No panics**: All char boundary checks in place
- ✅ **GRL files with Unicode comments**: Now parse correctly

### Testing
- All 100 tests pass (84 unit + 8 Unicode + 8 other integration tests)
- Unicode test coverage: arrows (→), emoji (🚀), CJK (规则), math symbols (∑∫∂), Vietnamese (Tiếng Việt)

## [0.5.1] - 2026-02-02

### Performance Improvements
- **50%+ faster pattern matching** for complex patterns:
  - `[a-z]+.+[0-9]+` (overlap patterns): 5.91x → 2.80x slower (52% improvement)
  - `(\w+)@(\w+)` (capture patterns): 4.98x → 2.46x slower (51% improvement)
  - `\w+\s+\d+` (adjacent charclass patterns): 6.76x → 3.10x slower (54% improvement)

### Optimizations
- **Reverse prefilter for 3-element patterns**: Find most selective element (e.g., digits) first, then scan backward
- **Early termination checks**: Skip positions that can't possibly match before expensive backtracking
- **Direct match calculation**: For non-overlapping charclass patterns, compute match bounds directly without `match_at_pos()`
- **`find_no_captures()` method**: Skip capture tracking when captures aren't needed
- **Improved dot class detection**: `is_dot_class()` method for better wildcard pattern handling

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
