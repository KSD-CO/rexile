# Rexile Feature Status

## PatternSet

- Multiple patterns with stable IDs, including duplicate patterns.
- First and all match locations and numbered captures.
- Independent matching with overlaps between rules.
- Shared immutable compilation, reusable caches, and cancellable visitors.
- UTF-8 offsets and empty-match iteration.

See [PatternSet documentation](docs/PATTERN_SET.md) for compatibility boundaries
and performance acceptance checks. Performance targets require measured
validation; feature availability does not imply every workload beats regex.
The [measured results](docs/PATTERN_SET_RESULTS.md) currently fail the per-case
speed gate, despite passing the aggregate speed, heap, and regression targets.

## Production-Ready Features (✓)

### Literals & Character Classes
- ✓ Literal strings
- ✓ Character ranges: `[a-z]`, `[A-Z]`, `[0-9]`
- ✓ Multiple ranges: `[a-zA-Z0-9]`
- ✓ Negated classes: `[^a-z]`, `[^\s]`
- ✓ Single characters: `[abc]`

### Quantifiers
- ✓ One or more: `+`
- ✓ Zero or more: `*`
- ✓ Zero or one: `?`
- ✓ At least N: `{n,}` (e.g., `a{2,}`)
- ✓ Lazy versions: `*?`, `+?`, `??`

### Escape Sequences
- ✓ Digits: `\d`
- ✓ Word characters: `\w`
- ✓ Whitespace: `\s`
- ✓ Escaped special chars: `\.`, `\+`, `\*`, etc.

### Boundaries & Anchors
- ✓ Word boundary: `\b`
- ✓ Non-word boundary: `\B`
- ✓ Start anchor: `^`
- ✓ End anchor: `$`

### Alternation
- ✓ Simple: `cat|dog`
- ✓ Complex: `(http|https)://`
- ✓ Multiple: `a|b|c`

### Groups
- ✓ Capturing: `(pattern)`
- ✓ Non-capturing: `(?:pattern)`
- ✓ Capture extraction via `captures()`

### Lookaround
- ✓ Positive lookahead: `(?=pattern)`
- ✓ Negative lookahead: `(?!pattern)`
- ⚠ Lookbehind must be in combined patterns

### Flags
- ✓ Global case insensitive: `(?i)`
- ✓ Global multiline anchors: `(?m)`
- ✓ Global DOTALL: `(?s)`
- ✓ Combined global flags: `(?ims)`
- ⚠ Scoped/toggled flags and `x`, `u`, `U`, `R` are rejected as unsupported

### Metacharacters
- ✓ Dot: `.` (matches any character)
- ✓ Greedy: `.*`, `.+`
- ✓ Lazy: `.*?`, `.+?`

## Known Limitations (⚠)

### Range Quantifiers
- ✓ Exact count `{n}` works correctly
- ✓ Bounded range `{n,m}` works correctly (FIXED in 0.4.10)
- ✓ At least N `{n,}` works correctly
- ℹ️ Performance note: ASCII fast path disabled for correctness, using UTF-8 path (~30% slower for bounded quantifiers)

### Lookaround
- ⚠ Standalone lookbehind patterns (e.g., `(?<=a)b`) not supported
- ✓ Lookbehind in combined patterns works
- ✓ Lookahead works in all contexts

### Case Insensitive
- ⚠ `(?i)` flag doesn't work in all contexts (e.g., with complex alternations)
- ✓ Works with simple literals

### Edge Cases
- ⚠ Some empty string patterns may behave unexpectedly
- ⚠ Certain combinations of features may not work (needs more testing)

## Test Results

- **Library tests**: 129/129 passing (100%)
- **Production features**: 49/52 passing (94.2%)
- **Full regex features**: 23/23 passing (100%)
- **Critical features**: 7/7 passing (100%)

## Recommended Usage for Rule Engines

### ✓ Safe to Use
```rust
// Literals and basic patterns
"error|warning|info"
"\\d+\\.\\d+\\.\\d+\\.\\d+"  // IP (without {1,3})

// Word boundaries
"\\bkeyword\\b"
"\\w+@\\w+\\.\\w+"  // Email

// Anchored patterns
"^GET "
"\\.$"

// Character classes
"[A-Z][a-z]+"
"[^\\s]+"

// Quantifiers
"\\w+"
"\\d{2,}"  // Use {n,} not {n,m}

// Lookahead
"\\w+(?=:)"  // Word before colon
"password(?!123)"  // Password not followed by 123
```

### ⚠ Use with Caution
```rust
// Avoid exact/bounded range quantifiers
// DON'T: "\\d{3}-\\d{4}"  
// DO:    "\\d+\\-\\d+"

// Don't use standalone lookbehind
// DON'T: "(?<=@)\\w+"
// DO:    "\\w+@" to match before @

// Case insensitive with simple patterns only
// OK:    "(?i)hello"
// AVOID: "(?i)(GET|POST)"  
```

## Summary

Rexile is **production-ready for rule engine use** with 94%+ feature coverage.
The core features needed for pattern matching, validation, and text processing
all work reliably. Known limitations are documented and have workarounds.

### Strengths
- Fast literal matching
- Full boundary and anchor support
- Working lookahead assertions
- Character classes and ranges
- Alternation and groups
- Capture extraction

### For Production Use
- Use `{n,}` instead of `{n,m}` quantifiers
- Use lookahead, avoid standalone lookbehind
- Test complex patterns before deploying
- All features in the "Safe to Use" section are battle-tested
