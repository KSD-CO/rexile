# ReXile Full Regex Engine - Implementation Roadmap

## Current Status (v0.1)

### ✅ Implemented
- Literal strings (`hello`)
- Alternation (`foo|bar|baz`) - using aho-corasick
- Anchors (`^start`, `end$`, `^exact$`)
- Pattern caching (global cache with OnceLock)
- Find operations (`find`, `find_all`)

### ⏸️ Planned but Not Started
- Character classes `[a-z]`, `[0-9]`, `[^abc]`
- Quantifiers `*`, `+`, `?`, `{n,m}`
- Escape sequences `\.`, `\d`, `\w`, `\s`

## Roadmap to Full Regex Support

### Phase 1: Character Classes (Priority: HIGH)
**Estimated effort:** 2-3 days

**Tasks:**
1. Parse character classes:
   - Simple: `[abc]` → match any of a, b, c
   - Ranges: `[a-z]`, `[0-9]`, `[A-Za-z0-9]`
   - Negation: `[^abc]` → match anything except a, b, c
   - Escape inside: `[\[\]]` → match [ or ]

2. Implement CharClass matcher:
   ```rust
   enum Matcher {
       Literal(String),
       MultiLiteral(AhoCorasick),
       AnchoredLiteral { ... },
       CharClass(CharClassMatcher),  // NEW
   }
   
   struct CharClassMatcher {
       ranges: Vec<(char, char)>,  // e.g., ('a', 'z'), ('0', '9')
       negated: bool,
   }
   ```

3. Fast matching:
   - For ASCII: use 256-bit bitmap (1 bit per byte)
   - For Unicode: use range checks
   - SIMD optimization for sequential char classes

**Files to modify:**
- `src/lib.rs` - add CharClass to AST and Matcher
- Add tests for char classes

### Phase 2: Quantifiers (Priority: HIGH)
**Estimated effort:** 3-5 days

**Tasks:**
1. Parse quantifiers:
   - `*` (0 or more)
   - `+` (1 or more)
   - `?` (0 or 1)
   - `{n}` (exactly n)
   - `{n,}` (n or more)
   - `{n,m}` (between n and m)

2. Implement matching strategies:
   - **Greedy matching** (default): match as much as possible
   - **Lazy matching** (`*?`, `+?`): match as little as possible
   
3. Engine choice:
   - **Option A:** Backtracking NFA (simple, can be slow)
   - **Option B:** Thompson NFA → DFA (faster, more complex)
   - **Option C:** Hybrid (use memchr for literal prefixes + NFA)

4. AST changes:
   ```rust
   enum Ast {
       Literal(String),
       Alternation(Vec<String>),
       Anchored { ... },
       CharClass { ... },
       Quantified {
           inner: Box<Ast>,
           min: usize,
           max: Option<usize>,  // None = unbounded
           greedy: bool,
       },
       Sequence(Vec<Ast>),  // NEW: pattern1 + pattern2
   }
   ```

**Challenges:**
- Backtracking can be exponential (e.g., `(a*)*b` on "aaa...a")
- Need to prevent ReDoS attacks
- May need to limit backtracking depth

**Files to modify:**
- `src/lib.rs` - add Quantified to AST
- `src/nfa.rs` - NEW: NFA engine with backtracking
- Add extensive tests for edge cases

### Phase 3: Escape Sequences (Priority: MEDIUM)
**Estimated effort:** 1-2 days

**Tasks:**
1. Basic escapes:
   - `\.` → literal `.`
   - `\*` → literal `*`
   - `\\` → literal `\`
   - `\n`, `\t`, `\r` → newline, tab, carriage return

2. Character class shortcuts:
   - `\d` → `[0-9]`
   - `\D` → `[^0-9]`
   - `\w` → `[a-zA-Z0-9_]`
   - `\W` → `[^a-zA-Z0-9_]`
   - `\s` → `[ \t\n\r]`
   - `\S` → `[^ \t\n\r]`

3. Parser changes:
   ```rust
   fn parse_escape(input: &str) -> Result<Ast, PatternError> {
       match input.chars().next() {
           Some('d') => Ok(Ast::CharClass { ranges: vec![('0', '9')], negated: false }),
           Some('w') => Ok(Ast::CharClass { ... }),
           Some('n') => Ok(Ast::Literal("\n".to_string())),
           // ...
       }
   }
   ```

**Files to modify:**
- `src/lib.rs` - update `parse_pattern()` to handle escapes

### Phase 4: Advanced Features (Priority: LOW)
**Estimated effort:** 5-10 days

**Optional features (can skip for "lite" version):**

1. **Word boundaries:**
   - `\b` → word boundary
   - `\B` → not word boundary

2. **Lookahead (limited):**
   - `(?=pattern)` → positive lookahead
   - `(?!pattern)` → negative lookahead
   - Challenging to implement efficiently

3. **Capturing groups:**
   - `(pattern)` → capture group
   - Return captures in `find()` result
   - More complex API

4. **Non-capturing groups:**
   - `(?:pattern)` → group without capture

5. **Backreferences:**
   - `\1`, `\2` → reference to capture group
   - Requires backtracking, very slow

### Phase 5: Optimization (Priority: MEDIUM)
**Estimated effort:** Ongoing

**Optimizations:**

1. **Literal prefix optimization:**
   ```rust
   // Pattern: "hello.*world"
   // Fast path: memchr::find("hello"), then match rest
   ```

2. **DFA compilation:**
   - Compile NFA → DFA for hot patterns
   - Cache DFA in global cache
   - Trade memory for speed

3. **SIMD matching:**
   - Use SIMD for character class matching
   - Vectorized string search

4. **Lazy quantifier optimization:**
   - Special case for common patterns like `.*?`

## Implementation Strategy

### Recommended Approach: Incremental

**Week 1: Character Classes**
```rust
// Goal: Support [a-z], [0-9], [^abc]
let pattern = Pattern::new("[a-z]+").unwrap();
assert!(pattern.is_match("hello"));
```

**Week 2: Basic Quantifiers**
```rust
// Goal: Support *, +, ?
let pattern = Pattern::new("a+b*c?").unwrap();
assert!(pattern.is_match("aaabbc"));
```

**Week 3: Escape Sequences**
```rust
// Goal: Support \d, \w, \s, \.
let pattern = Pattern::new(r"\d{3}-\d{3}").unwrap();
assert!(pattern.is_match("123-456"));
```

**Week 4+: Advanced Features**
- Bounded repetition `{n,m}`
- Lazy quantifiers `*?`, `+?`
- Word boundaries `\b`
- (Optional) Groups and captures

## Alternative: Use Existing Regex Engine

### Option A: Fork regex-lite
- regex team has a `regex-lite` crate
- Smaller than full regex
- Already has most features
- Could customize and rebrand

### Option B: Wrap regex with Better API
```rust
pub struct Pattern {
    inner: regex::Regex,
}

impl Pattern {
    pub fn new(pattern: &str) -> Result<Self, PatternError> {
        Ok(Pattern {
            inner: regex::Regex::new(pattern)
                .map_err(|e| PatternError::ParseError(e.to_string()))?
        })
    }
    
    // Simpler API on top of regex
}
```

### Option C: Build Minimal Regex (Current Approach)
- Keep ReXile minimal
- Only add features actually needed for GRL parser
- Don't try to compete with full regex

## Decision Points

### Question 1: Full Regex or Lite?

**Full Regex Pros:**
- ✅ Can replace regex crate completely
- ✅ More useful as general library
- ✅ Learning experience

**Full Regex Cons:**
- ❌ 4-8 weeks of work
- ❌ Will never match regex crate's maturity
- ❌ Maintenance burden
- ❌ Likely slower than regex for complex patterns

**Lite Regex Pros:**
- ✅ Faster to implement (2-3 weeks)
- ✅ Smaller binary size
- ✅ Easier to maintain
- ✅ Focus on common 80% use cases

**Lite Regex Cons:**
- ❌ Not a drop-in regex replacement
- ❌ Users need to check feature support

### Question 2: Performance Target?

**Option A: Match regex crate**
- Very challenging
- Requires DFA compilation, JIT, extensive optimization
- 6+ months of work

**Option B: 80-90% of regex speed**
- Reasonable goal
- Focus on common patterns
- Acceptable trade-off for simplicity

**Option C: Fast for our use cases**
- Optimize for GRL parser patterns
- May be slower for other patterns
- Pragmatic approach

## Recommendation

### 🎯 Recommended Path: Regex-Lite with Practical Features

**Phase 1 (Essential):**
- ✅ Character classes `[a-z]`, `[0-9]`
- ✅ Basic quantifiers `*`, `+`, `?`
- ✅ Escape sequences `\d`, `\w`, `\s`

**Phase 2 (Nice to Have):**
- ⚠️ Bounded quantifiers `{n,m}` (if time permits)
- ⚠️ Word boundaries `\b` (if needed)

**Phase 3 (Skip for Now):**
- ❌ Lookahead/lookbehind (too complex)
- ❌ Backreferences (slow, rarely needed)
- ❌ Full Unicode support (use ranges instead)

**Timeline:**
- Week 1-2: Character classes
- Week 3-4: Quantifiers
- Week 5: Escape sequences
- Week 6: Polish, docs, benchmarks

**Result:**
- 70-80% of common regex use cases covered
- Significantly simpler than full regex
- Good performance for target use cases
- Clear feature matrix (users know what's supported)

## Next Steps

1. **Decide:** Full regex or lite?
2. **Start small:** Implement character classes first
3. **Test extensively:** Each feature needs comprehensive tests
4. **Benchmark:** Compare performance vs regex crate
5. **Document:** Clear feature support matrix

Should we start with Phase 1 (Character Classes)?
