//! Multi-pattern searches with stable IDs and independent capture groups.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::fmt;
use std::ops::ControlFlow;
use std::sync::Arc;

use aho_corasick::{AhoCorasick, AhoCorasickKind, FindOverlappingIter, Input, MatchKind};
use memchr::memmem;

use crate::capture_engine::{CaptureState, CaseFoldedText};
use crate::set_program::Program;
use crate::{
    char_boundaries, literal_from_ast, prefix_analysis, Ast, Captures, Flags, Match, Matcher,
    Pattern, PatternError, SearchProgress,
};

/// Failure to compile a pattern set. No partial set is returned.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PatternSetError {
    /// An input pattern failed to compile.
    Pattern {
        /// Zero-based input index.
        pattern_id: usize,
        /// The original pattern error.
        source: PatternError,
    },
    /// The shared literal index could not be built.
    Build(String),
}
impl fmt::Display for PatternSetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pattern { pattern_id, source } => write!(f, "pattern {pattern_id}: {source}"),
            Self::Build(message) => write!(f, "pattern set index: {message}"),
        }
    }
}
impl std::error::Error for PatternSetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Pattern { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Whether to visit the first match of each rule or every independent match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetSearchMode {
    /// At most one match per pattern.
    FirstPerPattern,
    /// Non-overlapping matches within each pattern; overlaps across patterns.
    All,
}

/// A bit set of matched pattern IDs, iterated in ascending input order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SetMatches {
    first: u64,
    words: Vec<u64>,
}
impl SetMatches {
    fn reset(&mut self, count: usize) {
        self.first = 0;
        if count <= 64 {
            self.words.clear();
            return;
        }
        self.words.resize((count - 64 + 63) / 64, 0);
        self.words.fill(0);
    }
    fn insert(&mut self, id: usize) {
        if id < 64 {
            self.first |= 1 << id;
        } else {
            self.words[id / 64 - 1] |= 1 << (id % 64);
        }
    }
    /// Whether a pattern matched. Out-of-range IDs return false.
    pub fn matched(&self, id: usize) -> bool {
        if id < 64 {
            self.first & (1 << id) != 0
        } else {
            self.words
                .get(id / 64 - 1)
                .is_some_and(|word| word & (1 << (id % 64)) != 0)
        }
    }
    /// Whether any pattern matched.
    pub fn matched_any(&self) -> bool {
        self.first != 0 || self.words.iter().any(|&word| word != 0)
    }
    /// Number of matched IDs.
    pub fn len(&self) -> usize {
        self.first.count_ones() as usize
            + self
                .words
                .iter()
                .map(|word| word.count_ones() as usize)
                .sum::<usize>()
    }

    /// Whether no patterns matched.
    pub fn is_empty(&self) -> bool {
        !self.matched_any()
    }
    /// Matched IDs, without allocating an intermediate list.
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        std::iter::once(&self.first)
            .chain(self.words.iter())
            .enumerate()
            .flat_map(|(block, &word)| {
                let mut remaining = word;
                std::iter::from_fn(move || {
                    if remaining == 0 {
                        return None;
                    }
                    let bit = remaining.trailing_zeros() as usize;
                    remaining &= remaining - 1;
                    Some(block * 64 + bit)
                })
            })
    }
}

/// A match together with its zero-based pattern ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetMatch<'h> {
    pattern_id: usize,
    matched: Match<'h>,
}
impl<'h> SetMatch<'h> {
    /// The rule's position in the constructor input.
    pub fn pattern_id(&self) -> usize {
        self.pattern_id
    }
    /// The corresponding ordinary match.
    pub fn matched(&self) -> Match<'h> {
        self.matched
    }
    /// Starting byte offset in the original haystack.
    pub fn start(&self) -> usize {
        self.matched.start()
    }
    /// Exclusive ending byte offset in the original haystack.
    pub fn end(&self) -> usize {
        self.matched.end()
    }
    /// Matched source text.
    pub fn as_str(&self) -> &'h str {
        self.matched.as_str()
    }
    /// Byte range in the original haystack.
    pub fn range(&self) -> std::ops::Range<usize> {
        self.matched.range()
    }
}

/// Owned capture positions with borrowed source text, for one rule.
#[derive(Debug, Clone)]
pub struct SetCaptures<'h> {
    pattern_id: usize,
    captures: Captures<'h>,
}
impl<'h> SetCaptures<'h> {
    /// The rule's position in the constructor input.
    pub fn pattern_id(&self) -> usize {
        self.pattern_id
    }
    /// Capture groups, numbered independently for this rule.
    pub fn captures(&self) -> &Captures<'h> {
        &self.captures
    }
    /// A group's source text; zero is the full match.
    pub fn get(&self, group: usize) -> Option<&'h str> {
        self.captures.get(group)
    }
    /// A group's byte offsets in the original haystack.
    pub fn pos(&self, group: usize) -> Option<(usize, usize)> {
        self.captures.pos(group)
    }
}

/// Borrowed capture slots, valid for the duration of a visitor callback.
#[derive(Debug, Clone, Copy)]
pub struct SetCapturesRef<'c, 'h> {
    pattern_id: usize,
    text: &'h str,
    positions: &'c [Option<(usize, usize)>],
}
impl<'c, 'h> SetCapturesRef<'c, 'h> {
    /// The rule's position in the constructor input.
    pub fn pattern_id(&self) -> usize {
        self.pattern_id
    }
    /// A group's source text; an absent group returns None.
    pub fn get(&self, group: usize) -> Option<&'h str> {
        let (start, end) = self.pos(group)?;
        self.text.get(start..end)
    }
    /// A group's source byte offsets.
    pub fn pos(&self, group: usize) -> Option<(usize, usize)> {
        self.positions.get(group).copied().flatten()
    }
    /// Number of capture slots, including group zero.
    pub fn len(&self) -> usize {
        self.positions.len()
    }
    /// Whether the view contains no slots.
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
    /// Copy positions when a result must outlive the callback.
    pub fn to_owned(&self) -> SetCaptures<'h> {
        SetCaptures {
            pattern_id: self.pattern_id,
            captures: Captures::from_positions(self.text, self.positions.to_vec()),
        }
    }
}

#[derive(Debug)]
struct Rule {
    pattern: Option<Box<Pattern>>,
    group_count: usize,
    case_insensitive: bool,
    literal: Option<String>,
    program: Option<Box<Program>>,
    indexed: bool,
    anchor: Option<Box<(usize, memmem::Finder<'static>)>>,
}
#[derive(Debug)]
struct CompiledSet {
    patterns: Vec<String>,
    rules: Vec<Rule>,
    index: Option<AhoCorasick>,
    fixed_index: Option<HashMap<u128, usize>>,
    first_fixed_key: u128,
    owners: Vec<Vec<usize>>,
    max_prefix_len: usize,
    prefix_width: Option<usize>,
    first_bytes: Vec<u8>,
    has_case_insensitive: bool,
    all_indexed: bool,
    fully_filtered: bool,
    has_anchors: bool,
    fallback_ids: Vec<usize>,
    deduplicate_candidates: bool,
    literal_only: bool,
    quick_boolean: bool,
    uniform_prefixes: bool,
    rejection: Option<memmem::Finder<'static>>,
}

/// Immutable compiled patterns sharing a literal index.
///
/// IDs follow input order, including duplicate patterns. Iterators merge each
/// rule's non-overlapping matches by (start, pattern_id), retaining overlaps
/// between rules. Capture groups are local to their rule.
///
///     use rexile::PatternSet;
///     let set = PatternSet::new([r"user=(\w+)", r"code=(\d+)"]).unwrap();
///     let hits: Vec<_> = set.captures_iter("code=42 user=alice").collect();
///     assert_eq!(hits[0].pattern_id(), 1);
///     assert_eq!(hits[0].get(1), Some("42"));
///     assert_eq!(hits[1].get(1), Some("alice"));
#[derive(Debug, Clone)]
pub struct PatternSet {
    inner: Arc<CompiledSet>,
}

/// Reusable search memory, usable with another set or its clones.
///
/// Caches hold no haystack references. Give each concurrent search its own
/// cache; cloning a set shares only immutable compiled state.
#[derive(Debug)]
pub struct SetCache {
    progress: Vec<SearchProgress>,
    verified: Vec<usize>,
    heap: BinaryHeap<Reverse<(usize, usize)>>,
    captures: CaptureState,
    whole_match: [Option<(usize, usize)>; 1],
    literal_captures: bool,
    folded: CaseFoldedText,
    matches: SetMatches,
    eligible: SetMatches,
    ascii: bool,
}
impl Default for SetCache {
    fn default() -> Self {
        Self {
            progress: Vec::new(),
            verified: Vec::new(),
            heap: BinaryHeap::new(),
            captures: CaptureState::default(),
            whole_match: [None],
            literal_captures: false,
            folded: CaseFoldedText::default(),
            matches: SetMatches::default(),
            eligible: SetMatches::default(),
            ascii: true,
        }
    }
}
impl SetCache {
    fn reset(&mut self, set: &PatternSet, text: &str) {
        // Initialize each slot once, including a newly allocated cache.
        self.progress.clear();
        self.progress
            .resize_with(set.len(), SearchProgress::default);
        if set.inner.deduplicate_candidates {
            self.verified.clear();
            self.verified.resize(set.len(), usize::MAX);
        }
        self.heap.clear();
        self.matches.reset(set.len());
        self.eligible.reset(set.len());
        self.ascii = text.is_ascii();
        if set.inner.has_case_insensitive {
            self.folded.reset(text);
        }
    }

    fn positions(&self) -> &[Option<(usize, usize)>] {
        if self.literal_captures {
            &self.whole_match
        } else {
            &self.captures.positions
        }
    }
}

impl PatternSet {
    /// Compile all patterns, or report the first invalid input by ID.
    pub fn new<I, S>(patterns: I) -> Result<Self, PatternSetError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut sources = Vec::new();
        let mut rules = Vec::new();
        let mut prefixes = Vec::new();
        let mut prefix_ids = HashMap::new();
        let mut owners: Vec<Vec<usize>> = Vec::new();
        let mut has_case_insensitive = false;
        for (id, source) in patterns.into_iter().enumerate() {
            let source = source.as_ref();
            let (ast, flags, effective_pattern) = if !source.contains([
                '\\', '.', '^', '$', '*', '+', '?', '(', ')', '[', ']', '{', '}', '|',
            ]) {
                (Ast::Literal(source.to_owned()), Flags::default(), source)
            } else {
                let (ast, flags, effective_pattern) =
                    Pattern::parse(source).map_err(|source| PatternSetError::Pattern {
                        pattern_id: id,
                        source,
                    })?;
                let ast = literal_from_ast(&ast).map(Ast::Literal).unwrap_or(ast);
                (ast, flags, effective_pattern)
            };
            let case_insensitive = flags.case_insensitive;
            has_case_insensitive |= case_insensitive;
            let program = match &ast {
                Ast::Literal(_) => None,
                Ast::CaseInsensitive(inner) => Program::compile(&crate::lowercase_ast(inner)),
                _ => Program::compile(&ast),
            };
            let analysis = if case_insensitive {
                None
            } else {
                prefix_analysis(&ast)
            };
            let mut indexed = false;
            if let Some(analysis) = analysis {
                if !analysis.literals.is_empty() && analysis.literals.iter().all(|s| !s.is_empty())
                {
                    indexed = true;
                    for mut prefix in analysis.literals {
                        // Bound reordering without changing possible start positions.
                        // The complete rule still verifies each candidate.
                        let mut length = prefix.len().min(64);
                        while !prefix.is_char_boundary(length) {
                            length -= 1;
                        }
                        prefix.truncate(length);
                        let next_id = prefixes.len();
                        let prefix_id = *prefix_ids.entry(prefix.clone()).or_insert_with(|| {
                            prefixes.push(prefix);
                            owners.push(Vec::new());
                            next_id
                        });
                        if owners[prefix_id].last() != Some(&id) {
                            owners[prefix_id].push(id);
                        }
                    }
                }
            }
            let anchor = if !indexed && !case_insensitive {
                program
                    .as_ref()
                    .and_then(Program::anchor)
                    .map(|(op, literal)| {
                        let next_id = prefixes.len();
                        let prefix_id =
                            *prefix_ids.entry(literal.to_owned()).or_insert_with(|| {
                                prefixes.push(literal.to_owned());
                                owners.push(Vec::new());
                                next_id
                            });
                        owners[prefix_id].push(id);
                        Box::new((op, memmem::Finder::new(literal.as_bytes()).into_owned()))
                    })
            } else {
                None
            };
            let pattern = if matches!(ast, Ast::Literal(_))
                || program.as_ref().is_some_and(|program| program.unicode_safe)
            {
                None
            } else {
                Some(Box::new(
                    Pattern::from_ast::<false>(source, effective_pattern, flags, &ast).map_err(
                        |source| PatternSetError::Pattern {
                            pattern_id: id,
                            source,
                        },
                    )?,
                ))
            };
            let group_count = program.as_ref().map_or_else(
                || {
                    pattern
                        .as_ref()
                        .map_or(0, |pattern| pattern.capture_group_count)
                },
                Program::group_count,
            );
            let literal = match ast {
                Ast::Literal(literal) => Some(literal),
                _ => None,
            };
            let program = if literal.is_some() {
                None
            } else {
                program.map(Box::new)
            };
            sources.push(source.to_owned());
            rules.push(Rule {
                pattern,
                group_count,
                case_insensitive,
                literal,
                program,
                indexed,
                anchor,
            });
        }
        let prefix_width = prefixes
            .first()
            .map(String::len)
            .filter(|width| prefixes.iter().all(|prefix| prefix.len() == *width));
        let mut first_bytes: Vec<_> = prefixes
            .iter()
            .filter_map(|prefix| prefix.as_bytes().first().copied())
            .collect();
        first_bytes.sort_unstable();
        first_bytes.dedup();
        let max_prefix_len = prefixes.iter().map(String::len).max().unwrap_or(0);
        let fixed_index = prefix_width
            .filter(|&width| width <= 16 && first_bytes.len() <= 3)
            .map(|_| {
                prefixes
                    .iter()
                    .enumerate()
                    .map(|(id, literal)| (fixed_key(literal.as_bytes()), id))
                    .collect::<HashMap<_, _>>()
            });
        let first_fixed_key = if fixed_index.is_some() {
            prefixes
                .first()
                .map_or(0, |prefix| fixed_key(prefix.as_bytes()))
        } else {
            0
        };
        let index = if prefixes.is_empty() || fixed_index.is_some() {
            None
        } else {
            Some(
                AhoCorasick::builder()
                    .match_kind(MatchKind::Standard)
                    .prefilter(false)
                    .kind(
                        (prefixes.iter().map(String::len).sum::<usize>() <= 65_536)
                            .then_some(AhoCorasickKind::DFA),
                    )
                    .build(&prefixes)
                    .map_err(|error| PatternSetError::Build(error.to_string()))?,
            )
        };
        let all_indexed = rules.iter().all(|rule| rule.indexed);
        let fully_filtered = rules
            .iter()
            .all(|rule| rule.indexed || rule.anchor.is_some());
        let has_anchors = rules.iter().any(|rule| rule.anchor.is_some());
        let fallback_ids = rules
            .iter()
            .enumerate()
            .filter_map(|(id, rule)| (!rule.indexed && rule.anchor.is_none()).then_some(id))
            .collect();
        let literal_only = all_indexed
            && rules
                .iter()
                .all(|rule| rule.literal.as_ref().is_some_and(|value| value.len() <= 64));
        let uniform_prefixes =
            all_indexed && prefixes.iter().all(|prefix| prefix.len() == max_prefix_len);
        let mut owner_counts = vec![0usize; rules.len()];
        for ids in &owners {
            for &id in ids {
                owner_counts[id] += 1;
            }
        }
        let deduplicate_candidates = owner_counts.iter().any(|&count| count > 1);
        let quick_boolean = fully_filtered
            && !has_case_insensitive
            && rules.iter().all(|rule| {
                rule.literal.is_some()
                    || rule
                        .program
                        .as_ref()
                        .is_some_and(|program| program.unicode_safe)
            });
        let rejection = if fully_filtered {
            prefixes.first().and_then(|first| {
                let mut length = first.len();
                for prefix in &prefixes[1..] {
                    length = first
                        .bytes()
                        .zip(prefix.bytes())
                        .take(length)
                        .take_while(|(a, b)| a == b)
                        .count();
                }
                while !first.is_char_boundary(length) {
                    length -= 1;
                }
                (length > 0).then(|| memmem::Finder::new(&first.as_bytes()[..length]).into_owned())
            })
        } else {
            None
        };
        Ok(Self {
            inner: Arc::new(CompiledSet {
                patterns: sources,
                rules,
                index,
                fixed_index,
                first_fixed_key,
                owners,
                max_prefix_len,
                prefix_width,
                first_bytes,
                has_case_insensitive,
                all_indexed,
                fully_filtered,
                has_anchors,
                fallback_ids,
                deduplicate_candidates,
                literal_only,
                quick_boolean,
                uniform_prefixes,
                rejection,
            }),
        })
    }
    /// Number of input patterns.
    pub fn len(&self) -> usize {
        self.inner.rules.len()
    }
    /// Whether the set contains no patterns.
    pub fn is_empty(&self) -> bool {
        self.inner.rules.is_empty()
    }
    /// Pattern source strings in ID order.
    pub fn patterns(&self) -> &[String] {
        &self.inner.patterns
    }
    /// Allocate reusable per-search buffers.
    pub fn create_cache(&self) -> SetCache {
        let mut cache = SetCache::default();
        cache.reset(self, "");
        cache
    }
    /// Whether any rule matches.
    pub fn is_match(&self, text: &str) -> bool {
        if self.inner.literal_only {
            return self.first_at_start(text).is_some() || self.scanner(text).next().is_some();
        }
        if self.reject(text) {
            return false;
        }
        if self.inner.quick_boolean {
            return self.quick_is_match(text);
        }
        self.is_match_with_cache(text, &mut SetCache::default())
    }
    /// Whether any rule matches, reusing buffers and stopping at the first hit.
    pub fn is_match_with_cache(&self, text: &str, cache: &mut SetCache) -> bool {
        if self.inner.literal_only {
            return self.first_at_start(text).is_some() || self.scanner(text).next().is_some();
        }
        if self.inner.quick_boolean {
            return !self.reject(text) && self.quick_is_match(text);
        }
        self.collect_ids(text, cache, true)
    }
    /// All matched IDs in ascending input order.
    pub fn matches(&self, text: &str) -> SetMatches {
        if self.inner.quick_boolean {
            let mut matches = SetMatches::default();
            self.collect_quick_ids(text, &mut matches);
            return matches;
        }
        let mut cache = SetCache::default();
        self.collect_ids(text, &mut cache, false);
        cache.matches
    }
    /// All matched IDs, borrowed from reusable cache memory.
    pub fn matches_with_cache<'c>(&self, text: &str, cache: &'c mut SetCache) -> &'c SetMatches {
        if self.inner.quick_boolean {
            self.collect_quick_ids(text, &mut cache.matches);
        } else {
            self.collect_ids(text, cache, false);
        }
        &cache.matches
    }
    fn scanner<'s, 'h>(&'s self, text: &'h str) -> CandidateScanner<'s, 'h> {
        let width = self
            .inner
            .prefix_width
            .filter(|_| self.inner.rejection.is_some() || self.inner.first_bytes.len() <= 3);
        CandidateScanner {
            index: self.inner.index.as_ref(),
            fixed_index: self.inner.fixed_index.as_ref(),
            text,
            width,
            from: 0,
            first_bytes: &self.inner.first_bytes,
            hint: self.inner.rejection.as_ref(),
            dense_start: None,
            dense_hits: 0,
            recent: if self.inner.fixed_index.is_some() {
                // Prefix IDs follow compilation order; this mapping is known
                // without a hash lookup, including the first search candidate.
                PrefixCache::One(self.inner.first_fixed_key, 0)
            } else {
                PrefixCache::Empty
            },
            overlapping: if width.is_none() {
                self.inner
                    .index
                    .as_ref()
                    .map(|index| index.find_overlapping_iter(text))
            } else {
                None
            },
        }
    }
    fn quick_candidate(&self, text: &str, id: usize, candidate: Candidate) -> bool {
        let rule = &self.inner.rules[id];
        if let Some(literal) = &rule.literal {
            return text
                .get(candidate.start()..)
                .is_some_and(|tail| tail.starts_with(literal));
        }
        rule.program.as_ref().is_some_and(|program| {
            program.verify_candidate(
                text,
                candidate.start(),
                candidate.len(),
                rule.anchor.as_deref().map(|(op, _)| *op),
            )
        })
    }
    fn quick_is_match(&self, text: &str) -> bool {
        // Try a cheap initial rule at the start before paying for the shared
        // unanchored search. A failure still searches every rule normally.
        if let Some(rule) = self.inner.rules.first() {
            if rule
                .program
                .as_ref()
                .is_some_and(|program| program.match_at(text, 0, &mut []).is_some())
            {
                return true;
            }
        }
        if self.inner.index.is_some() || self.inner.fixed_index.is_some() {
            for candidate in self.scanner(text) {
                for &id in &self.inner.owners[candidate.prefix_id] {
                    if self.quick_candidate(text, id, candidate) {
                        return true;
                    }
                }
            }
        }
        false
    }
    fn reject(&self, text: &str) -> bool {
        self.is_empty()
            || self
                .inner
                .rejection
                .as_ref()
                .is_some_and(|finder| finder.find(text.as_bytes()).is_none())
    }
    // Use one hot loop for cached and owned IDs, without constructing capture
    // or iterator buffers when the compiler proved they are unnecessary.
    #[inline(never)]
    fn collect_quick_ids(&self, text: &str, matches: &mut SetMatches) {
        matches.reset(self.len());
        if self.reject(text) {
            return;
        }
        if self.inner.literal_only {
            if self.inner.index.is_some() || self.inner.fixed_index.is_some() {
                for event in self.scanner(text) {
                    for &id in &self.inner.owners[event.prefix_id] {
                        matches.insert(id);
                    }
                }
            }
            return;
        }
        for candidate in self.scanner(text) {
            for &id in &self.inner.owners[candidate.prefix_id] {
                if !matches.matched(id) && self.quick_candidate(text, id, candidate) {
                    matches.insert(id);
                }
            }
        }
    }

    fn collect_ids(&self, text: &str, cache: &mut SetCache, stop: bool) -> bool {
        cache.reset(self, text);
        if self.reject(text) {
            return false;
        }
        if self.inner.index.is_some() || self.inner.fixed_index.is_some() {
            for candidate in self.scanner(text) {
                for &id in &self.inner.owners[candidate.prefix_id] {
                    if cache.matches.matched(id) {
                        continue;
                    }
                    let start = if let Some((op, _)) = self.inner.rules[id].anchor.as_deref() {
                        cache.eligible.insert(id);
                        if !cache.ascii {
                            continue;
                        }
                        self.inner.rules[id].program.as_ref().and_then(|program| {
                            program.candidate_start(text, *op, candidate.start())
                        })
                    } else {
                        Some(candidate.start())
                    };
                    if start
                        .and_then(|start| self.exact(text, id, start, cache, false))
                        .is_some()
                    {
                        cache.matches.insert(id);
                        if stop {
                            return true;
                        }
                    }
                }
            }
        }
        for (id, rule) in self.inner.rules.iter().enumerate() {
            if !rule.indexed
                && (rule.anchor.is_none() || (!cache.ascii && cache.eligible.matched(id)))
                && self.next_fallback(text, id, cache).is_some()
            {
                cache.matches.insert(id);
                if stop {
                    return true;
                }
            }
        }
        cache.matches.matched_any()
    }
    fn exact(
        &self,
        text: &str,
        id: usize,
        start: usize,
        cache: &mut SetCache,
        capture_groups: bool,
    ) -> Option<usize> {
        if !text.is_char_boundary(start) {
            return None;
        }
        let rule = &self.inner.rules[id];
        if capture_groups {
            cache.literal_captures = rule.literal.is_some();
        }
        if let Some(literal) = &rule.literal {
            if !text.get(start..)?.starts_with(literal) {
                return None;
            }
            let end = start + literal.len();
            if capture_groups {
                cache.whole_match[0] = Some((start, end));
            }
            return Some(end);
        }
        let program = rule
            .program
            .as_ref()
            .filter(|program| cache.ascii || program.unicode_safe);
        // Unicode folding can create boundaries inside one source character.
        // Keep all slots there so find and captures reject the same candidates.
        let need_slots =
            capture_groups || program.is_none() || (rule.case_insensitive && !cache.ascii);
        if need_slots {
            cache.captures.reset(rule.group_count);
        }
        let (match_text, match_start) = if rule.case_insensitive {
            (cache.folded.text(), cache.folded.folded_offset(start)?)
        } else {
            (text, start)
        };
        let end = match program {
            Some(program) => program.match_at(
                match_text,
                match_start,
                if need_slots {
                    &mut cache.captures.positions
                } else {
                    &mut []
                },
            ),
            _ => {
                let pattern = rule.pattern.as_ref()?;
                let matcher = match &pattern.matcher {
                    Matcher::CaseInsensitive(inner) => inner.as_ref(),
                    matcher => matcher,
                };
                matcher.match_at_with_captures(match_text, match_start, &mut cache.captures)
            }
        }?;
        if !need_slots {
            return Some(end);
        }
        cache.captures.positions[0] = Some((match_start, end));
        if !rule.case_insensitive && program.is_some() {
            // The program consumes ASCII runs and complete literal strings;
            // each emitted capture position is already a UTF-8 boundary.
            return Some(end);
        }
        if rule.case_insensitive {
            cache.folded.map_positions(&mut cache.captures.positions)?;
        }
        let (_, source_end) = cache.captures.positions[0]?;
        if cache
            .captures
            .positions
            .iter()
            .flatten()
            .any(|&(s, e)| s > e || text.get(s..e).is_none())
        {
            return None;
        }
        Some(source_end)
    }
    fn next_fallback(&self, text: &str, id: usize, cache: &mut SetCache) -> Option<usize> {
        if cache.progress[id].finished {
            return None;
        }
        let rule = &self.inner.rules[id];
        if let Some((op, finder)) = rule.anchor.as_deref() {
            if !cache.eligible.matched(id) {
                cache.progress[id].finished = true;
                return None;
            }
            if cache.ascii {
                let mut from = cache.progress[id].pos;
                while from <= text.len() {
                    let Some(relative) = finder.find(&text.as_bytes()[from..]) else {
                        break;
                    };
                    let anchor_start = from + relative;
                    from = anchor_start + 1;
                    let Some(start) = rule
                        .program
                        .as_ref()
                        .and_then(|program| program.candidate_start(text, *op, anchor_start))
                    else {
                        continue;
                    };
                    if start < cache.progress[id].pos {
                        continue;
                    }
                    if self.exact(text, id, start, cache, false).is_some() {
                        return Some(start);
                    }
                }
                cache.progress[id].finished = true;
                return None;
            }
        }
        for start in char_boundaries(text, cache.progress[id].pos) {
            if let Some(end) = self.exact(text, id, start, cache, false) {
                if start == end
                    && (cache.progress[id].suppress_empty && cache.progress[id].pos == end)
                {
                    continue;
                }
                return Some(start);
            }
        }
        cache.progress[id].finished = true;
        None
    }

    fn collect_first<T>(
        &self,
        text: &str,
        captures: bool,
        mut make: impl FnMut(usize, usize, usize, &[Option<(usize, usize)>]) -> T,
        id_of: impl Fn(&T) -> usize,
    ) -> Vec<T> {
        let mut output = Vec::new();
        let mut scanner = self.scanner(text);
        let Some(first) = scanner.next() else {
            return output;
        };
        // One indexed literal per rule gives monotonically increasing starts
        // for that rule, even when different rules have different widths.
        let ordered = self.inner.prefix_width.is_some() || !self.inner.deduplicate_candidates;
        let mut seen = SetMatches::default();
        let mut positions = if ordered {
            seen.reset(self.len());
            Vec::new()
        } else {
            vec![(usize::MAX, usize::MAX); self.len()]
        };
        let mut slots = CaptureState::default();
        for event in std::iter::once(first).chain(scanner) {
            for &id in &self.inner.owners[event.prefix_id] {
                if ordered && seen.matched(id) {
                    continue;
                }
                let rule = &self.inner.rules[id];
                // Fixed-width prefixes arrive in start order. An internal
                // anchor has one indexed literal per rule, and each reversed
                // literal/run maps increasing anchor offsets to nondecreasing
                // starts. Neither case can improve an already verified hit.
                if !ordered && rule.anchor.is_some() && positions[id].1 != usize::MAX {
                    continue;
                }
                let start = match rule.anchor.as_deref() {
                    Some((op, _)) => rule
                        .program
                        .as_ref()
                        .and_then(|program| program.candidate_start(text, *op, event.start())),
                    None => Some(event.start()),
                };
                let Some(start) = start else {
                    continue;
                };
                if !ordered && start >= positions[id].0 {
                    continue;
                }
                if captures {
                    slots.reset(rule.group_count);
                }
                let end = if let Some(literal) = &rule.literal {
                    text.get(start..)
                        .filter(|tail| tail.starts_with(literal))
                        .map(|_| start + literal.len())
                } else {
                    rule.program.as_ref().and_then(|program| {
                        program.match_at(
                            text,
                            start,
                            if captures {
                                &mut slots.positions
                            } else {
                                &mut []
                            },
                        )
                    })
                };
                let Some(end) = end else {
                    continue;
                };
                if captures {
                    slots.positions[0] = Some((start, end));
                }
                let hit = make(id, start, end, &slots.positions);
                if ordered {
                    seen.insert(id);
                    output.push(hit);
                    continue;
                }
                let position = if positions[id].1 == usize::MAX {
                    let position = output.len();
                    output.push(hit);
                    position
                } else {
                    output[positions[id].1] = hit;
                    positions[id].1
                };
                positions[id] = (start, position);
            }
        }
        output.sort_unstable_by_key(id_of);
        output
    }
    /// Earliest match; ties go to the smallest pattern ID.
    pub fn find<'h>(&self, text: &'h str) -> Option<SetMatch<'h>> {
        if let Some(end) = self.first_at_start(text) {
            return Some(SetMatch {
                pattern_id: 0,
                matched: Match::new(text, 0, end),
            });
        }
        self.find_iter(text).next()
    }

    fn first_at_start(&self, text: &str) -> Option<usize> {
        if !self.inner.quick_boolean {
            return None;
        }
        let rule = self.inner.rules.first()?;
        if let Some(literal) = &rule.literal {
            text.starts_with(literal).then_some(literal.len())
        } else {
            rule.program.as_ref()?.match_at(text, 0, &mut [])
        }
    }
    /// The first match of every matching rule, in ID order.
    pub fn find_each<'h>(&self, text: &'h str) -> Vec<SetMatch<'h>> {
        if self.inner.quick_boolean {
            return self.collect_first(
                text,
                false,
                |pattern_id, start, end, _| SetMatch {
                    pattern_id,
                    matched: Match::new(text, start, end),
                },
                SetMatch::pattern_id,
            );
        }
        let mut result = Vec::new();
        let _ = self.visit_matches(
            text,
            &mut SetCache::default(),
            SetSearchMode::FirstPerPattern,
            |hit| {
                result.push(hit);
                ControlFlow::<()>::Continue(())
            },
        );
        result.sort_unstable_by_key(SetMatch::pattern_id);
        result
    }
    /// Lazily merge independent matches by (start, pattern_id).
    pub fn find_iter<'s, 'h>(&'s self, text: &'h str) -> SetFindIter<'s, 'h> {
        let mut cache = SetCache::default();
        let cursor = Cursor::new(self, text, &mut cache, SetSearchMode::All);
        SetFindIter { cursor, cache }
    }
    /// Captures of the earliest match; ties go to the smallest pattern ID.
    pub fn captures<'h>(&self, text: &'h str) -> Option<SetCaptures<'h>> {
        if let Some(end) = self.first_at_start(text) {
            let rule = &self.inner.rules[0];
            let mut positions = vec![None; rule.group_count + 1];
            if let Some(program) = &rule.program {
                program.match_at(text, 0, &mut positions)?;
            }
            positions[0] = Some((0, end));
            return Some(SetCaptures {
                pattern_id: 0,
                captures: Captures::from_positions(text, positions),
            });
        }
        self.captures_iter(text).next()
    }
    /// Captures of the first match of each matching rule, in ID order.
    pub fn captures_each<'h>(&self, text: &'h str) -> Vec<SetCaptures<'h>> {
        if self.inner.quick_boolean {
            return self.collect_first(
                text,
                true,
                |pattern_id, _, _, positions| SetCaptures {
                    pattern_id,
                    captures: Captures::from_positions(text, positions.to_vec()),
                },
                SetCaptures::pattern_id,
            );
        }
        let mut result = Vec::new();
        let _ = self.visit_captures(
            text,
            &mut SetCache::default(),
            SetSearchMode::FirstPerPattern,
            |hit| {
                result.push(hit.to_owned());
                ControlFlow::<()>::Continue(())
            },
        );
        result.sort_unstable_by_key(SetCaptures::pattern_id);
        result
    }
    /// Lazily merge owned captures, preserving overlaps between rules.
    pub fn captures_iter<'s, 'h>(&'s self, text: &'h str) -> SetCapturesIter<'s, 'h> {
        let mut matches = self.find_iter(text);
        matches.cursor.capture_groups = true;
        SetCapturesIter { matches }
    }
    /// Visit matches using reusable memory; propagate the callback's break value.
    pub fn visit_matches<'h, B>(
        &self,
        text: &'h str,
        cache: &mut SetCache,
        mode: SetSearchMode,
        mut visitor: impl FnMut(SetMatch<'h>) -> ControlFlow<B>,
    ) -> ControlFlow<B> {
        let mut cursor = Cursor::new(self, text, cache, mode);
        while let Some(hit) = cursor.next(cache) {
            if let ControlFlow::Break(value) = visitor(hit) {
                return ControlFlow::Break(value);
            }
        }
        ControlFlow::Continue(())
    }
    /// Visit borrowed captures without allocating an owned result per hit.
    ///
    ///     use std::ops::ControlFlow;
    ///     use rexile::{PatternSet, SetSearchMode};
    ///     let set = PatternSet::new([r"code=(\d+)"]).unwrap();
    ///     let mut cache = set.create_cache();
    ///     let mut values = Vec::new();
    ///     let _ = set.visit_captures("code=42 code=7", &mut cache, SetSearchMode::All, |hit| {
    ///         values.push(hit.get(1).unwrap());
    ///         ControlFlow::<()>::Continue(())
    ///     });
    ///     assert_eq!(values, ["42", "7"]);
    pub fn visit_captures<'h, B>(
        &self,
        text: &'h str,
        cache: &mut SetCache,
        mode: SetSearchMode,
        mut visitor: impl FnMut(SetCapturesRef<'_, 'h>) -> ControlFlow<B>,
    ) -> ControlFlow<B> {
        let mut cursor = Cursor::new(self, text, cache, mode);
        cursor.capture_groups = true;
        while let Some(hit) = cursor.next(cache) {
            let captures = SetCapturesRef {
                pattern_id: hit.pattern_id,
                text,
                positions: cache.positions(),
            };
            if let ControlFlow::Break(value) = visitor(captures) {
                return ControlFlow::Break(value);
            }
        }
        ControlFlow::Continue(())
    }
}

// Used only for equal-width index entries of at most sixteen bytes.
fn fixed_key(bytes: &[u8]) -> u128 {
    let mut key = [0; 16];
    key[..bytes.len()].copy_from_slice(bytes);
    u128::from_le_bytes(key)
}

#[derive(Clone, Copy)]
struct Candidate {
    prefix_id: usize,
    start: usize,
    end: usize,
}
impl Candidate {
    fn start(self) -> usize {
        self.start
    }
    fn end(self) -> usize {
        self.end
    }
    fn len(self) -> usize {
        self.end - self.start
    }
}
impl From<aho_corasick::Match> for Candidate {
    fn from(hit: aho_corasick::Match) -> Self {
        Self {
            prefix_id: hit.pattern().as_usize(),
            start: hit.start(),
            end: hit.end(),
        }
    }
}

// Initialize the larger cache only when more than one prefix was observed.
// A one-hit query should not pay to initialize four empty cache entries.
enum PrefixCache {
    Empty,
    One(u128, usize),
    Recent {
        keys: [u128; 4],
        ids: [usize; 4],
        next: usize,
    },
}
impl PrefixCache {
    #[inline]
    fn get(&mut self, index: &HashMap<u128, usize>, key: u128) -> Option<usize> {
        match self {
            Self::Empty => {
                let &id = index.get(&key)?;
                *self = Self::One(key, id);
                Some(id)
            }
            Self::One(previous, id) if *previous == key => Some(*id),
            Self::One(previous, previous_id) => {
                let &id = index.get(&key)?;
                *self = Self::Recent {
                    keys: [*previous, key, 0, 0],
                    ids: [*previous_id, id, usize::MAX, usize::MAX],
                    next: 2,
                };
                Some(id)
            }
            Self::Recent { keys, ids, next } => {
                for slot in 0..keys.len() {
                    if keys[slot] == key && ids[slot] != usize::MAX {
                        return Some(ids[slot]);
                    }
                }
                let &id = index.get(&key)?;
                keys[*next] = key;
                ids[*next] = id;
                *next = (*next + 1) % keys.len();
                Some(id)
            }
        }
    }
}

// Equal-width prefixes cannot represent two different literals at one start.
// SIMD finds possible starts; an anchored lookup checks the entire prefix and
// reports all duplicate owners through the index's owner table.
struct CandidateScanner<'s, 'h> {
    index: Option<&'s AhoCorasick>,
    fixed_index: Option<&'s HashMap<u128, usize>>,
    text: &'h str,
    overlapping: Option<FindOverlappingIter<'s, 'h>>,
    width: Option<usize>,
    from: usize,
    first_bytes: &'s [u8],
    hint: Option<&'s memmem::Finder<'static>>,
    dense_start: Option<usize>,
    dense_hits: usize,
    recent: PrefixCache,
}

impl Iterator for CandidateScanner<'_, '_> {
    type Item = Candidate;
    fn next(&mut self) -> Option<Self::Item> {
        let Some(width) = self.width else {
            return self.overlapping.as_mut()?.next().map(Candidate::from);
        };
        if let Some(overlapping) = &mut self.overlapping {
            let hit = Candidate::from(overlapping.next()?);
            let gap = hit.start().saturating_sub(self.from);
            self.from = hit.start() + 1;
            if gap > 256 {
                self.overlapping = None;
                self.dense_hits = 0;
                self.dense_start = None;
            }
            return Some(hit);
        }
        loop {
            let remaining = self.text.as_bytes().get(self.from..)?;
            if remaining.len() < width {
                return None;
            }
            let relative = if let Some(hint) = self.hint {
                hint.find(remaining)
            } else {
                match self.first_bytes {
                    [a] => memchr::memchr(*a, remaining),
                    [a, b] => memchr::memchr2(*a, *b, remaining),
                    [a, b, c] => memchr::memchr3(*a, *b, *c, remaining),
                    _ => None,
                }
            }?;
            let start = self.from + relative;
            self.from = start + 1;
            if start + width > self.text.len() {
                return None;
            }
            let hit = if let Some(fixed) = self.fixed_index {
                let key = fixed_key(&self.text.as_bytes()[start..start + width]);
                let prefix_id = self.recent.get(fixed, key);
                prefix_id.map(|prefix_id| Candidate {
                    prefix_id,
                    start,
                    end: start + width,
                })
            } else {
                // A width-sized window cannot contain a later same-width hit.
                self.index?
                    .find(Input::new(self.text).span(start..start + width))
                    .map(Candidate::from)
            };
            if let Some(hit) = hit {
                if self.fixed_index.is_some() {
                    return Some(hit);
                }
                let first = *self.dense_start.get_or_insert(start);
                self.dense_hits += 1;
                if self.dense_hits == 4 {
                    if start - first <= width * 6 {
                        self.overlapping = Some(self.index?.find_overlapping_iter(
                            Input::new(self.text).span(self.from..self.text.len()),
                        ));
                    }
                    self.dense_start = None;
                    self.dense_hits = 0;
                }
                return Some(hit);
            }
        }
    }
}

struct Cursor<'s, 'h> {
    set: &'s PatternSet,
    text: &'h str,
    scanner: Option<CandidateScanner<'s, 'h>>,
    pending: Option<Candidate>,
    mode: SetSearchMode,
    started: bool,
    refill: Option<usize>,
    owner_offset: usize,
    exhausted: bool,
    capture_groups: bool,
}
impl<'s, 'h> Cursor<'s, 'h> {
    fn new(set: &'s PatternSet, text: &'h str, cache: &mut SetCache, mode: SetSearchMode) -> Self {
        cache.heap.clear();
        let exhausted = set.reject(text);
        let mut scanner = if exhausted {
            None
        } else {
            Some(set.scanner(text))
        };
        let pending = scanner.as_mut().and_then(Iterator::next);
        let exhausted = exhausted || (set.inner.fully_filtered && pending.is_none());
        if !exhausted {
            cache.reset(set, text);
            if set.inner.has_anchors {
                for event in set.scanner(text) {
                    for &id in &set.inner.owners[event.prefix_id] {
                        if set.inner.rules[id].anchor.is_some() {
                            cache.eligible.insert(id);
                        }
                    }
                }
            }
        }

        Self {
            set,
            text,
            scanner,
            pending,
            mode,
            started: false,
            refill: None,
            owner_offset: 0,
            exhausted,
            capture_groups: false,
        }
    }
    fn next(&mut self, cache: &mut SetCache) -> Option<SetMatch<'h>> {
        if self.exhausted {
            return None;
        }
        if self.set.inner.uniform_prefixes {
            return self.next_uniform(cache);
        }

        if !self.started {
            self.started = true;
            for &id in &self.set.inner.fallback_ids {
                if let Some(start) = self.set.next_fallback(self.text, id, cache) {
                    cache.heap.push(Reverse((start, id)));
                }
            }
            // Only anchored rules encountered by the shared index can match.
            // Iterate set bits instead of walking every compiled rule.
            for block in 0..=cache.eligible.words.len() {
                let mut ids = if block == 0 {
                    cache.eligible.first
                } else {
                    cache.eligible.words[block - 1]
                };
                while ids != 0 {
                    let id = block * 64 + ids.trailing_zeros() as usize;
                    ids &= ids - 1;
                    if let Some(start) = self.set.next_fallback(self.text, id, cache) {
                        cache.heap.push(Reverse((start, id)));
                    }
                }
            }
        }
        if let Some(id) = self.refill.take() {
            let start = self.set.next_fallback(self.text, id, cache);
            if let Some(start) = start {
                cache.heap.push(Reverse((start, id)));
            }
        }
        loop {
            // An unseen literal ending at E cannot start before E-max_prefix_len.
            // Consume all possible earlier/tied candidates before emitting.
            while self.pending.is_some_and(|event| {
                cache.heap.peek().map_or(true, |Reverse((start, _))| {
                    event.end().saturating_sub(self.set.inner.max_prefix_len) <= *start
                })
            }) {
                let event = self.pending.take()?;
                for &id in &self.set.inner.owners[event.prefix_id] {
                    if self.set.inner.rules[id].indexed
                        && !cache.progress[id].finished
                        && event.start() >= cache.progress[id].pos
                    {
                        cache.heap.push(Reverse((event.start(), id)));
                    }
                }
                self.pending = self.scanner.as_mut().and_then(Iterator::next);
            }
            let Reverse((start, id)) = cache.heap.pop()?;
            if cache.progress[id].finished
                || start < cache.progress[id].pos
                || (self.set.inner.deduplicate_candidates && cache.verified[id] == start)
            {
                continue;
            }
            if self.set.inner.deduplicate_candidates {
                cache.verified[id] = start;
            }
            let Some(end) = self
                .set
                .exact(self.text, id, start, cache, self.capture_groups)
            else {
                continue;
            };
            if !cache.progress[id].accept(self.text, start, end) {
                continue;
            }
            if self.mode == SetSearchMode::FirstPerPattern {
                cache.progress[id].finished = true;
            }
            if !self.set.inner.rules[id].indexed {
                self.refill = Some(id);
            }
            return Some(SetMatch {
                pattern_id: id,
                matched: Match::new(self.text, start, end),
            });
        }
    }
    fn next_uniform(&mut self, cache: &mut SetCache) -> Option<SetMatch<'h>> {
        loop {
            let event = self.pending?;
            let owners = &self.set.inner.owners[event.prefix_id];
            if self.owner_offset == owners.len() {
                self.pending = self.scanner.as_mut().and_then(Iterator::next);
                self.owner_offset = 0;
                continue;
            }
            let id = owners[self.owner_offset];
            self.owner_offset += 1;
            let start = event.start();
            if cache.progress[id].finished || start < cache.progress[id].pos {
                continue;
            }
            let Some(end) = self
                .set
                .exact(self.text, id, start, cache, self.capture_groups)
            else {
                continue;
            };
            if !cache.progress[id].accept(self.text, start, end) {
                continue;
            }
            if self.mode == SetSearchMode::FirstPerPattern {
                cache.progress[id].finished = true;
            }
            return Some(SetMatch {
                pattern_id: id,
                matched: Match::new(self.text, start, end),
            });
        }
    }
}

/// Lazy iterator of matches across all rules.
pub struct SetFindIter<'s, 'h> {
    cursor: Cursor<'s, 'h>,
    cache: SetCache,
}
impl<'h> Iterator for SetFindIter<'_, 'h> {
    type Item = SetMatch<'h>;
    fn next(&mut self) -> Option<Self::Item> {
        self.cursor.next(&mut self.cache)
    }
}
impl std::iter::FusedIterator for SetFindIter<'_, '_> {}

/// Lazy iterator of captures across all rules.
pub struct SetCapturesIter<'s, 'h> {
    matches: SetFindIter<'s, 'h>,
}
impl<'h> Iterator for SetCapturesIter<'_, 'h> {
    type Item = SetCaptures<'h>;
    fn next(&mut self) -> Option<Self::Item> {
        let hit = self.matches.next()?;
        Some(
            SetCapturesRef {
                pattern_id: hit.pattern_id,
                text: self.matches.cursor.text,
                positions: self.matches.cache.positions(),
            }
            .to_owned(),
        )
    }
}
impl std::iter::FusedIterator for SetCapturesIter<'_, '_> {}
