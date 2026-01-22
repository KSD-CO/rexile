//! ReXile - Fast regex-lite engine built on memchr and aho-corasick
//!
//! **Zero dependency on the regex crate!**

mod charclass;
mod quantifier;
mod escape;
mod sequence;
mod sequence_parser;
mod group;
mod boundary;  // Phase 6: Word boundary support
mod lookaround;  // Phase 7: Lookahead/lookbehind
mod captures;  // Phase 8: Capture groups

use aho_corasick::AhoCorasick;
use charclass::CharClass;
use escape::{parse_escape, starts_with_escape};
use memchr::memmem;
use sequence::Sequence;
use sequence_parser::{is_sequence_pattern, parse_sequence};
use group::Group;
use quantifier::{parse_quantified_pattern, QuantifiedPattern};
use boundary::BoundaryType;
use lookaround::{Lookaround, LookaroundType};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

// Re-export capture types for public API
pub use captures::{Captures, Group as CaptureGroup};

/// Main ReXile pattern type
#[derive(Debug, Clone)]
pub struct Pattern {
    matcher: Matcher,
}

/// Type alias for convenience
pub type ReXile = Pattern;

impl Pattern {
    pub fn new(pattern: &str) -> Result<Self, PatternError> {
        let ast = parse_pattern(pattern)?;
        let matcher = compile_ast(&ast)?;
        Ok(Pattern { matcher })
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.matcher.is_match(text)
    }

    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        self.matcher.find(text)
    }

    pub fn find_all(&self, text: &str) -> Vec<(usize, usize)> {
        // OPTIMIZED: Fast path for Literal using memchr's find_iter
        match &self.matcher {
            Matcher::Literal(lit) => {
                // Use memmem::find_iter for direct SIMD iteration
                memmem::find_iter(text.as_bytes(), lit.as_bytes())
                    .map(|pos| (pos, pos + lit.len()))
                    .collect()
            }
            Matcher::MultiLiteral(ac) => {
                // AhoCorasick already has find_iter
                ac.find_iter(text)
                    .map(|mat| (mat.start(), mat.end()))
                    .collect()
            }
            _ => {
                // Complex patterns: use general iterator
                self.find_iter(text).collect()
            }
        }
    }
    
    /// Create an iterator over all matches (zero-allocation)
    pub fn find_iter<'a>(&'a self, text: &'a str) -> FindIter<'a> {
        FindIter {
            matcher: &self.matcher,
            text,
            pos: 0,
        }
    }
    
    /// Capture groups from the first match
    ///
    /// Returns a `Captures` object if the pattern matches, containing the full match
    /// and any captured groups. Returns None if no match is found.
    ///
    /// # Example
    /// ```
    /// use rexile::Pattern;
    ///
    /// let pattern = Pattern::new(r"(\w+)@(\w+)\.(\w+)").unwrap();
    /// if let Some(caps) = pattern.captures("email: test@example.com") {
    ///     println!("Full: {}", &caps[0]);    // test@example.com
    ///     println!("User: {}", &caps[1]);    // test
    ///     println!("Domain: {}", &caps[2]);  // example
    ///     println!("TLD: {}", &caps[3]);     // com
    /// }
    /// ```
    pub fn captures<'t>(&self, text: &'t str) -> Option<Captures<'t>> {
        // Check if this is a PatternWithCaptures matcher
        if let Matcher::PatternWithCaptures { elements, total_groups } = &self.matcher {
            // Find first match and extract capture positions
            for start_pos in 0..text.len() {
                let mut pos = start_pos;
                let mut capture_positions: Vec<(usize, usize)> = Vec::new();
                let mut all_matched = true;
                
                for element in elements {
                    let (matcher, group_num_opt) = match element {
                        CompiledCaptureElement::Capture(m, num) => (m, Some(*num)),
                        CompiledCaptureElement::NonCapture(m) => (m, None),
                    };
                    
                    if let Some((rel_start, rel_end)) = matcher.find(&text[pos..]) {
                        if rel_start != 0 {
                            // Element must match at current position
                            all_matched = false;
                            break;
                        }
                        
                        let abs_start = pos;
                        let abs_end = pos + rel_end;
                        
                        // If this is a capture group, record its position
                        if let Some(group_num) = group_num_opt {
                            // Ensure we have enough space
                            while capture_positions.len() < group_num {
                                capture_positions.push((0, 0));
                            }
                            capture_positions[group_num - 1] = (abs_start, abs_end);
                        }
                        
                        pos = abs_end;
                    } else {
                        all_matched = false;
                        break;
                    }
                }
                
                if all_matched {
                    // Create Captures with full match and capture groups
                    let mut caps = Captures::new(text, (start_pos, pos), *total_groups);
                    
                    // Add each capture group using the set method
                    for (i, &(start, end)) in capture_positions.iter().enumerate() {
                        caps.set(i + 1, start, end);
                    }
                    
                    return Some(caps);
                }
            }
            None
        } else {
            // Simple pattern without explicit captures - just return full match
            self.find(text).map(|(start, end)| {
                Captures::new(text, (start, end), 0)
            })
        }
    }
    
    /// Iterate over all captures in the text
    ///
    /// Returns an iterator that yields `Captures` for each match found.
    ///
    /// # Example
    /// ```
    /// use rexile::Pattern;
    ///
    /// let pattern = Pattern::new(r"(\w+)=(\d+)").unwrap();
    /// for caps in pattern.captures_iter("a=1 b=2 c=3") {
    ///     println!("{} = {}", &caps[1], &caps[2]);
    /// }
    /// ```
    pub fn captures_iter<'r, 't>(&'r self, text: &'t str) -> CapturesIter<'r, 't> {
        CapturesIter {
            pattern: self,
            text,
            pos: 0,
        }
    }
    
    /// Replace all matches with a replacement string
    ///
    /// Supports capture group references using $1, $2, etc.
    ///
    /// # Example
    /// ```
    /// use rexile::Pattern;
    ///
    /// let pattern = Pattern::new(r"(\w+)=(\d+)").unwrap();
    /// let result = pattern.replace_all("a=1 b=2", "$1:[$2]");
    /// assert_eq!(result, "a:[1] b:[2]");
    /// ```
    pub fn replace_all(&self, text: &str, replacement: &str) -> String {
        // TODO: Implement with capture group replacement
        // For now, simple literal replacement
        let mut result = String::new();
        let mut last_end = 0;
        
        for (start, end) in self.find_all(text) {
            result.push_str(&text[last_end..start]);
            result.push_str(replacement);
            last_end = end;
        }
        result.push_str(&text[last_end..]);
        result
    }
    
    /// Split text by matches of this pattern
    ///
    /// # Example
    /// ```
    /// use rexile::Pattern;
    ///
    /// let pattern = Pattern::new(r"\s+").unwrap();
    /// let parts: Vec<_> = pattern.split("a  b   c").collect();
    /// assert_eq!(parts, vec!["a", "b", "c"]);
    /// ```
    pub fn split<'r, 't>(&'r self, text: &'t str) -> SplitIter<'r, 't> {
        SplitIter {
            pattern: self,
            text,
            pos: 0,
            finished: false,
        }
    }
}

/// Iterator over pattern matches (zero-allocation)
pub struct FindIter<'a> {
    matcher: &'a Matcher,
    text: &'a str,
    pos: usize,
}

impl<'a> Iterator for FindIter<'a> {
    type Item = (usize, usize);
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.text.len() {
            return None;
        }
        
        // Find next match starting from current position
        let remaining = &self.text[self.pos..];
        if let Some((rel_start, rel_end)) = self.matcher.find(remaining) {
            let abs_start = self.pos + rel_start;
            let abs_end = self.pos + rel_end;
            
            // Move position past this match to avoid infinite loop
            self.pos = abs_end.max(self.pos + 1);
            
            Some((abs_start, abs_end))
        } else {
            None
        }
    }
}

/// Iterator over captures for each match
pub struct CapturesIter<'r, 't> {
    pattern: &'r Pattern,
    text: &'t str,
    pos: usize,
}

impl<'r, 't> Iterator for CapturesIter<'r, 't> {
    type Item = Captures<'t>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.text.len() {
            return None;
        }
        
        // Check if this is a PatternWithCaptures matcher
        if let Matcher::PatternWithCaptures { elements, total_groups } = &self.pattern.matcher {
            // Find next match starting from current position and extract capture positions
            let remaining = &self.text[self.pos..];
            for start_offset in 0..remaining.len() {
                let mut pos = start_offset;
                let mut capture_positions: Vec<(usize, usize)> = Vec::new();
                let mut all_matched = true;
                
                for element in elements {
                    let (matcher, group_num_opt) = match element {
                        CompiledCaptureElement::Capture(m, num) => (m, Some(*num)),
                        CompiledCaptureElement::NonCapture(m) => (m, None),
                    };
                    
                    if let Some((rel_start, rel_end)) = matcher.find(&remaining[pos..]) {
                        if rel_start != 0 {
                            // Element must match at current position
                            all_matched = false;
                            break;
                        }
                        
                        let abs_start = pos;
                        let abs_end = pos + rel_end;
                        
                        // If this is a capture group, record its position
                        if let Some(group_num) = group_num_opt {
                            // Ensure we have enough space
                            while capture_positions.len() < group_num {
                                capture_positions.push((0, 0));
                            }
                            capture_positions[group_num - 1] = (abs_start, abs_end);
                        }
                        
                        pos = abs_end;
                    } else {
                        all_matched = false;
                        break;
                    }
                }
                
                if all_matched {
                    // Convert relative positions to absolute positions
                    let abs_start = self.pos + start_offset;
                    let abs_end = self.pos + pos;
                    
                    // Move position past this match
                    self.pos = abs_end.max(self.pos + 1);
                    
                    // Create Captures with full match and capture groups
                    let mut caps = Captures::new(self.text, (abs_start, abs_end), *total_groups);
                    
                    // Add each capture group using the set method
                    for (i, &(start, end)) in capture_positions.iter().enumerate() {
                        caps.set(i + 1, self.pos - pos + start, self.pos - pos + end);
                    }
                    
                    return Some(caps);
                }
            }
            None
        } else {
            // Simple pattern without explicit captures
            let remaining = &self.text[self.pos..];
            if let Some((rel_start, rel_end)) = self.pattern.matcher.find(remaining) {
                let abs_start = self.pos + rel_start;
                let abs_end = self.pos + rel_end;
                
                // Move position past this match
                self.pos = abs_end.max(self.pos + 1);
                
                // Create captures for this match
                Some(Captures::new(self.text, (abs_start, abs_end), 0))
            } else {
                None
            }
        }
    }
}

/// Iterator over text split by pattern matches
pub struct SplitIter<'r, 't> {
    pattern: &'r Pattern,
    text: &'t str,
    pos: usize,
    finished: bool,
}

impl<'r, 't> Iterator for SplitIter<'r, 't> {
    type Item = &'t str;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        
        // Find next match starting from current position
        let remaining = &self.text[self.pos..];
        if let Some((rel_start, rel_end)) = self.pattern.matcher.find(remaining) {
            let abs_start = self.pos + rel_start;
            let abs_end = self.pos + rel_end;
            
            // Return text before the match
            let result = &self.text[self.pos..abs_start];
            self.pos = abs_end;
            
            Some(result)
        } else {
            // No more matches, return remaining text
            self.finished = true;
            Some(&self.text[self.pos..])
        }
    }
}

static CACHE: OnceLock<Mutex<HashMap<String, Pattern>>> = OnceLock::new();

fn get_cache() -> &'static Mutex<HashMap<String, Pattern>> {
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn get_pattern(pattern: &str) -> Result<Pattern, PatternError> {
    let mut cache = get_cache().lock().unwrap();
    if let Some(p) = cache.get(pattern) {
        return Ok(p.clone());
    }
    let compiled = Pattern::new(pattern)?;
    cache.insert(pattern.to_string(), compiled.clone());
    Ok(compiled)
}

pub fn is_match(pattern: &str, text: &str) -> Result<bool, PatternError> {
    Ok(get_pattern(pattern)?.is_match(text))
}

pub fn find(pattern: &str, text: &str) -> Result<Option<(usize, usize)>, PatternError> {
    Ok(get_pattern(pattern)?.find(text))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternError {
    ParseError(String),
    UnsupportedFeature(String),
}

impl std::fmt::Display for PatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            PatternError::UnsupportedFeature(msg) => write!(f, "Unsupported: {}", msg),
        }
    }
}

impl std::error::Error for PatternError {}

#[derive(Debug, Clone, PartialEq)]
enum Ast {
    Literal(String),
    Alternation(Vec<String>),
    Anchored { literal: String, start: bool, end: bool },
    AnchoredGroup { group: Group, start: bool, end: bool },
    CharClass(CharClass),
    Quantified(QuantifiedPattern),
    Sequence(Sequence),
    Group(Group),
    Boundary(BoundaryType),  // Phase 6: Word boundary support
    Lookaround(Lookaround),  // Phase 7: Lookahead/lookbehind
    Capture(Box<Ast>, usize),  // Phase 8: Capture group (pattern, group_index)
    CombinedWithLookaround { prefix: Box<Ast>, lookaround: Lookaround },  // Phase 7.2: foo(?=bar)
    PatternWithCaptures { elements: Vec<CaptureElement>, total_groups: usize },  // Phase 8.1: Hello (\w+)
}

/// Parse patterns that contain groups combined with other elements
/// Handles: ^(hello), (foo)(bar), prefix(foo|bar), (foo|bar)suffix, (http|https)://
fn parse_pattern_with_groups(pattern: &str) -> Result<Ast, PatternError> {
    // Case 1: Multiple consecutive groups: (foo)(bar) - CHECK FIRST!
    if pattern.matches('(').count() > 1 && !pattern.contains('|') {
        let mut combined_literals = Vec::new();
        let mut pos = 0;
        let mut all_parsed = true;
        
        while pos < pattern.len() && pattern[pos..].starts_with('(') {
            match group::parse_group(&pattern[pos..]) {
                Ok((group, bytes_consumed)) => {
                    // Extract literals from this group
                    match &group.content {
                        group::GroupContent::Single(s) => {
                            combined_literals.push(s.clone());
                        }
                        group::GroupContent::Sequence(seq) => {
                            // Try to extract literal from sequence of chars
                            let mut literal = String::new();
                            let mut is_simple = true;
                            
                            for elem in &seq.elements {
                                match elem {
                                    crate::sequence::SequenceElement::Char(ch) => {
                                        literal.push(*ch);
                                    }
                                    crate::sequence::SequenceElement::Literal(lit) => {
                                        literal.push_str(lit);
                                    }
                                    _ => {
                                        // Not a simple literal sequence
                                        is_simple = false;
                                        break;
                                    }
                                }
                            }
                            
                            if is_simple {
                                combined_literals.push(literal);
                            } else {
                                all_parsed = false;
                                break;
                            }
                        }
                        group::GroupContent::Alternation(_parts) => {
                            // Can't easily combine alternations
                            all_parsed = false;
                            break;
                        }
                    }
                    pos += bytes_consumed;
                }
                Err(_) => {
                    all_parsed = false;
                    break;
                }
            }
        }
        
        if all_parsed && pos == pattern.len() && !combined_literals.is_empty() {
            // All groups parsed successfully - build as sequence
            // Create a sequence of literal elements for consecutive matching
            use crate::sequence::{Sequence, SequenceElement};
            
            let mut elements = Vec::new();
            for literal in combined_literals {
                // Each literal becomes a sequence element
                elements.push(SequenceElement::Literal(literal));
            }
            
            let seq = Sequence { elements };
            return Ok(Ast::Sequence(seq));
        }
    }
    
    // Case 2: Anchor + Group: ^(hello) or (world)$
    if pattern.starts_with("^(") || pattern.ends_with(")$") {
        let has_start = pattern.starts_with('^');
        let has_end = pattern.ends_with('$');
        
        // Strip anchors properly - need to handle chaining correctly
        let mut inner = pattern;
        if has_start {
            inner = &inner[1..];  // Remove '^'
        }
        if has_end {
            inner = &inner[..inner.len()-1];  // Remove '$'
        }
        
        if inner.starts_with('(') {
            if let Ok((group, bytes_consumed)) = group::parse_group(inner) {
                if bytes_consumed == inner.len() {
                    // Extract the actual pattern from group for anchored matching
                    let group_literal = match &group.content {
                        group::GroupContent::Single(s) => Some(s.clone()),
                        group::GroupContent::Sequence(seq) => {
                            // Try to extract literal from sequence of chars
                            let mut literal = String::new();
                            let mut is_simple = true;
                            
                            for elem in &seq.elements {
                                match elem {
                                    crate::sequence::SequenceElement::Char(ch) => {
                                        literal.push(*ch);
                                    }
                                    crate::sequence::SequenceElement::Literal(lit) => {
                                        literal.push_str(lit);
                                    }
                                    _ => {
                                        // Not a simple literal - can't anchor
                                        is_simple = false;
                                        break;
                                    }
                                }
                            }
                            
                            if is_simple {
                                Some(literal)
                            } else {
                                None
                            }
                        }
                        group::GroupContent::Alternation(_parts) => {
                            // For alternation like ^(foo|bar), can't use simple Anchored
                            None
                        }
                    };
                    
                    if let Some(lit) = group_literal {
                        return Ok(Ast::Anchored {
                            literal: lit,
                            start: has_start,
                            end: has_end,
                        });
                    } else {
                        // Complex group - use AnchoredGroup
                        return Ok(Ast::AnchoredGroup {
                            group,
                            start: has_start,
                            end: has_end,
                        });
                    }
                }
            }
        }
    }
    
    // Case 3: Just a single group
    if pattern.starts_with('(') {
        if let Ok((group, bytes_consumed)) = group::parse_group(pattern) {
            if bytes_consumed == pattern.len() {
                return Ok(Ast::Group(group));
            }
            
            // Case 4: Group with suffix: (foo|bar)suffix, (http|https)://
            if bytes_consumed < pattern.len() {
                let suffix = &pattern[bytes_consumed..];
                // Build a combined pattern
                // For alternation groups, expand: (a|b)c -> ac|bc
                match &group.content {
                    group::GroupContent::Alternation(parts) => {
                        let expanded: Vec<String> = parts.iter()
                            .map(|p| format!("{}{}", p, suffix))
                            .collect();
                        return Ok(Ast::Alternation(expanded));
                    }
                    group::GroupContent::Sequence(seq) => {
                        // Group with sequence + suffix: (\w+)@ or (\d+).
                        // Need to append suffix to the sequence
                        use crate::sequence::{Sequence, SequenceElement};
                        
                        let mut new_elements = seq.elements.clone();
                        // Add suffix as literal elements
                        for ch in suffix.chars() {
                            new_elements.push(SequenceElement::Char(ch));
                        }
                        
                        let combined_seq = Sequence { elements: new_elements };
                        return Ok(Ast::Sequence(combined_seq));
                    }
                    group::GroupContent::Single(s) => {
                        // Simple literal + suffix
                        let combined = format!("{}{}", s, suffix);
                        return Ok(Ast::Literal(combined));
                    }
                }
            }
        }
    }
    
    // Case 5: Prefix + Group: prefix(foo|bar) - but NOT ^(hello) or $(hello)
    if let Some(group_start) = pattern.find('(') {
        if group_start > 0 {
            let prefix = &pattern[..group_start];
            // Skip if prefix is just an anchor
            if prefix != "^" && prefix != "$" {
                let group_part = &pattern[group_start..];
                
                if let Ok((group, bytes_consumed)) = group::parse_group(group_part) {
                    if bytes_consumed == group_part.len() {
                        // prefix + group
                        match &group.content {
                            group::GroupContent::Alternation(parts) => {
                                let expanded: Vec<String> = parts.iter()
                                    .map(|p| format!("{}{}", prefix, p))
                                    .collect();
                                return Ok(Ast::Alternation(expanded));
                            }
                            _ => {
                                // Single pattern with prefix
                                return Ok(Ast::Group(group));
                            }
                        }
                    }
                }
            }
        }
    }
    
    Err(PatternError::ParseError(
        "Complex group pattern not fully supported".to_string()
    ))
}

fn parse_pattern(pattern: &str) -> Result<Ast, PatternError> {
    if pattern.is_empty() {
        return Ok(Ast::Literal(String::new()));
    }
    
    // Phase 7: Check for lookaround assertions (?=...), (?!...), (?<=...), (?<!...)
    if pattern.starts_with("(?=") || pattern.starts_with("(?!") || 
       pattern.starts_with("(?<=") || pattern.starts_with("(?<!") {
        return parse_lookaround(pattern);
    }
    
    // Phase 7.2: Check for combined patterns with lookaround: foo(?=bar), \d+(?!x)
    if pattern.contains("(?=") || pattern.contains("(?!") || 
       pattern.contains("(?<=") || pattern.contains("(?<!") {
        // Try to parse as combined pattern with lookaround
        if let Ok(ast) = parse_combined_with_lookaround(pattern) {
            return Ok(ast);
        }
    }
    
    // Phase 8: Check for capture groups (...) - but not (?:...) which is handled by group parser
    // Simple heuristic: if starts with ( but not (? or (?:, might be capture group
    if pattern.starts_with('(') && !pattern.starts_with("(?") {
        // Check if this is a simple capture group pattern
        if let Some(close_idx) = find_matching_paren(pattern, 0) {
            if close_idx == pattern.len() - 1 {
                // Entire pattern is a capture group: (pattern)
                let inner = &pattern[1..close_idx];
                let inner_ast = parse_pattern(inner)?;
                return Ok(Ast::Capture(Box::new(inner_ast), 1)); // Group 1
            }
        }
    }
    
    // Phase 8.1: Check for patterns with embedded captures: Hello (\w+), (\w+)=(\d+)
    // Phase 8.2: Also handles non-capturing groups: (?:Hello) (\w+)
    if pattern.contains('(') && !pattern.contains("(?=") 
       && !pattern.contains("(?!") && !pattern.contains("(?<=") && !pattern.contains("(?<!") {
        // Try to parse as pattern with captures (including non-capturing groups)
        if let Ok(ast) = parse_pattern_with_captures(pattern) {
            return Ok(ast);
        }
    }
    
    // Special handling for patterns with groups and other elements
    // e.g., ^(hello), (foo)(bar), prefix(foo|bar), (foo|bar)suffix
    if pattern.contains('(') {
        // Try to parse as complex pattern with groups
        if let Ok(ast) = parse_pattern_with_groups(pattern) {
            return Ok(ast);
        }
    }
    
    // Check for anchors (before sequences)
    let has_start_anchor = pattern.starts_with('^');
    let has_end_anchor = pattern.ends_with('$');
    
    if has_start_anchor || has_end_anchor {
        let literal = pattern
            .strip_prefix('^').unwrap_or(pattern)
            .strip_suffix('$').unwrap_or(pattern);
        
        // Don't treat anchored patterns as sequences
        return Ok(Ast::Anchored {
            literal: literal.to_string(),
            start: has_start_anchor,
            end: has_end_anchor,
        });
    }
    
    // Check for alternation (|)
    if pattern.contains('|') && !pattern.contains('[') {
        let parts: Vec<String> = pattern.split('|').map(|s| s.to_string()).collect();
        return Ok(Ast::Alternation(parts));
    }
    
    // Check for sequence pattern (most complex)
    if is_sequence_pattern(pattern) {
        match parse_sequence(pattern) {
            Ok(seq) => return Ok(Ast::Sequence(seq)),
            Err(_) => {
                // Fall through to other parsers
            }
        }
    }
    
    // Check for escape sequences: \d, \w, \s, \b, \B, \., etc.
    if starts_with_escape(pattern) {
        match parse_escape(pattern) {
            Ok((seq, bytes_consumed)) => {
                // If it's the whole pattern
                if bytes_consumed == pattern.len() {
                    // Check for boundary first (since it doesn't convert to CharClass)
                    if let Some(boundary_type) = seq.to_boundary() {
                        return Ok(Ast::Boundary(boundary_type));
                    }
                    // Convert to CharClass if possible
                    if let Some(cc) = seq.to_char_class() {
                        return Ok(Ast::CharClass(cc));
                    }
                    // Or to literal char
                    if let Some(ch) = seq.to_char() {
                        return Ok(Ast::Literal(ch.to_string()));
                    }
                }
                // Otherwise, check for quantifier after escape
                let remaining = &pattern[bytes_consumed..];
                if !remaining.is_empty() {
                    if let Some(q_char) = remaining.chars().next() {
                        if q_char == '*' || q_char == '+' || q_char == '?' || q_char == '{' {
                            // This is an escape with quantifier: \d+, \w*, \d{4}, etc.
                            if let Ok(qp) = parse_quantified_pattern(pattern) {
                                return Ok(Ast::Quantified(qp));
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(PatternError::ParseError(e)),
        }
    }
    
    // Check for quantified patterns: a+, [0-9]*, \d+, etc.
    let has_quantifier = pattern.ends_with('*') || 
                        pattern.ends_with('+') || 
                        pattern.ends_with('?') ||
                        (pattern.contains('{') && pattern.ends_with('}'));
    
    if has_quantifier {
        // Try to parse as quantified pattern
        match parse_quantified_pattern(pattern) {
            Ok(qp) => return Ok(Ast::Quantified(qp)),
            Err(_) => {
                // Fall through to other parsers
            }
        }
    }
    
    // Check for character class [...]
    if pattern.starts_with('[') && pattern.contains(']') {
        let end_idx = pattern.find(']').unwrap();
        if end_idx == pattern.len() - 1 {
            // Pure character class pattern: [a-z]
            let class_content = &pattern[1..end_idx];
            let char_class = CharClass::parse(class_content)
                .map_err(|e| PatternError::ParseError(e))?;
            return Ok(Ast::CharClass(char_class));
        }
        // Character class with quantifier is handled above
    }
    
    // Default: treat as literal
    Ok(Ast::Literal(pattern.to_string()))
}

#[derive(Debug, Clone)]
enum Matcher {
    Literal(String),
    MultiLiteral(AhoCorasick),
    AnchoredLiteral { literal: String, start: bool, end: bool },
    AnchoredGroup { group: Group, start: bool, end: bool },
    CharClass(CharClass),
    Quantified(QuantifiedPattern),
    Sequence(Sequence),
    Group(Group),
    DigitRun,  // Specialized fast path for \d+ pattern
    WordRun,   // Specialized fast path for \w+ pattern
    Boundary(BoundaryType),  // Phase 6: Word boundary matcher
    Lookaround(Box<Lookaround>, Box<Matcher>),  // Phase 7: Lookaround with compiled inner matcher
    Capture(Box<Matcher>, usize),  // Phase 8: Capture matcher (inner pattern, group_index)
    CombinedWithLookaround { prefix: Box<Matcher>, lookaround: Box<Lookaround>, lookaround_matcher: Box<Matcher> },  // Phase 7.2
    PatternWithCaptures { elements: Vec<CompiledCaptureElement>, total_groups: usize },  // Phase 8.1
}

/// Compiled capture element
#[derive(Debug, Clone)]
enum CompiledCaptureElement {
    Capture(Matcher, usize),     // Compiled matcher, group number
    NonCapture(Matcher),         // Compiled matcher (non-capturing)
}

impl Matcher {
    fn is_match(&self, text: &str) -> bool {
        match self {
            Matcher::Literal(lit) => memmem::find(text.as_bytes(), lit.as_bytes()).is_some(),
            Matcher::MultiLiteral(ac) => ac.is_match(text),
            Matcher::AnchoredLiteral { literal, start, end } => match (start, end) {
                (true, true) => text == literal,
                (true, false) => text.starts_with(literal),
                (false, true) => text.ends_with(literal),
                _ => unreachable!(),
            },
            Matcher::AnchoredGroup { group, start, end } => {
                // Check if group matches with anchor constraints
                match (start, end) {
                    (true, true) => {
                        // Must match entire text
                        group.match_at(text, 0).map(|len| len == text.len()).unwrap_or(false)
                    }
                    (true, false) => {
                        // Must match at start
                        group.match_at(text, 0).is_some()
                    }
                    (false, true) => {
                        // Must match at end
                        if let Some((start_pos, end_pos)) = group.find(text) {
                            end_pos == text.len()
                        } else {
                            false
                        }
                    }
                    _ => unreachable!(),
                }
            },
            Matcher::CharClass(cc) => {
                // OPTIMIZED: Use SIMD-friendly find_first for ASCII text
                cc.find_first(text).is_some()
            }
            Matcher::Quantified(qp) => qp.is_match(text),  // NEW: Early termination
            Matcher::Sequence(seq) => seq.is_match(text),  // NEW: Early termination
            Matcher::Group(group) => group.is_match(text), // NEW: Early termination
            Matcher::DigitRun => Self::digit_run_is_match(text),  // NEW: Specialized digit fast path
            Matcher::WordRun => Self::word_run_is_match(text),    // NEW: Specialized word fast path
            Matcher::Boundary(boundary_type) => boundary_type.find_first(text).is_some(),
            Matcher::Lookaround(lookaround, inner_matcher) => {
                // Lookaround assertions are zero-width, check if they match at any position
                for pos in 0..=text.len() {
                    if lookaround.matches_at(text, pos, inner_matcher) {
                        return true;
                    }
                }
                false
            }
            Matcher::Capture(inner_matcher, _group_index) => {
                // Capture groups don't affect matching, just check inner pattern
                inner_matcher.is_match(text)
            }
            Matcher::CombinedWithLookaround { prefix, lookaround, lookaround_matcher } => {
                // Need to find where prefix matches, then check lookaround at that position
                if let Some((start, end)) = prefix.find(text) {
                    // Check if lookaround succeeds at the end position of the prefix match
                    lookaround.matches_at(text, end, lookaround_matcher)
                } else {
                    false
                }
            }
            Matcher::PatternWithCaptures { elements, .. } => {
                // Check if all elements match in sequence
                let mut pos = 0;
                for element in elements {
                    let matcher = match element {
                        CompiledCaptureElement::Capture(m, _) => m,
                        CompiledCaptureElement::NonCapture(m) => m,
                    };
                    if let Some((_, end)) = matcher.find(&text[pos..]) {
                        pos += end;
                    } else {
                        return false;
                    }
                }
                true
            }
        }
    }
    
    /// Specialized fast path for \d+ pattern
    #[inline(always)]
    fn digit_run_is_match(text: &str) -> bool {
        let bytes = text.as_bytes();
        if bytes.is_empty() {
            return false;
        }
        
        // Check if text starts with at least one digit
        bytes.iter().any(|&b| b >= b'0' && b <= b'9')
    }
    
    /// Specialized fast path for \w+ pattern  
    #[inline(always)]
    fn word_run_is_match(text: &str) -> bool {
        let bytes = text.as_bytes();
        if bytes.is_empty() {
            return false;
        }
        
        // Check if text contains at least one word char [a-zA-Z0-9_]
        bytes.iter().any(|&b| {
            (b >= b'a' && b <= b'z') || 
            (b >= b'A' && b <= b'Z') || 
            (b >= b'0' && b <= b'9') || 
            b == b'_'
        })
    }

    fn find(&self, text: &str) -> Option<(usize, usize)> {
        match self {
            Matcher::Literal(lit) => {
                let pos = memmem::find(text.as_bytes(), lit.as_bytes())?;
                Some((pos, pos + lit.len()))
            }
            Matcher::MultiLiteral(ac) => {
                let mat = ac.find(text)?;
                Some((mat.start(), mat.end()))
            }
            Matcher::AnchoredLiteral { literal, start, end } => match (start, end) {
                (true, true) => (text == literal).then(|| (0, text.len())),
                (true, false) => text.starts_with(literal).then(|| (0, literal.len())),
                (false, true) => text.ends_with(literal).then(|| (text.len() - literal.len(), text.len())),
                _ => unreachable!(),
            },
            Matcher::AnchoredGroup { group, start, end } => {
                match (start, end) {
                    (true, true) => {
                        // Must match entire text
                        group.match_at(text, 0).and_then(|len| {
                            if len == text.len() {
                                Some((0, len))
                            } else {
                                None
                            }
                        })
                    }
                    (true, false) => {
                        // Must match at start
                        group.match_at(text, 0).map(|len| (0, len))
                    }
                    (false, true) => {
                        // Must match at end
                        group.find(text).and_then(|(start_pos, end_pos)| {
                            if end_pos == text.len() {
                                Some((start_pos, end_pos))
                            } else {
                                None
                            }
                        })
                    }
                    _ => unreachable!(),
                }
            },
            Matcher::CharClass(cc) => {
                // Find first character matching the class
                for (idx, ch) in text.char_indices() {
                    if cc.matches(ch) {
                        return Some((idx, idx + ch.len_utf8()));
                    }
                }
                None
            }
            Matcher::Quantified(qp) => qp.find(text),
            Matcher::Sequence(seq) => seq.find(text),
            Matcher::Group(group) => group.find(text),
            Matcher::DigitRun => Self::digit_run_find(text),  // NEW: Specialized digit find
            Matcher::WordRun => Self::word_run_find(text),    // NEW: Specialized word find
            Matcher::Boundary(boundary_type) => {
                // Boundary returns position, need to map to (pos, pos) range
                boundary_type.find_first(text).map(|pos| (pos, pos))
            }
            Matcher::Lookaround(lookaround, inner_matcher) => {
                // Find first position where lookaround succeeds
                for pos in 0..=text.len() {
                    if lookaround.matches_at(text, pos, inner_matcher) {
                        return Some((pos, pos)); // Zero-width match
                    }
                }
                None
            }
            Matcher::Capture(inner_matcher, _group_index) => {
                // Capture groups don't affect position, use inner matcher
                inner_matcher.find(text)
            }
            Matcher::CombinedWithLookaround { prefix, lookaround, lookaround_matcher } => {
                // Find first position where prefix matches AND lookaround succeeds
                let mut search_pos = 0;
                while search_pos < text.len() {
                    let remaining = &text[search_pos..];
                    if let Some((rel_start, rel_end)) = prefix.find(remaining) {
                        let abs_start = search_pos + rel_start;
                        let abs_end = search_pos + rel_end;
                        
                        // Check if lookaround succeeds at the end of the prefix match
                        if lookaround.matches_at(text, abs_end, lookaround_matcher) {
                            return Some((abs_start, abs_end));
                        }
                        
                        // Move search position past this match to try next one
                        search_pos = abs_start + 1;
                    } else {
                        break;
                    }
                }
                None
            }
            Matcher::PatternWithCaptures { elements, .. } => {
                // Find first match of all elements in sequence
                for start_pos in 0..text.len() {
                    let mut pos = start_pos;
                    let mut all_matched = true;
                    
                    for element in elements {
                        let matcher = match element {
                            CompiledCaptureElement::Capture(m, _) => m,
                            CompiledCaptureElement::NonCapture(m) => m,
                        };
                        
                        if let Some((rel_start, rel_end)) = matcher.find(&text[pos..]) {
                            if rel_start != 0 {
                                // Element must match at current position
                                all_matched = false;
                                break;
                            }
                            pos += rel_end;
                        } else {
                            all_matched = false;
                            break;
                        }
                    }
                    
                    if all_matched {
                        return Some((start_pos, pos));
                    }
                }
                None
            }
        }
    }
    
    /// Find first run of digits in text
    #[inline(always)]
    fn digit_run_find(text: &str) -> Option<(usize, usize)> {
        let bytes = text.as_bytes();
        
        // Find start: first digit
        let mut start = None;
        for (i, &b) in bytes.iter().enumerate() {
            if b >= b'0' && b <= b'9' {
                start = Some(i);
                break;
            }
        }
        
        let start_idx = start?;
        
        // Find end: first non-digit after start
        let mut end_idx = bytes.len();
        for (i, &b) in bytes[start_idx..].iter().enumerate() {
            if b < b'0' || b > b'9' {
                end_idx = start_idx + i;
                break;
            }
        }
        
        Some((start_idx, end_idx))
    }
    
    /// Find first run of word characters in text
    #[inline(always)]
    fn word_run_find(text: &str) -> Option<(usize, usize)> {
        let bytes = text.as_bytes();
        
        // Find start: first word char
        let mut start = None;
        for (i, &b) in bytes.iter().enumerate() {
            if (b >= b'a' && b <= b'z') || 
               (b >= b'A' && b <= b'Z') || 
               (b >= b'0' && b <= b'9') || 
               b == b'_' {
                start = Some(i);
                break;
            }
        }
        
        let start_idx = start?;
        
        // Find end: first non-word char after start
        let mut end_idx = bytes.len();
        for (i, &b) in bytes[start_idx..].iter().enumerate() {
            if !((b >= b'a' && b <= b'z') || 
                 (b >= b'A' && b <= b'Z') || 
                 (b >= b'0' && b <= b'9') || 
                 b == b'_') {
                end_idx = start_idx + i;
                break;
            }
        }
        
        Some((start_idx, end_idx))
    }

    fn find_all(&self, text: &str) -> Vec<(usize, usize)> {
        match self {
            Matcher::Literal(lit) => {
                let finder = memmem::Finder::new(lit.as_bytes());
                finder.find_iter(text.as_bytes()).map(|pos| (pos, pos + lit.len())).collect()
            }
            Matcher::MultiLiteral(ac) => {
                ac.find_iter(text).map(|mat| (mat.start(), mat.end())).collect()
            }
            Matcher::AnchoredLiteral { .. } => {
                if let Some(m) = self.find(text) {
                    vec![m]
                } else {
                    vec![]
                }
            }
            Matcher::AnchoredGroup { .. } => {
                // Anchored groups can only match once
                if let Some(m) = self.find(text) {
                    vec![m]
                } else {
                    vec![]
                }
            }
            Matcher::CharClass(cc) => {
                // Find all characters matching the class
                text.char_indices()
                    .filter(|(_, ch)| cc.matches(*ch))
                    .map(|(idx, ch)| (idx, idx + ch.len_utf8()))
                    .collect()
            }
            Matcher::Quantified(qp) => qp.find_all(text),
            Matcher::Sequence(seq) => seq.find_all(text),
            Matcher::Group(group) => group.find_all(text),
            Matcher::DigitRun => Self::digit_run_find_all(text),  // NEW: Specialized digit find_all
            Matcher::WordRun => Self::word_run_find_all(text),    // NEW: Specialized word find_all
            Matcher::Boundary(boundary_type) => {
                // Boundary returns positions, map to (pos, pos) ranges
                boundary_type.find_all(text).into_iter().map(|pos| (pos, pos)).collect()
            }
            Matcher::Lookaround(lookaround, inner_matcher) => {
                // Find all positions where lookaround succeeds
                (0..=text.len())
                    .filter(|&pos| lookaround.matches_at(text, pos, inner_matcher))
                    .map(|pos| (pos, pos)) // Zero-width matches
                    .collect()
            }
            Matcher::Capture(inner_matcher, _group_index) => {
                // Capture groups don't affect find_all, use inner matcher
                inner_matcher.find_all(text)
            }
            Matcher::CombinedWithLookaround { prefix, lookaround, lookaround_matcher } => {
                // Find all positions where prefix matches AND lookaround succeeds
                let mut matches = Vec::new();
                let mut search_pos = 0;
                
                while search_pos < text.len() {
                    let remaining = &text[search_pos..];
                    if let Some((rel_start, rel_end)) = prefix.find(remaining) {
                        let abs_start = search_pos + rel_start;
                        let abs_end = search_pos + rel_end;
                        
                        // Check if lookaround succeeds at the end of the prefix match
                        if lookaround.matches_at(text, abs_end, lookaround_matcher) {
                            matches.push((abs_start, abs_end));
                        }
                        
                        // Move search position past the start of this match
                        search_pos = abs_start + 1;
                    } else {
                        break;
                    }
                }
                
                matches
            }
            Matcher::PatternWithCaptures { elements, .. } => {
                // Find all matches of all elements in sequence
                let mut matches = Vec::new();
                let mut start_pos = 0;
                
                while start_pos < text.len() {
                    let mut pos = start_pos;
                    let mut all_matched = true;
                    
                    for element in elements {
                        let matcher = match element {
                            CompiledCaptureElement::Capture(m, _) => m,
                            CompiledCaptureElement::NonCapture(m) => m,
                        };
                        
                        if let Some((rel_start, rel_end)) = matcher.find(&text[pos..]) {
                            if rel_start != 0 {
                                // Element must match at current position
                                all_matched = false;
                                break;
                            }
                            pos += rel_end;
                        } else {
                            all_matched = false;
                            break;
                        }
                    }
                    
                    if all_matched {
                        matches.push((start_pos, pos));
                        start_pos = pos.max(start_pos + 1); // Move past this match
                    } else {
                        start_pos += 1;
                    }
                }
                
                matches
            }
        }
    }
    
    /// Find all runs of digits in text (optimized)
    #[inline]
    fn digit_run_find_all(text: &str) -> Vec<(usize, usize)> {
        let bytes = text.as_bytes();
        let mut matches = Vec::new();
        let mut i = 0;
        
        while i < bytes.len() {
            // Skip non-digits
            while i < bytes.len() && (bytes[i] < b'0' || bytes[i] > b'9') {
                i += 1;
            }
            
            if i >= bytes.len() {
                break;
            }
            
            // Found start of digit run
            let start = i;
            
            // Consume all digits
            while i < bytes.len() && bytes[i] >= b'0' && bytes[i] <= b'9' {
                i += 1;
            }
            
            matches.push((start, i));
        }
        
        matches
    }
    
    /// Find all runs of word characters in text (optimized)
    #[inline]
    fn word_run_find_all(text: &str) -> Vec<(usize, usize)> {
        let bytes = text.as_bytes();
        let mut matches = Vec::new();
        let mut i = 0;
        
        while i < bytes.len() {
            // Skip non-word chars
            while i < bytes.len() {
                let b = bytes[i];
                if (b >= b'a' && b <= b'z') || 
                   (b >= b'A' && b <= b'Z') || 
                   (b >= b'0' && b <= b'9') || 
                   b == b'_' {
                    break;
                }
                i += 1;
            }
            
            if i >= bytes.len() {
                break;
            }
            
            // Found start of word run
            let start = i;
            
            // Consume all word chars
            while i < bytes.len() {
                let b = bytes[i];
                if !((b >= b'a' && b <= b'z') || 
                     (b >= b'A' && b <= b'Z') || 
                     (b >= b'0' && b <= b'9') || 
                     b == b'_') {
                    break;
                }
                i += 1;
            }
            
            matches.push((start, i));
        }
        
        matches
    }
}

fn compile_ast(ast: &Ast) -> Result<Matcher, PatternError> {
    match ast {
        Ast::Literal(lit) => Ok(Matcher::Literal(lit.clone())),
        Ast::Alternation(parts) => {
            let ac = AhoCorasick::new(parts)
                .map_err(|e| PatternError::ParseError(format!("Aho-Corasick: {}", e)))?;
            Ok(Matcher::MultiLiteral(ac))
        }
        Ast::Anchored { literal, start, end } => Ok(Matcher::AnchoredLiteral {
            literal: literal.clone(),
            start: *start,
            end: *end,
        }),
        Ast::AnchoredGroup { group, start, end } => Ok(Matcher::AnchoredGroup {
            group: group.clone(),
            start: *start,
            end: *end,
        }),
        Ast::CharClass(cc) => Ok(Matcher::CharClass(cc.clone())),
        Ast::Quantified(qp) => {
            // OPTIMIZATION: Detect \d+ and \w+ patterns for specialized fast path
            if let crate::quantifier::Quantifier::OneOrMore = qp.quantifier {
                if let crate::quantifier::QuantifiedElement::CharClass(ref cc) = qp.element {
                    // Check if this is \d+ (digits)
                    if is_digit_charclass(cc) {
                        return Ok(Matcher::DigitRun);
                    }
                    // Check if this is \w+ (word chars)
                    if is_word_charclass(cc) {
                        return Ok(Matcher::WordRun);
                    }
                }
            }
            Ok(Matcher::Quantified(qp.clone()))
        }
        Ast::Sequence(seq) => Ok(Matcher::Sequence(seq.clone())),
        Ast::Group(group) => Ok(Matcher::Group(group.clone())),
        Ast::Boundary(boundary_type) => Ok(Matcher::Boundary(*boundary_type)),
        Ast::Lookaround(lookaround) => {
            // Compile the inner pattern of the lookaround
            let inner_matcher = compile_ast(&lookaround.pattern)?;
            Ok(Matcher::Lookaround(
                Box::new(lookaround.clone()),
                Box::new(inner_matcher)
            ))
        }
        Ast::Capture(inner_ast, group_index) => {
            // Compile the inner pattern of the capture group
            let inner_matcher = compile_ast(inner_ast)?;
            Ok(Matcher::Capture(Box::new(inner_matcher), *group_index))
        }
        Ast::CombinedWithLookaround { prefix, lookaround } => {
            // Compile both the prefix and the lookaround's inner pattern
            let prefix_matcher = compile_ast(prefix)?;
            let lookaround_inner = compile_ast(&lookaround.pattern)?;
            Ok(Matcher::CombinedWithLookaround {
                prefix: Box::new(prefix_matcher),
                lookaround: Box::new(lookaround.clone()),
                lookaround_matcher: Box::new(lookaround_inner),
            })
        }
        Ast::PatternWithCaptures { elements, total_groups } => {
            // Compile each element
            let mut compiled_elements = Vec::new();
            for elem in elements {
                match elem {
                    CaptureElement::Capture(ast, group_num) => {
                        let matcher = compile_ast(ast)?;
                        compiled_elements.push(CompiledCaptureElement::Capture(matcher, *group_num));
                    }
                    CaptureElement::NonCapture(ast) => {
                        let matcher = compile_ast(ast)?;
                        compiled_elements.push(CompiledCaptureElement::NonCapture(matcher));
                    }
                }
            }
            Ok(Matcher::PatternWithCaptures {
                elements: compiled_elements,
                total_groups: *total_groups,
            })
        }
    }
}

/// Check if CharClass matches \d pattern (only [0-9])
fn is_digit_charclass(cc: &CharClass) -> bool {
    // Check if ranges contain exactly [0-9] and no other chars
    cc.ranges.len() == 1 && 
    cc.ranges[0] == ('0', '9') && 
    cc.chars.is_empty() && 
    !cc.negated
}

/// Check if CharClass matches \w pattern ([a-zA-Z0-9_])
fn is_word_charclass(cc: &CharClass) -> bool {
    // Check if ranges contain [a-z], [A-Z], [0-9] and chars contain '_'
    if cc.negated || cc.ranges.len() != 3 {
        return false;
    }
    
    let mut has_lower = false;
    let mut has_upper = false;
    let mut has_digit = false;
    
    for &(start, end) in &cc.ranges {
        if start == 'a' && end == 'z' {
            has_lower = true;
        } else if start == 'A' && end == 'Z' {
            has_upper = true;
        } else if start == '0' && end == '9' {
            has_digit = true;
        }
    }
    
    has_lower && has_upper && has_digit && 
    cc.chars.len() == 1 && cc.chars[0] == '_'
}

/// Parse lookaround assertion patterns: (?=...), (?!...), (?<=...), (?<!...)
fn parse_lookaround(pattern: &str) -> Result<Ast, PatternError> {
    let lookaround_type = if pattern.starts_with("(?=") {
        LookaroundType::PositiveLookahead
    } else if pattern.starts_with("(?!") {
        LookaroundType::NegativeLookahead
    } else if pattern.starts_with("(?<=") {
        LookaroundType::PositiveLookbehind
    } else if pattern.starts_with("(?<!") {
        LookaroundType::NegativeLookbehind
    } else {
        return Err(PatternError::ParseError("Invalid lookaround syntax".to_string()));
    };
    
    // Find the matching closing parenthesis
    let prefix_len = if pattern.starts_with("(?<=") || pattern.starts_with("(?<!") {
        4 // "(?<=" or "(?<!"
    } else {
        3 // "(?=" or "(?!"
    };
    
    if let Some(close_idx) = find_matching_paren(pattern, 0) {
        if close_idx != pattern.len() - 1 {
            return Err(PatternError::ParseError(
                "Lookaround must be entire pattern (combined patterns not yet supported)".to_string()
            ));
        }
        
        let inner = &pattern[prefix_len..close_idx];
        let inner_ast = parse_pattern(inner)?;
        
        Ok(Ast::Lookaround(Lookaround::new(lookaround_type, inner_ast)))
    } else {
        Err(PatternError::ParseError("Unmatched parenthesis in lookaround".to_string()))
    }
}

/// Parse combined patterns with lookaround: foo(?=bar), \d+(?!x), etc.
fn parse_combined_with_lookaround(pattern: &str) -> Result<Ast, PatternError> {
    // Find the lookaround position
    let lookaround_patterns = ["(?=", "(?!", "(?<=", "(?<!"];
    
    for lookaround_start in lookaround_patterns {
        if let Some(pos) = pattern.find(lookaround_start) {
            if pos == 0 {
                // This is a standalone lookaround, not combined
                continue;
            }
            
            // Split into prefix and lookaround
            let prefix = &pattern[..pos];
            let lookaround_part = &pattern[pos..];
            
            // Parse the prefix
            let prefix_ast = parse_pattern(prefix)?;
            
            // Parse the lookaround
            let lookaround_type = if lookaround_start == "(?=" {
                LookaroundType::PositiveLookahead
            } else if lookaround_start == "(?!" {
                LookaroundType::NegativeLookahead
            } else if lookaround_start == "(?<=" {
                LookaroundType::PositiveLookbehind
            } else {
                LookaroundType::NegativeLookbehind
            };
            
            let prefix_len = lookaround_start.len();
            if let Some(close_idx) = find_matching_paren(lookaround_part, 0) {
                if close_idx != lookaround_part.len() - 1 {
                    return Err(PatternError::ParseError(
                        "Extra characters after lookaround".to_string()
                    ));
                }
                
                let inner = &lookaround_part[prefix_len..close_idx];
                let inner_ast = parse_pattern(inner)?;
                
                let lookaround = Lookaround::new(lookaround_type, inner_ast);
                
                return Ok(Ast::CombinedWithLookaround {
                    prefix: Box::new(prefix_ast),
                    lookaround,
                });
            } else {
                return Err(PatternError::ParseError("Unmatched parenthesis in lookaround".to_string()));
            }
        }
    }
    
    Err(PatternError::ParseError("No lookaround found in pattern".to_string()))
}

/// Find the index of the matching closing parenthesis
/// Returns None if no match found
fn find_matching_paren(pattern: &str, start: usize) -> Option<usize> {
    let bytes = pattern.as_bytes();
    if start >= bytes.len() || bytes[start] != b'(' {
        return None;
    }
    
    let mut depth = 0;
    for (i, &ch) in bytes[start..].iter().enumerate() {
        match ch {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + i);
                }
            }
            b'\\' => {
                // Skip next character (could be escaped parenthesis)
                // This is a simple approach, doesn't handle all edge cases
            }
            _ => {}
        }
    }
    
    None // Unmatched
}

/// Parse patterns with embedded capture groups: Hello (\w+), (\w+)=(\d+), (\d{4})-(\d{2})-(\d{2})
/// Returns an AST that represents a sequence with captures
fn parse_pattern_with_captures(pattern: &str) -> Result<Ast, PatternError> {
    // This is a complex parser that needs to:
    // 1. Find all capture groups in the pattern
    // 2. Parse the parts between captures
    // 3. Assign group numbers
    // 4. Build an AST that tracks all of this
    
    // For now, let's implement a simple version that handles basic cases
    // like: literal(\w+), (\w+)literal, (\w+)(\d+)
    
    let mut elements: Vec<CaptureElement> = Vec::new();
    let mut group_number = 1;
    let mut pos = 0;
    
    while pos < pattern.len() {
        if pattern[pos..].starts_with("(?:") {
            // Found a non-capturing group (?:...)
            if let Some(close_idx) = find_matching_paren(pattern, pos) {
                // Parse the content as a non-capturing group
                let inner = &pattern[pos+3..close_idx]; // Skip "(?:"
                let inner_ast = parse_pattern(inner)?;
                
                elements.push(CaptureElement::NonCapture(inner_ast));
                pos = close_idx + 1;
            } else {
                return Err(PatternError::ParseError("Unmatched parenthesis".to_string()));
            }
        } else if pattern[pos..].starts_with('(') && !pattern[pos..].starts_with("(?") {
            // Found a capture group
            if let Some(close_idx) = find_matching_paren(pattern, pos) {
                // Parse the content of the capture
                let inner = &pattern[pos+1..close_idx];
                let inner_ast = parse_pattern(inner)?;
                
                elements.push(CaptureElement::Capture(inner_ast, group_number));
                group_number += 1;
                pos = close_idx + 1;
            } else {
                return Err(PatternError::ParseError("Unmatched parenthesis".to_string()));
            }
        } else {
            // Find the next capture group or end of pattern
            let next_paren = pattern[pos..].find('(').map(|i| pos + i).unwrap_or(pattern.len());
            
            if next_paren > pos {
                // There's a literal or other pattern before the next capture
                let segment = &pattern[pos..next_paren];
                let segment_ast = parse_pattern(segment)?;
                elements.push(CaptureElement::NonCapture(segment_ast));
                pos = next_paren;
            } else {
                // Move forward
                pos += 1;
            }
        }
    }
    
    // If we only have one element and it's a single capture, return it directly
    if elements.len() == 1 {
        if let CaptureElement::Capture(ast, num) = &elements[0] {
            return Ok(Ast::Capture(Box::new(ast.clone()), *num));
        }
    }
    
    // Build a PatternWithCaptures AST
    Ok(Ast::PatternWithCaptures { elements, total_groups: group_number - 1 })
}

/// Element in a pattern with captures
#[derive(Debug, Clone, PartialEq)]
enum CaptureElement {
    Capture(Ast, usize),      // (pattern), group_number
    NonCapture(Ast),          // literal or other pattern
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal() {
        let p = Pattern::new("hello").unwrap();
        assert!(p.is_match("hello world"));
        assert!(!p.is_match("goodbye"));
    }

    #[test]
    fn alternation() {
        let p = Pattern::new("foo|bar|baz").unwrap();
        assert!(p.is_match("foo"));
        assert!(p.is_match("bar"));
        assert!(!p.is_match("qux"));
    }

    #[test]
    fn anchors() {
        let p = Pattern::new("^hello$").unwrap();
        assert!(p.is_match("hello"));
        assert!(!p.is_match("hello world"));
    }

    #[test]
    fn find_test() {
        let p = Pattern::new("world").unwrap();
        assert_eq!(p.find("hello world"), Some((6, 11)));
    }

    #[test]
    fn cached() {
        assert!(is_match("test", "this is a test").unwrap());
    }
}

    #[test]
    fn char_class_simple() {
        let p = Pattern::new("[abc]").unwrap();
        assert!(p.is_match("a"));
        assert!(p.is_match("apple"));
        assert!(p.is_match("cab"));
        assert!(!p.is_match("xyz"));
    }
    
    #[test]
    fn char_class_range() {
        let p = Pattern::new("[a-z]").unwrap();
        assert!(p.is_match("hello"));
        assert!(p.is_match("xyz"));
        assert!(!p.is_match("HELLO"));
        assert!(!p.is_match("123"));
    }
    
    #[test]
    fn char_class_multiple_ranges() {
        let p = Pattern::new("[a-zA-Z0-9]").unwrap();
        assert!(p.is_match("hello"));
        assert!(p.is_match("WORLD"));
        assert!(p.is_match("test123"));
        assert!(!p.is_match("!!!"));
    }
    
    #[test]
    fn char_class_negated() {
        let p = Pattern::new("[^0-9]").unwrap();
        assert!(p.is_match("abc"));
        assert!(!p.is_match("123"));
        assert!(p.is_match("a1b")); // Contains non-digit
    }
    
    #[test]
    fn char_class_find() {
        let p = Pattern::new("[0-9]").unwrap();
        assert_eq!(p.find("abc123"), Some((3, 4))); // Finds 1
        
        let matches = p.find_all("a1b2c3");
        assert_eq!(matches, vec![(1, 2), (3, 4), (5, 6)]);
    }

