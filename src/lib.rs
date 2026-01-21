//! ReXile - Fast regex-lite engine built on memchr and aho-corasick
//!
//! **Zero dependency on the regex crate!**

use aho_corasick::AhoCorasick;
use memchr::memmem;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Pattern {
    matcher: Matcher,
}

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

static CACHE: Lazy<Mutex<HashMap<String, Pattern>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn get_pattern(pattern: &str) -> Result<Pattern, PatternError> {
    let mut cache = CACHE.lock().unwrap();
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
}

fn parse_pattern(pattern: &str) -> Result<Ast, PatternError> {
    if pattern.is_empty() {
        return Ok(Ast::Literal(String::new()));
    }
    if pattern.contains('|') {
        let parts: Vec<String> = pattern.split('|').map(|s| s.to_string()).collect();
        return Ok(Ast::Alternation(parts));
    }
    let has_start = pattern.starts_with('^');
    let has_end = pattern.ends_with('$');
    if has_start || has_end {
        let core = if has_start && has_end {
            &pattern[1..pattern.len() - 1]
        } else if has_start {
            &pattern[1..]
        } else {
            &pattern[..pattern.len() - 1]
        };
        return Ok(Ast::Anchored {
            literal: core.to_string(),
            start: has_start,
            end: has_end,
        });
    }
    Ok(Ast::Literal(pattern.to_string()))
}

#[derive(Debug, Clone)]
enum Matcher {
    Literal(String),
    MultiLiteral(AhoCorasick),
    AnchoredLiteral { literal: String, start: bool, end: bool },
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
