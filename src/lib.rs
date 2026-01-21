//! ReXile - Fast regex-lite engine built on memchr and aho-corasick
//!
//! **Zero dependency on the regex crate!**

mod charclass;
mod quantifier;
mod escape;
mod sequence;
mod sequence_parser;
mod group;

use aho_corasick::AhoCorasick;
use charclass::CharClass;
use escape::{parse_escape, starts_with_escape};
use memchr::memmem;
use sequence::Sequence;
use sequence_parser::{is_sequence_pattern, parse_sequence};
use group::Group;
use quantifier::{parse_quantified_pattern, QuantifiedPattern};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

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
        self.matcher.find_all(text)
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

#[derive(Debug, Clone)]
enum Ast {
    Literal(String),
    Alternation(Vec<String>),
    Anchored { literal: String, start: bool, end: bool },
    CharClass(CharClass),
    Quantified(QuantifiedPattern),
    Sequence(Sequence),
    Group(Group),  // NEW: Group support
}

fn parse_pattern(pattern: &str) -> Result<Ast, PatternError> {
    if pattern.is_empty() {
        return Ok(Ast::Literal(String::new()));
    }
    
    // Check for groups FIRST (before anything else except anchors)
    if pattern.starts_with('(') {
        match group::parse_group(pattern) {
            Ok((group, bytes_consumed)) => {
                // If the whole pattern is just a group, return it
                if bytes_consumed == pattern.len() {
                    return Ok(Ast::Group(group));
                }
                // Otherwise might be part of a sequence - fall through
            }
            Err(_) => {
                // Not a valid group, try other parsers
            }
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
    
    // Check for escape sequences: \d, \w, \s, \., etc.
    if starts_with_escape(pattern) {
        match parse_escape(pattern) {
            Ok((seq, bytes_consumed)) => {
                // If it's the whole pattern
                if bytes_consumed == pattern.len() {
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
                        if q_char == '*' || q_char == '+' || q_char == '?' {
                            // This is an escape with quantifier: \d+, \w*, etc.
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
    CharClass(CharClass),
    Quantified(QuantifiedPattern),
    Sequence(Sequence),
    Group(Group),  // NEW: Group matcher
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
            Matcher::CharClass(cc) => {
                // Character class matches if ANY character in text matches the class
                text.chars().any(|ch| cc.matches(ch))
            }
            Matcher::Quantified(qp) => qp.find(text).is_some(),
            Matcher::Sequence(seq) => seq.find(text).is_some(),
            Matcher::Group(group) => group.find(text).is_some(),
        }
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
        }
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
        }
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
        Ast::CharClass(cc) => Ok(Matcher::CharClass(cc.clone())),
        Ast::Quantified(qp) => Ok(Matcher::Quantified(qp.clone())),
        Ast::Sequence(seq) => Ok(Matcher::Sequence(seq.clone())),
        Ast::Group(group) => Ok(Matcher::Group(group.clone())),
    }
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

